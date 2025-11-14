//! Linux System Audio Capture
//!
//! This module implements system audio capture for Linux using PulseAudio or PipeWire.
//! It uses monitor sources to capture system audio output.

use super::{AudioChunkCallback, SystemAudioCaptureDevice, VirtualDeviceInfo};
use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat, Stream};
use log::{debug, info, warn};
use std::sync::{Arc, Mutex};

/// Known monitor/virtual device patterns on Linux
const MONITOR_DEVICE_PATTERNS: &[&str] =
    &["monitor", "Monitor", ".monitor", "PulseAudio", "PipeWire"];

/// Linux system audio capture implementation
pub struct LinuxSystemAudio {
    host: Host,
    stream: Arc<Mutex<Option<Stream>>>,
    is_capturing: Arc<Mutex<bool>>,
    current_device: Arc<Mutex<Option<Device>>>,
    sample_rate: Arc<Mutex<u32>>,
}

impl LinuxSystemAudio {
    /// Create a new Linux system audio capture instance
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

    /// Check if a device is a monitor source
    fn is_monitor_device(device_name: &str) -> bool {
        MONITOR_DEVICE_PATTERNS
            .iter()
            .any(|&pattern| device_name.contains(pattern))
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
            (48000, 2) // Default values
        };

        Ok(VirtualDeviceInfo {
            name: name.clone(),
            available: true,
            device_id: name.clone(),
            sample_rate,
            channels,
        })
    }

    /// Find a device by name
    fn find_device_by_name(&self, name: &str) -> Result<Option<Device>> {
        for device in self.host.input_devices()? {
            if let Ok(device_name) = device.name() {
                if device_name == name || device_name.contains(name) {
                    return Ok(Some(device));
                }
            }
        }
        Ok(None)
    }
}

impl SystemAudioCaptureDevice for LinuxSystemAudio {
    fn detect_virtual_device(&self) -> Result<Option<VirtualDeviceInfo>> {
        debug!("Detecting monitor sources on Linux");

        // Look for monitor sources
        for device in self.host.input_devices()? {
            if let Ok(name) = device.name() {
                if Self::is_monitor_device(&name) {
                    info!("Found monitor source: {}", name);
                    return Ok(Some(self.device_to_info(&device)?));
                }
            }
        }

        debug!("No monitor source found");
        Ok(None)
    }

    fn list_output_devices(&self) -> Result<Vec<VirtualDeviceInfo>> {
        let mut devices = Vec::new();

        // List all input devices (monitors appear as input devices)
        for device in self.host.input_devices()? {
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
            // Try to auto-detect monitor source
            if let Some(vd_info) = self.detect_virtual_device()? {
                self.find_device_by_name(&vd_info.name)?
                    .context("Detected monitor source not found")?
            } else {
                return Err(anyhow::anyhow!(
                    "No monitor source found. Please check your PulseAudio/PipeWire configuration."
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

        // PulseAudio/PipeWire typically handle device changes gracefully
        // but we log for debugging

        Ok(())
    }
}

impl LinuxSystemAudio {
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

        // Target chunk size: ~100ms at 16kHz = 1600 samples
        const CHUNK_SIZE: usize = 1600;

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
                while buffer.len() >= CHUNK_SIZE {
                    let chunk: Vec<f32> = buffer.drain(..CHUNK_SIZE).collect();
                    callback(chunk);
                }
            },
            |err| {
                eprintln!("Stream error: {}", err);
            },
            None,
        )?;

        Ok(stream)
    }
}

impl Default for LinuxSystemAudio {
    fn default() -> Self {
        Self::new().expect("Failed to create LinuxSystemAudio")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_linux_system_audio() {
        let result = LinuxSystemAudio::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_monitor_device() {
        assert!(LinuxSystemAudio::is_monitor_device(
            "alsa_output.pci.monitor"
        ));
        assert!(LinuxSystemAudio::is_monitor_device("PulseAudio Monitor"));
        assert!(!LinuxSystemAudio::is_monitor_device("HDA Intel PCH"));
    }

    #[test]
    fn test_detect_virtual_device() {
        let audio = LinuxSystemAudio::new().unwrap();
        let result = audio.detect_virtual_device();
        assert!(result.is_ok());
        // May return None if no monitor source is available
    }

    #[test]
    fn test_list_output_devices() {
        let audio = LinuxSystemAudio::new().unwrap();
        let result = audio.list_output_devices();
        assert!(result.is_ok());
    }
}
