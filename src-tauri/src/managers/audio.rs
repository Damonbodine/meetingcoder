use crate::audio_toolkit::{list_input_devices, vad::SmoothedVad, AudioRecorder, SileroVad};
use crate::settings::get_settings;
use crate::system_audio::SendableSystemAudio;
use crate::utils;
use log::{debug, info};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::Manager;

const WHISPER_SAMPLE_RATE: usize = 16000;

/* ──────────────────────────────────────────────────────────────── */

#[derive(Clone, Debug)]
pub enum RecordingState {
    Idle,
    Recording { binding_id: String },
}

#[derive(Clone, Debug)]
pub enum MicrophoneMode {
    AlwaysOn,
    OnDemand,
}

#[derive(Clone, Debug)]
pub enum AudioSource {
    Microphone,
    SystemAudio(String), // device_name
}

/* ──────────────────────────────────────────────────────────────── */

fn create_audio_recorder(
    vad_path: &str,
    app_handle: &tauri::AppHandle,
) -> Result<AudioRecorder, anyhow::Error> {
    let silero = SileroVad::new(vad_path, 0.3)
        .map_err(|e| anyhow::anyhow!("Failed to create SileroVad: {}", e))?;
    let smoothed_vad = SmoothedVad::new(Box::new(silero), 15, 15, 2);

    // Recorder with VAD plus a spectrum-level callback that forwards updates to
    // the frontend.
    let recorder = AudioRecorder::new()
        .map_err(|e| anyhow::anyhow!("Failed to create AudioRecorder: {}", e))?
        .with_vad(Box::new(smoothed_vad))
        .with_level_callback({
            let app_handle = app_handle.clone();
            move |levels| {
                utils::emit_levels(&app_handle, &levels);
            }
        });

    Ok(recorder)
}

/* ──────────────────────────────────────────────────────────────── */

#[derive(Clone)]
pub struct AudioRecordingManager {
    state: Arc<Mutex<RecordingState>>,
    mode: Arc<Mutex<MicrophoneMode>>,
    app_handle: tauri::AppHandle,

    recorder: Arc<Mutex<Option<AudioRecorder>>>,
    is_open: Arc<Mutex<bool>>,
    is_recording: Arc<Mutex<bool>>,
    initial_volume: Arc<Mutex<Option<u8>>>,

    // System audio capture
    system_audio: Arc<Mutex<Option<SendableSystemAudio>>>,
    current_source: Arc<Mutex<AudioSource>>,
    system_audio_buffer: Arc<Mutex<Vec<f32>>>,
}

impl AudioRecordingManager {
    /* ---------- construction ------------------------------------------------ */

    pub fn new(app: &tauri::AppHandle) -> Result<Self, anyhow::Error> {
        let settings = get_settings(app);
        let mode = if settings.always_on_microphone {
            MicrophoneMode::AlwaysOn
        } else {
            MicrophoneMode::OnDemand
        };

        let manager = Self {
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            mode: Arc::new(Mutex::new(mode.clone())),
            app_handle: app.clone(),

            recorder: Arc::new(Mutex::new(None)),
            is_open: Arc::new(Mutex::new(false)),
            is_recording: Arc::new(Mutex::new(false)),
            initial_volume: Arc::new(Mutex::new(None)),

            system_audio: Arc::new(Mutex::new(None)),
            current_source: Arc::new(Mutex::new(AudioSource::Microphone)),
            system_audio_buffer: Arc::new(Mutex::new(Vec::new())),
        };

        // Always-on?  Open immediately.
        if matches!(mode, MicrophoneMode::AlwaysOn) {
            manager.start_microphone_stream()?;
        }

        Ok(manager)
    }

    /* ---------- microphone life-cycle -------------------------------------- */

    pub fn start_microphone_stream(&self) -> Result<(), anyhow::Error> {
        let mut open_flag = self.is_open.lock().unwrap();
        if *open_flag {
            debug!("Microphone stream already active");
            return Ok(());
        }

        let start_time = Instant::now();

        let settings = get_settings(&self.app_handle);
        let mut initial_volume_guard = self.initial_volume.lock().unwrap();

        if settings.mute_while_recording {
            *initial_volume_guard = Some(cpvc::get_system_volume());
            cpvc::set_system_volume(0);
        } else {
            *initial_volume_guard = None;
        }

        let vad_path = self
            .app_handle
            .path()
            .resolve(
                "resources/models/silero_vad_v4.onnx",
                tauri::path::BaseDirectory::Resource,
            )
            .map_err(|e| anyhow::anyhow!("Failed to resolve VAD path: {}", e))?;
        let mut recorder_opt = self.recorder.lock().unwrap();

        if recorder_opt.is_none() {
            *recorder_opt = Some(create_audio_recorder(
                vad_path.to_str().unwrap(),
                &self.app_handle,
            )?);
        }

        // Get the selected device from settings
        let settings = get_settings(&self.app_handle);
        let selected_device = if let Some(device_name) = settings.selected_microphone {
            // Find the device by name
            match list_input_devices() {
                Ok(devices) => devices
                    .into_iter()
                    .find(|d| d.name == device_name)
                    .map(|d| d.device),
                Err(e) => {
                    debug!("Failed to list devices, using default: {}", e);
                    None
                }
            }
        } else {
            None
        };

        if let Some(rec) = recorder_opt.as_mut() {
            rec.open(selected_device)
                .map_err(|e| anyhow::anyhow!("Failed to open recorder: {}", e))?;
        }

        *open_flag = true;
        info!(
            "Microphone stream initialized in {:?}",
            start_time.elapsed()
        );
        Ok(())
    }

    pub fn stop_microphone_stream(&self) {
        let mut open_flag = self.is_open.lock().unwrap();
        if !*open_flag {
            return;
        }

        let mut initial_volume_guard = self.initial_volume.lock().unwrap();
        if let Some(vol) = *initial_volume_guard {
            cpvc::set_system_volume(vol);
        }
        *initial_volume_guard = None;

        if let Some(rec) = self.recorder.lock().unwrap().as_mut() {
            // If still recording, stop first.
            if *self.is_recording.lock().unwrap() {
                let _ = rec.stop();
                *self.is_recording.lock().unwrap() = false;
            }
            let _ = rec.close();
        }

        *open_flag = false;
        debug!("Microphone stream stopped");
    }

    /* ---------- mode switching --------------------------------------------- */

    pub fn update_mode(&self, new_mode: MicrophoneMode) -> Result<(), anyhow::Error> {
        let mode_guard = self.mode.lock().unwrap();
        let cur_mode = mode_guard.clone();

        match (cur_mode, &new_mode) {
            (MicrophoneMode::AlwaysOn, MicrophoneMode::OnDemand) => {
                if matches!(*self.state.lock().unwrap(), RecordingState::Idle) {
                    drop(mode_guard);
                    self.stop_microphone_stream();
                }
            }
            (MicrophoneMode::OnDemand, MicrophoneMode::AlwaysOn) => {
                drop(mode_guard);
                self.start_microphone_stream()?;
            }
            _ => {}
        }

        *self.mode.lock().unwrap() = new_mode;
        Ok(())
    }

    /* ---------- recording --------------------------------------------------- */

    pub fn try_start_recording(&self, binding_id: &str) -> bool {
        let mut state = self.state.lock().unwrap();

        if let RecordingState::Idle = *state {
            // Ensure microphone is open in on-demand mode
            if matches!(*self.mode.lock().unwrap(), MicrophoneMode::OnDemand) {
                if let Err(e) = self.start_microphone_stream() {
                    eprintln!("Failed to open microphone stream: {e}");
                    return false;
                }
            }

            if let Some(rec) = self.recorder.lock().unwrap().as_ref() {
                if rec.start().is_ok() {
                    *self.is_recording.lock().unwrap() = true;
                    *state = RecordingState::Recording {
                        binding_id: binding_id.to_string(),
                    };
                    debug!("Recording started for binding {binding_id}");
                    return true;
                }
            }
            eprintln!("Recorder not available");
            false
        } else {
            false
        }
    }

    pub fn update_selected_device(&self) -> Result<(), anyhow::Error> {
        // If currently open, restart the microphone stream to use the new device
        if *self.is_open.lock().unwrap() {
            self.stop_microphone_stream();
            self.start_microphone_stream()?;
        }
        Ok(())
    }

    pub fn stop_recording(&self, binding_id: &str) -> Option<Vec<f32>> {
        let mut state = self.state.lock().unwrap();

        match *state {
            RecordingState::Recording {
                binding_id: ref active,
            } if active == binding_id => {
                *state = RecordingState::Idle;
                drop(state);

                let samples = if let Some(rec) = self.recorder.lock().unwrap().as_ref() {
                    match rec.stop() {
                        Ok(buf) => buf,
                        Err(e) => {
                            eprintln!("stop() failed: {e}");
                            Vec::new()
                        }
                    }
                } else {
                    eprintln!("Recorder not available");
                    Vec::new()
                };

                *self.is_recording.lock().unwrap() = false;

                // In on-demand mode turn the mic off again
                if matches!(*self.mode.lock().unwrap(), MicrophoneMode::OnDemand) {
                    self.stop_microphone_stream();
                }

                // Pad if very short
                let s_len = samples.len();
                // println!("Got {} samples", { s_len });
                if s_len < WHISPER_SAMPLE_RATE && s_len > 0 {
                    let mut padded = samples;
                    padded.resize(WHISPER_SAMPLE_RATE * 5 / 4, 0.0);
                    Some(padded)
                } else {
                    Some(samples)
                }
            }
            _ => None,
        }
    }

    /// Cancel any ongoing recording without returning audio samples
    pub fn cancel_recording(&self) {
        let mut state = self.state.lock().unwrap();

        if let RecordingState::Recording { .. } = *state {
            *state = RecordingState::Idle;
            drop(state);

            if let Some(rec) = self.recorder.lock().unwrap().as_ref() {
                let _ = rec.stop(); // Discard the result
            }

            *self.is_recording.lock().unwrap() = false;

            // In on-demand mode turn the mic off again
            if matches!(*self.mode.lock().unwrap(), MicrophoneMode::OnDemand) {
                self.stop_microphone_stream();
            }
        }
    }

    /* ---------- system audio support ---------------------------------------- */

    /// Start system audio capture
    pub fn start_system_audio(&self, device_name: String) -> Result<(), anyhow::Error> {
        let mut sys_audio = self.system_audio.lock().unwrap();

        // Create system audio capturer if not exists
        if sys_audio.is_none() {
            *sys_audio = Some(SendableSystemAudio::new()?);
        }

        // Start capturing with buffer
        if let Some(ref capturer) = *sys_audio {
            let buffer = self.system_audio_buffer.clone();
            capturer.start_capture(Some(device_name.clone()), buffer)?;
            info!("System audio capture started from device: {}", device_name);
        }

        // Update current source
        *self.current_source.lock().unwrap() = AudioSource::SystemAudio(device_name);
        *self.is_open.lock().unwrap() = true;

        Ok(())
    }

    /// Stop system audio capture
    pub fn stop_system_audio(&self) -> Result<(), anyhow::Error> {
        let mut sys_audio = self.system_audio.lock().unwrap();

        if let Some(ref mut capturer) = *sys_audio {
            capturer.stop_capture()?;
            info!("System audio capture stopped");
        }

        *self.is_open.lock().unwrap() = false;
        Ok(())
    }

    /// Set the audio source (microphone or system audio)
    pub fn set_audio_source(&self, source: AudioSource) -> Result<(), anyhow::Error> {
        // Stop current source
        match *self.current_source.lock().unwrap() {
            AudioSource::Microphone => {
                if *self.is_open.lock().unwrap() {
                    self.stop_microphone_stream();
                }
            }
            AudioSource::SystemAudio(_) => {
                if *self.is_open.lock().unwrap() {
                    self.stop_system_audio()?;
                }
            }
        }

        // Start new source
        match source {
            AudioSource::Microphone => {
                self.start_microphone_stream()?;
                *self.current_source.lock().unwrap() = AudioSource::Microphone;
            }
            AudioSource::SystemAudio(device_name) => {
                self.start_system_audio(device_name)?;
            }
        }

        Ok(())
    }

    /// Get buffered audio from system audio (for continuous recording)
    pub fn get_system_audio_buffer(&self, duration_secs: f32) -> Vec<f32> {
        let mut buffer = self.system_audio_buffer.lock().unwrap();
        let samples_needed = (WHISPER_SAMPLE_RATE as f32 * duration_secs) as usize;

        if buffer.len() >= samples_needed {
            let chunk: Vec<f32> = buffer.drain(..samples_needed).collect();
            chunk
        } else {
            // Return what we have and clear
            let chunk = buffer.clone();
            buffer.clear();
            chunk
        }
    }

    /// Get current audio source
    pub fn get_audio_source(&self) -> AudioSource {
        self.current_source.lock().unwrap().clone()
    }

    /// Get the current buffer size (for testing/debugging)
    pub fn get_system_audio_buffer_size(&self) -> usize {
        self.system_audio_buffer.lock().unwrap().len()
    }

    /// Clear the system audio buffer
    pub fn clear_system_audio_buffer(&self) {
        self.system_audio_buffer.lock().unwrap().clear();
    }

    /// Save the current buffer to a WAV file (for testing/debugging)
    pub fn save_system_audio_buffer_to_wav(&self, filename: &str) -> Result<String, anyhow::Error> {
        use hound::{WavSpec, WavWriter};

        let buffer = self.system_audio_buffer.lock().unwrap();

        if buffer.is_empty() {
            return Err(anyhow::anyhow!("Buffer is empty, nothing to save"));
        }

        // Save to Desktop for easy access
        let desktop_dir = self.app_handle
            .path()
            .desktop_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get desktop dir: {}", e))?;

        let filepath = desktop_dir.join(filename);

        let spec = WavSpec {
            channels: 1,
            sample_rate: WHISPER_SAMPLE_RATE as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut writer = WavWriter::create(&filepath, spec)?;

        for &sample in buffer.iter() {
            writer.write_sample(sample)?;
        }

        writer.finalize()?;

        info!("Saved {} samples to {:?}", buffer.len(), filepath);
        Ok(filepath.to_string_lossy().to_string())
    }
}
