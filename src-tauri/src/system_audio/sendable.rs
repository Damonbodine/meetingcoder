//! Sendable wrapper for system audio capture
//!
//! This module provides a Send-able interface for system audio capture
//! by managing the audio stream in a dedicated thread and communicating
//! via channels.

use super::{
    ring_buffer::SpscRingBuffer, SystemAudioCapture, SystemAudioCaptureDevice, VirtualDeviceInfo,
};
use anyhow::Result;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

const TARGET_SAMPLE_RATE: usize = 16000; // Whisper sample rate
                                         // Hard cap the buffered audio to prevent unbounded memory growth.
                                         // 180 seconds @ 16kHz mono float32 ~= 11.5 MB.
const MAX_BUFFER_SECONDS: usize = 180;
const MAX_BUFFER_SAMPLES: usize = TARGET_SAMPLE_RATE * MAX_BUFFER_SECONDS;

enum ControlMessage {
    Start {
        device_id: Option<String>,
        buffer: Arc<SpscRingBuffer>,
    },
    Stop,
    Shutdown,
}

/// A Send-able wrapper for system audio capture that manages
/// the audio stream in a dedicated thread
pub struct SendableSystemAudio {
    control_tx: Sender<ControlMessage>,
    _thread: JoinHandle<()>,
    is_capturing: Arc<Mutex<bool>>,
    device_sample_rate: Arc<AtomicU32>,
    resample_ratio_milli: Arc<AtomicU32>,
}

impl SendableSystemAudio {
    /// Create a new sendable system audio capture
    pub fn new() -> Result<Self> {
        let (control_tx, control_rx) = channel::<ControlMessage>();
        let is_capturing = Arc::new(Mutex::new(false));
        let is_capturing_clone = is_capturing.clone();
        let device_sample_rate = Arc::new(AtomicU32::new(0));
        let device_sample_rate_clone = device_sample_rate.clone();
        let resample_ratio_milli = Arc::new(AtomicU32::new(1000));
        let resample_ratio_milli_clone = resample_ratio_milli.clone();

        // Spawn a thread to handle audio capture
        let thread = thread::spawn(move || {
            let mut capture: Option<SystemAudioCapture> = None;

            while let Ok(msg) = control_rx.recv() {
                match msg {
                    ControlMessage::Start { device_id, buffer } => {
                        // Create capture if not exists
                        if capture.is_none() {
                            match SystemAudioCapture::new() {
                                Ok(c) => capture = Some(c),
                                Err(e) => {
                                    eprintln!("Failed to create system audio capture: {}", e);
                                    continue;
                                }
                            }
                        }

                        // Detect actual device sample rate BEFORE starting capture
                        let device_sample_rate: usize = if let Some(ref cap) = capture {
                            // If a specific device was requested, try to locate it and read its config
                            if let Some(ref id) = device_id {
                                match cap.list_output_devices() {
                                    Ok(devices) => devices
                                        .into_iter()
                                        .find(|d| d.name == *id)
                                        .map(|d| d.sample_rate as usize)
                                        .unwrap_or(48000),
                                    Err(_) => 48000,
                                }
                            } else {
                                // Try to auto-detect the virtual device we will use
                                match cap.detect_virtual_device() {
                                    Ok(Some(info)) => info.sample_rate as usize,
                                    _ => 48000,
                                }
                            }
                        } else {
                            48000
                        };

                        println!(
                            "Device sample rate (pre-start): {} Hz, target: {} Hz",
                            device_sample_rate, TARGET_SAMPLE_RATE
                        );
                        device_sample_rate_clone
                            .store(device_sample_rate as u32, Ordering::Release);

                        // Create resampler if needed (wrapped in Arc<Mutex> for callback)
                        let needs_resampling = device_sample_rate != TARGET_SAMPLE_RATE;
                        let resampler: Arc<Mutex<Option<SincFixedIn<f32>>>> = if needs_resampling {
                            let params = SincInterpolationParameters {
                                // Tuned lower for reduced CPU usage (slightly lower quality, acceptable for ASR)
                                sinc_len: 128,
                                f_cutoff: 0.95,
                                interpolation: SincInterpolationType::Linear,
                                oversampling_factor: 128,
                                window: WindowFunction::BlackmanHarris2,
                            };

                            match SincFixedIn::<f32>::new(
                                TARGET_SAMPLE_RATE as f64 / device_sample_rate as f64,
                                2.0,
                                params,
                                device_sample_rate,
                                1, // mono
                            ) {
                                Ok(r) => {
                                    let ratio =
                                        TARGET_SAMPLE_RATE as f64 / device_sample_rate as f64;
                                    println!(
                                        "Resampler created: {} Hz -> {} Hz (ratio {:.6})",
                                        device_sample_rate, TARGET_SAMPLE_RATE, ratio
                                    );
                                    resample_ratio_milli_clone
                                        .store((ratio * 1000.0) as u32, Ordering::Release);
                                    Arc::new(Mutex::new(Some(r)))
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Failed to create resampler: {}, audio may be corrupted",
                                        e
                                    );
                                    Arc::new(Mutex::new(None))
                                }
                            }
                        } else {
                            Arc::new(Mutex::new(None))
                        };

                        // Set up callback with resampling. Buffer input to meet rubato's required frame size.
                        let mut in_accumulator: Vec<f32> = Vec::new();
                        let callback = Box::new(move |chunk: Vec<f32>| {
                            let mut out_to_push: Vec<f32> = Vec::new();

                            let mut resampler_guard = resampler.lock().unwrap();
                            if let Some(ref mut r) = *resampler_guard {
                                // Accumulate input until we have at least input_frames_next()
                                in_accumulator.extend_from_slice(&chunk);

                                loop {
                                    let needed = r.input_frames_next();
                                    let have = in_accumulator.len();
                                    if have < needed {
                                        break;
                                    }

                                    // Take exactly 'needed' frames for processing
                                    let input_chunk: Vec<f32> =
                                        in_accumulator.drain(..needed).collect();

                                    match r.process(&[input_chunk], None) {
                                        Ok(output) => {
                                            // Mono -> take channel 0
                                            if let Some(ch0) = output.get(0) {
                                                out_to_push.extend_from_slice(ch0);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Resampling error: {} (needed {}, have {})",
                                                e, needed, have
                                            );
                                            break;
                                        }
                                    }
                                }
                            } else {
                                // No resampling needed; forward the chunk directly
                                out_to_push = chunk;
                            }

                            if !out_to_push.is_empty() {
                                buffer.push(&out_to_push);
                            }
                        });

                        // Start capturing
                        if let Some(ref mut cap) = capture {
                            if let Err(e) = cap.start_capture(callback, device_id) {
                                eprintln!("Failed to start capture: {}", e);
                            } else {
                                *is_capturing_clone.lock().unwrap() = true;
                            }
                        }
                    }
                    ControlMessage::Stop => {
                        if let Some(ref mut cap) = capture {
                            let _ = cap.stop_capture();
                            *is_capturing_clone.lock().unwrap() = false;
                        }
                    }
                    ControlMessage::Shutdown => {
                        if let Some(ref mut cap) = capture {
                            let _ = cap.stop_capture();
                        }
                        *is_capturing_clone.lock().unwrap() = false;
                        break;
                    }
                }
            }
        });

        Ok(Self {
            control_tx,
            _thread: thread,
            is_capturing,
            device_sample_rate,
            resample_ratio_milli,
        })
    }

    /// Start capturing system audio
    pub fn start_capture(
        &self,
        device_id: Option<String>,
        buffer: Arc<SpscRingBuffer>,
    ) -> Result<()> {
        self.control_tx
            .send(ControlMessage::Start { device_id, buffer })
            .map_err(|e| anyhow::anyhow!("Failed to send start message: {}", e))?;
        Ok(())
    }

    /// Stop capturing system audio
    pub fn stop_capture(&self) -> Result<()> {
        self.control_tx
            .send(ControlMessage::Stop)
            .map_err(|e| anyhow::anyhow!("Failed to send stop message: {}", e))?;
        Ok(())
    }

    /// Check if currently capturing
    pub fn is_capturing(&self) -> bool {
        *self.is_capturing.lock().unwrap()
    }

    /// Detect virtual audio device
    pub fn detect_virtual_device(&self) -> Result<Option<VirtualDeviceInfo>> {
        // Create a temporary capture instance for detection
        let capture = SystemAudioCapture::new()?;
        capture.detect_virtual_device()
    }

    /// List available devices
    pub fn list_output_devices(&self) -> Result<Vec<VirtualDeviceInfo>> {
        // Create a temporary capture instance for listing
        let capture = SystemAudioCapture::new()?;
        capture.list_output_devices()
    }

    pub fn get_device_sample_rate(&self) -> u32 {
        self.device_sample_rate.load(Ordering::Acquire)
    }

    /// Resample ratio in milli-units (1000 = 1.0x)
    pub fn get_resample_ratio_milli(&self) -> u32 {
        self.resample_ratio_milli.load(Ordering::Acquire)
    }
}

impl Drop for SendableSystemAudio {
    fn drop(&mut self) {
        let _ = self.control_tx.send(ControlMessage::Shutdown);
    }
}

// SendableSystemAudio is Send because all its fields are Send
unsafe impl Send for SendableSystemAudio {}
unsafe impl Sync for SendableSystemAudio {}
