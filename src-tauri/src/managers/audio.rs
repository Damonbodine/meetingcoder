use crate::audio_toolkit::{list_input_devices, vad::SmoothedVad, AudioRecorder, SileroVad};
use crate::settings::get_settings;
use crate::system_audio::{ring_buffer::SpscRingBuffer, SendableSystemAudio};
use crate::utils;
use log::{debug, info};
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
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
    system_audio_buffer: Arc<Mutex<Arc<SpscRingBuffer>>>,
    silent_chunks: AtomicU64,
    restart_attempts_total: AtomicU64,
    restart_successes: AtomicU64,
    last_restart_error: Arc<Mutex<Option<String>>>,
    // Persist recent audio errors (most recent first, max 10)
    recent_errors: Arc<Mutex<VecDeque<String>>>,
    // Timestamps of restart attempts (for rate metrics)
    restart_attempt_times: Arc<Mutex<VecDeque<Instant>>>,
    // Track last attempt/success instants for cooldown calculation
    last_restart_attempt: Arc<Mutex<Option<Instant>>>,
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
            // Default capacity based on settings
            system_audio_buffer: Arc::new(Mutex::new(SpscRingBuffer::new(
                WHISPER_SAMPLE_RATE * (settings.system_audio_buffer_seconds.max(1) as usize),
            ))),
            silent_chunks: AtomicU64::new(0),
            restart_attempts_total: AtomicU64::new(0),
            restart_successes: AtomicU64::new(0),
            last_restart_error: Arc::new(Mutex::new(None)),
            recent_errors: Arc::new(Mutex::new(VecDeque::with_capacity(10))),
            restart_attempt_times: Arc::new(Mutex::new(VecDeque::with_capacity(64))),
            last_restart_attempt: Arc::new(Mutex::new(None)),
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
            let buffer = self.system_audio_buffer.lock().unwrap().clone();
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

    /// Restart system audio capture (stops and starts again)
    pub fn restart_system_audio(&self) -> Result<(), anyhow::Error> {
        info!("Restarting system audio capture...");

        // Get the current device name before stopping
        let device_name = match self.current_source.lock().unwrap().clone() {
            AudioSource::SystemAudio(name) => name,
            _ => {
                return Err(anyhow::anyhow!("Not currently using system audio"));
            }
        };

        // Stop current capture
        self.stop_system_audio()
            .map_err(|e| anyhow::anyhow!("Failed to stop audio during restart: {}", e))?;

        // Clear the audio buffer to prevent corrupted samples
        info!("Clearing audio buffer during restart");
        self.clear_system_audio_buffer();

        // Small delay to let the system clean up (increased to 1s for safety)
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Verify device still exists before attempting restart
        if let Some(ref sys_audio) = *self.system_audio.lock().unwrap() {
            match sys_audio.list_output_devices() {
                Ok(devices) => {
                    if !devices.iter().any(|d| d.name == device_name) {
                        return Err(anyhow::anyhow!(
                            "Audio device '{}' no longer available. Please check your audio settings.",
                            device_name
                        ));
                    }
                }
                Err(e) => {
                    log::warn!("Could not verify device existence: {}", e);
                    // Continue anyway, start_system_audio will fail if device is truly gone
                }
            }
        }

        // Restart capture
        self.start_system_audio(device_name.clone()).map_err(|e| {
            anyhow::anyhow!("Failed to start audio device '{}': {}. Check if BlackHole is still installed and set as output.", device_name, e)
        })?;

        info!("System audio capture restarted successfully");
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
        let samples_needed = (WHISPER_SAMPLE_RATE as f32 * duration_secs) as usize;
        let buffer = self.system_audio_buffer.lock().unwrap().clone();
        let chunk = buffer.drain_n(samples_needed);

        // Silence gating: compute RMS; track below-threshold chunks for diagnostics
        if !chunk.is_empty() {
            let rms = (chunk.iter().map(|v| v * v).sum::<f32>() / (chunk.len() as f32)).sqrt();
            let dbfs = 20.0 * (rms.max(1e-12)).log10();
            let settings = get_settings(&self.app_handle);
            if dbfs < settings.system_audio_silence_threshold {
                self.silent_chunks.fetch_add(1, Ordering::Relaxed);
            }
        }

        chunk
    }

    /// Get current audio source
    pub fn get_audio_source(&self) -> AudioSource {
        self.current_source.lock().unwrap().clone()
    }

    /// Get the current device name label for diagnostics
    pub fn get_current_device_name(&self) -> String {
        match self.current_source.lock().unwrap().clone() {
            AudioSource::Microphone => "microphone".to_string(),
            AudioSource::SystemAudio(name) => name,
        }
    }

    /// Get the current buffer size (for testing/debugging)
    pub fn get_system_audio_buffer_size(&self) -> usize {
        self.system_audio_buffer.lock().unwrap().len()
    }

    /// Clear the system audio buffer
    pub fn clear_system_audio_buffer(&self) {
        // Clear by draining all available samples
        let buffer = self.system_audio_buffer.lock().unwrap().clone();
        let to_drain = buffer.len();
        let _ = buffer.drain_n(to_drain);
    }

    /// Diagnostics: total capacity in samples
    pub fn get_system_audio_buffer_capacity(&self) -> usize {
        self.system_audio_buffer.lock().unwrap().capacity()
    }

    /// Diagnostics: total overwritten samples since start
    pub fn get_system_audio_overwritten_count(&self) -> u64 {
        self.system_audio_buffer.lock().unwrap().overwritten_count()
    }

    /// Diagnostics: device sample rate as seen by capture thread
    pub fn get_device_sample_rate(&self) -> u32 {
        if let Some(ref s) = *self.system_audio.lock().unwrap() {
            s.get_device_sample_rate()
        } else {
            0
        }
    }

    /// Diagnostics: effective resample ratio (1.0 = no resample)
    pub fn get_resample_ratio(&self) -> f32 {
        if let Some(ref s) = *self.system_audio.lock().unwrap() {
            s.get_resample_ratio_milli() as f32 / 1000.0
        } else {
            1.0
        }
    }

    /// Diagnostics: is system audio currently capturing
    pub fn is_system_audio_capturing(&self) -> bool {
        if let Some(ref s) = *self.system_audio.lock().unwrap() {
            s.is_capturing()
        } else {
            false
        }
    }

    /// Diagnostics: number of chunks observed below silence threshold
    pub fn get_silent_chunks_count(&self) -> u64 {
        self.silent_chunks.load(Ordering::Relaxed)
    }

    /// Diagnostics: Restart counters and last error
    pub fn record_restart_attempt(&self) {
        self.restart_attempts_total.fetch_add(1, Ordering::Relaxed);
        // Clear last error before a new attempt
        if let Ok(mut e) = self.last_restart_error.lock() {
            *e = None;
        }
        // Track attempt time for rate limiting metrics
        if let Ok(mut times) = self.restart_attempt_times.lock() {
            let now = Instant::now();
            times.push_back(now);
            // Retain only last hour worth of attempts
            let cutoff = now - Duration::from_secs(3600);
            while let Some(&front) = times.front() {
                if front < cutoff {
                    times.pop_front();
                } else {
                    break;
                }
            }
        }
        // Update last attempt timestamp for cooldown computation
        if let Ok(mut t) = self.last_restart_attempt.lock() {
            *t = Some(Instant::now());
        }
    }
    pub fn record_restart_success(&self) {
        self.restart_successes.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut e) = self.last_restart_error.lock() {
            *e = None;
        }
        // On success, reset cooldown marker
        if let Ok(mut t) = self.last_restart_attempt.lock() {
            *t = None;
        }
    }
    pub fn record_restart_failure(&self, err: String) {
        if let Ok(mut e) = self.last_restart_error.lock() {
            *e = Some(err.clone());
        }
        // Append to recent errors log (most recent first, max 10)
        if let Ok(mut q) = self.recent_errors.lock() {
            q.push_front(err);
            while q.len() > 10 {
                q.pop_back();
            }
        }
    }
    pub fn get_restart_attempts_total(&self) -> u64 {
        self.restart_attempts_total.load(Ordering::Relaxed)
    }
    pub fn get_restart_successes(&self) -> u64 {
        self.restart_successes.load(Ordering::Relaxed)
    }
    pub fn get_last_restart_error(&self) -> Option<String> {
        self.last_restart_error.lock().ok().and_then(|e| e.clone())
    }

    /// Number of restart attempts recorded in the last hour
    pub fn get_restart_attempts_last_hour(&self) -> u64 {
        if let Ok(mut times) = self.restart_attempt_times.lock() {
            let now = Instant::now();
            let cutoff = now - Duration::from_secs(3600);
            while let Some(&front) = times.front() {
                if front < cutoff {
                    times.pop_front();
                } else {
                    break;
                }
            }
            times.len() as u64
        } else {
            0
        }
    }

    /// Cooldown remaining (seconds) before next eligible restart attempt, based on 30s policy.
    pub fn get_restart_cooldown_remaining_secs(&self) -> u64 {
        const MIN_RESTART_INTERVAL: Duration = Duration::from_secs(30);
        if let Ok(t) = self.last_restart_attempt.lock() {
            if let Some(last) = *t {
                let elapsed = last.elapsed();
                if elapsed < MIN_RESTART_INTERVAL {
                    return (MIN_RESTART_INTERVAL - elapsed).as_secs();
                }
            }
        }
        0
    }

    /// Recent audio errors (most recent first)
    pub fn get_recent_audio_errors(&self) -> Vec<String> {
        self.recent_errors
            .lock()
            .map(|q| q.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Save the current buffer to a WAV file (for testing/debugging)
    pub fn save_system_audio_buffer_to_wav(&self, filename: &str) -> Result<String, anyhow::Error> {
        use hound::{WavSpec, WavWriter};

        let buffer = self.system_audio_buffer.lock().unwrap().clone();
        let avail = buffer.len();
        if avail == 0 {
            return Err(anyhow::anyhow!("Buffer is empty, nothing to save"));
        }

        // Save to Desktop for easy access
        let desktop_dir = self
            .app_handle
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

        // Drain all current samples for saving (debug behavior)
        let data = buffer.drain_n(avail);
        for sample in data.iter() {
            writer.write_sample(*sample)?;
        }

        writer.finalize()?;

        info!("Saved {} samples to {:?}", avail, filepath);
        Ok(filepath.to_string_lossy().to_string())
    }

    /// Reconfigure the ring buffer capacity in seconds, restarting capture if active.
    pub fn reconfigure_system_audio_buffer(&self, seconds: u32) -> Result<(), anyhow::Error> {
        let new_cap = WHISPER_SAMPLE_RATE * seconds.max(1) as usize;
        let new_buf = SpscRingBuffer::new(new_cap);

        let active_device = match self.current_source.lock().unwrap().clone() {
            AudioSource::SystemAudio(name) if *self.is_open.lock().unwrap() => Some(name),
            _ => None,
        };

        if let Some(dev) = active_device {
            // Restart with new buffer
            self.stop_system_audio()?;
            {
                let mut guard = self.system_audio_buffer.lock().unwrap();
                *guard = new_buf.clone();
            }
            self.start_system_audio(dev)?;
        } else {
            // Just swap buffer for future starts
            let mut guard = self.system_audio_buffer.lock().unwrap();
            *guard = new_buf;
        }
        Ok(())
    }
}
