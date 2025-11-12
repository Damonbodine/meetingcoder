//! macOS System Audio Capture
//!
//! This module implements system audio capture for macOS using Core Audio and cpal.
//! It detects and captures from virtual audio devices like BlackHole and Loopback.

use super::{AudioChunkCallback, SystemAudioCaptureDevice, VirtualDeviceInfo};
use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, SampleFormat};
use log::{debug, info, warn};
use std::sync::{Arc, Mutex};

/// Known virtual audio device names on macOS
const VIRTUAL_DEVICE_NAMES: &[&str] = &[
    "BlackHole",
    "BlackHole 2ch",
    "BlackHole 16ch",
    "Loopback Audio",
    "Soundflower",
];

/// macOS system audio capture implementation
pub struct MacOSSystemAudio {
    host: Host,
    stream: Arc<Mutex<Option<Stream>>>,
    is_capturing: Arc<Mutex<bool>>,
    current_device: Arc<Mutex<Option<Device>>>,
    sample_rate: Arc<Mutex<u32>>,
}

impl MacOSSystemAudio {
    /// Create a new macOS system audio capture instance
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();

        Ok(Self {
            host,
            stream: Arc::new(Mutex::new(None)),
            is_capturing: Arc::new(Mutex::new(false)),
            current_device: Arc::new(Mutex::new(None)),
            sample_rate: Arc::new(Mutex::new(16000)),
        })
    }

    /// Check if a device is a known virtual audio device
    fn is_virtual_device(device_name: &str) -> bool {
        VIRTUAL_DEVICE_NAMES
            .iter()
            .any(|&vd_name| device_name.contains(vd_name))
    }

    /// Convert device to VirtualDeviceInfo
    fn device_to_info(&self, device: &Device) -> Result<VirtualDeviceInfo> {
        let name = device
            .name()
            .unwrap_or_else(|_| "Unknown Device".to_string());

        // Try to get device config
        let config = device.default_input_config().ok();

        let (sample_rate, channels) = if let Some(cfg) = config {
            (cfg.sample_rate().0, cfg.channels())
        } else {
            (48000, 2) // Default values if we can't get config
        };

        Ok(VirtualDeviceInfo {
            name: name.clone(),
            available: true,
            device_id: name.clone(), // On macOS, we use name as ID
            sample_rate,
            channels,
        })
    }

    /// Find a device by name
    fn find_device_by_name(&self, name: &str) -> Result<Option<Device>> {
        for device in self.host.input_devices()? {
            if let Ok(device_name) = device.name() {
                if device_name == name {
                    return Ok(Some(device));
                }
            }
        }
        Ok(None)
    }
}

impl SystemAudioCaptureDevice for MacOSSystemAudio {
    fn detect_virtual_device(&self) -> Result<Option<VirtualDeviceInfo>> {
        debug!("Detecting virtual audio devices on macOS");

        for device in self.host.input_devices()? {
            if let Ok(name) = device.name() {
                if Self::is_virtual_device(&name) {
                    info!("Found virtual audio device: {}", name);
                    return Ok(Some(self.device_to_info(&device)?));
                }
            }
        }

        debug!("No virtual audio device found");
        Ok(None)
    }

    fn list_output_devices(&self) -> Result<Vec<VirtualDeviceInfo>> {
        let mut devices = Vec::new();

        // On macOS, we list input devices because virtual audio devices
        // route output to input
        for device in self.host.input_devices()? {
            // Include all devices, not just virtual ones
            if let Ok(info) = self.device_to_info(&device) {
                devices.push(info);
            }
        }

        Ok(devices)
    }

    fn start_capture(
        &mut self,
        mut callback: AudioChunkCallback,
        device_id: Option<String>,
    ) -> Result<()> {
        // Check if already capturing
        if *self.is_capturing.lock().unwrap() {
            warn!("Already capturing system audio");
            return Ok(());
        }

        // Find the device
        let device = if let Some(ref id) = device_id {
            self.find_device_by_name(id)?
                .context("Specified device not found")?
        } else {
            // Try to auto-detect virtual device
            if let Some(vd_info) = self.detect_virtual_device()? {
                self.find_device_by_name(&vd_info.name)?
                    .context("Detected virtual device not found")?
            } else {
                return Err(anyhow::anyhow!(
                    "No virtual audio device found. Please install BlackHole or Loopback."
                ));
            }
        };

        let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        info!("Starting capture from device: {}", device_name);

        // Get device config
        let config = device
            .default_input_config()
            .context("Failed to get default input config")?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();

        info!(
            "Device config - Sample rate: {}, Channels: {}, Format: {:?}",
            sample_rate, channels, sample_format
        );

        *self.sample_rate.lock().unwrap() = sample_rate;

        // Build the stream based on sample format
        let stream = match sample_format {
            SampleFormat::F32 => {
                self.build_stream::<f32>(&device, &config.into(), channels, callback)?
            }
            SampleFormat::I16 => {
                self.build_stream::<i16>(&device, &config.into(), channels, callback)?
            }
            SampleFormat::U16 => {
                self.build_stream::<u16>(&device, &config.into(), channels, callback)?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported sample format: {:?}",
                    sample_format
                ));
            }
        };

        // Start the stream
        stream.play().context("Failed to start audio stream")?;

        // Store the stream and device
        *self.stream.lock().unwrap() = Some(stream);
        *self.current_device.lock().unwrap() = Some(device);
        *self.is_capturing.lock().unwrap() = true;

        info!("System audio capture started successfully");
        Ok(())
    }

    fn stop_capture(&mut self) -> Result<()> {
        if !*self.is_capturing.lock().unwrap() {
            debug!("Not currently capturing");
            return Ok(());
        }

        // Drop the stream (automatically stops it)
        *self.stream.lock().unwrap() = None;
        *self.current_device.lock().unwrap() = None;
        *self.is_capturing.lock().unwrap() = false;

        info!("System audio capture stopped");
        Ok(())
    }

    fn is_capturing(&self) -> bool {
        *self.is_capturing.lock().unwrap()
    }

    fn get_sample_rate(&self) -> u32 {
        *self.sample_rate.lock().unwrap()
    }

    fn handle_device_change(&mut self) -> Result<()> {
        if !self.is_capturing() {
            return Ok(());
        }

        warn!("Handling device change - attempting to restart capture");

        // For now, just log the event
        // In a production system, you'd want to:
        // 1. Detect the specific change
        // 2. Re-enumerate devices
        // 3. Attempt to reconnect to the same or a fallback device

        Ok(())
    }
}

impl MacOSSystemAudio {
    /// Build an input stream with the given sample type
    fn build_stream<T>(
        &self,
        device: &Device,
        config: &cpal::StreamConfig,
        channels: usize,
        mut callback: AudioChunkCallback,
    ) -> Result<Stream>
    where
        T: cpal::Sample + cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        // We'll accumulate samples and send chunks
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let buffer_clone = buffer.clone();

        // Target chunk size: ~100ms at device sample rate
        let chunk_size: usize = (config.sample_rate.0 as usize / 10).max(1);

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let mut buffer = buffer_clone.lock().unwrap();

                // Convert to mono f32
                if channels == 1 {
                    // Already mono, just convert
                    buffer.extend(data.iter().map(|&s| s.to_sample::<f32>()));
                } else {
                    // Convert multi-channel to mono by averaging
                    for frame in data.chunks(channels) {
                        let mono_sample: f32 =
                            frame.iter().map(|&s| s.to_sample::<f32>()).sum::<f32>()
                                / channels as f32;
                        buffer.push(mono_sample);
                    }
                }

                // Send chunks when we have enough samples
                while buffer.len() >= chunk_size {
                    let chunk: Vec<f32> = buffer.drain(..chunk_size).collect();
                    callback(chunk);
                }
            },
            |err| {
                log::error!("Audio stream error: {}. This may cause transcription to stop. Try restarting the meeting.", err);
            },
            None,
        )?;

        Ok(stream)
    }
}

impl Default for MacOSSystemAudio {
    fn default() -> Self {
        Self::new().expect("Failed to create MacOSSystemAudio")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_macos_system_audio() {
        let result = MacOSSystemAudio::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_virtual_device() {
        assert!(MacOSSystemAudio::is_virtual_device("BlackHole 2ch"));
        assert!(MacOSSystemAudio::is_virtual_device("Loopback Audio"));
        assert!(!MacOSSystemAudio::is_virtual_device("MacBook Pro Microphone"));
    }

    #[test]
    fn test_detect_virtual_device() {
        let audio = MacOSSystemAudio::new().unwrap();
        let result = audio.detect_virtual_device();
        assert!(result.is_ok());
        // Note: May return None if no virtual device is installed
    }

    #[test]
    fn test_list_output_devices() {
        let audio = MacOSSystemAudio::new().unwrap();
        let result = audio.list_output_devices();
        assert!(result.is_ok());
        if let Ok(devices) = result {
            // Should have at least some devices on any Mac
            assert!(!devices.is_empty());
        }
    }
}
