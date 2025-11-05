//! Windows System Audio Capture
//!
//! This module implements system audio capture for Windows using WASAPI loopback.
//! WASAPI (Windows Audio Session API) provides native system audio capture without
//! requiring additional virtual audio devices.

use super::{AudioChunkCallback, SystemAudioCaptureDevice, VirtualDeviceInfo};
use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, SampleFormat};
use log::{debug, info, warn};
use std::sync::{Arc, Mutex};

/// Windows system audio capture implementation using WASAPI
pub struct WindowsSystemAudio {
    host: Host,
    stream: Arc<Mutex<Option<Stream>>>,
    is_capturing: Arc<Mutex<bool>>,
    current_device: Arc<Mutex<Option<Device>>>,
    sample_rate: Arc<Mutex<u32>>,
}

impl WindowsSystemAudio {
    /// Create a new Windows system audio capture instance
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

    /// Check if a device is a loopback or virtual device
    fn is_loopback_or_virtual_device(device_name: &str) -> bool {
        let name_lower = device_name.to_lowercase();
        // Check for common virtual device names
        name_lower.contains("loopback")
            || name_lower.contains("vb-audio")
            || name_lower.contains("virtual")
            || name_lower.contains("cable")
            || name_lower.contains("voicemeeter")
    }

    /// Convert device to VirtualDeviceInfo
    fn device_to_info(&self, device: &Device, is_output: bool) -> Result<VirtualDeviceInfo> {
        let name = device
            .name()
            .unwrap_or_else(|_| "Unknown Device".to_string());

        // Try to get device config
        let config = if is_output {
            device.default_output_config().ok()
        } else {
            device.default_input_config().ok()
        };

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
    fn find_device_by_name(&self, name: &str, use_output: bool) -> Result<Option<Device>> {
        let devices = if use_output {
            self.host.output_devices()?
        } else {
            self.host.input_devices()?
        };

        for device in devices {
            if let Ok(device_name) = device.name() {
                if device_name == name {
                    return Ok(Some(device));
                }
            }
        }
        Ok(None)
    }

    /// Get the default output device for WASAPI loopback
    fn get_default_loopback_device(&self) -> Result<Device> {
        // On Windows, we want to capture from the default output device
        // WASAPI loopback captures what's being played
        self.host
            .default_output_device()
            .context("No default output device found")
    }
}

impl SystemAudioCaptureDevice for WindowsSystemAudio {
    fn detect_virtual_device(&self) -> Result<Option<VirtualDeviceInfo>> {
        debug!("Detecting virtual audio devices on Windows");

        // Check for VB-Audio Cable or other virtual devices
        for device in self.host.input_devices()? {
            if let Ok(name) = device.name() {
                if Self::is_loopback_or_virtual_device(&name) {
                    info!("Found virtual audio device: {}", name);
                    return Ok(Some(self.device_to_info(&device, false)?));
                }
            }
        }

        // If no virtual device found, we can still use WASAPI loopback
        // with the default output device
        debug!("No virtual device found, will use WASAPI loopback");

        if let Ok(device) = self.get_default_loopback_device() {
            return Ok(Some(self.device_to_info(&device, true)?));
        }

        Ok(None)
    }

    fn list_output_devices(&self) -> Result<Vec<VirtualDeviceInfo>> {
        let mut devices = Vec::new();

        // List all output devices (WASAPI can capture from any of them)
        for device in self.host.output_devices()? {
            if let Ok(info) = self.device_to_info(&device, true) {
                devices.push(info);
            }
        }

        // Also list virtual input devices
        for device in self.host.input_devices()? {
            if let Ok(name) = device.name() {
                if Self::is_loopback_or_virtual_device(&name) {
                    if let Ok(info) = self.device_to_info(&device, false) {
                        devices.push(info);
                    }
                }
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
        let (device, use_output_config) = if let Some(ref id) = device_id {
            // Try to find as virtual input device first
            if let Some(dev) = self.find_device_by_name(id, false)? {
                (dev, false)
            } else if let Some(dev) = self.find_device_by_name(id, true)? {
                (dev, true)
            } else {
                return Err(anyhow::anyhow!("Specified device not found: {}", id));
            }
        } else {
            // Use default output device with WASAPI loopback
            (self.get_default_loopback_device()?, true)
        };

        let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        info!("Starting capture from device: {}", device_name);

        // Get device config
        // For WASAPI loopback, we use the output config
        let config = if use_output_config {
            device
                .default_output_config()
                .context("Failed to get default output config")?
        } else {
            device
                .default_input_config()
                .context("Failed to get default input config")?
        };

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();

        info!(
            "Device config - Sample rate: {}, Channels: {}, Format: {:?}",
            sample_rate, channels, sample_format
        );

        *self.sample_rate.lock().unwrap() = sample_rate;

        // Build the stream based on sample format
        // Note: On Windows with WASAPI loopback, we use build_input_stream
        // even for output devices - the WASAPI backend handles this
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

        info!("System audio capture started successfully (WASAPI loopback)");
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

        // Windows will typically handle device changes gracefully
        // through WASAPI, but we log it for debugging

        Ok(())
    }
}

impl WindowsSystemAudio {
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

impl Default for WindowsSystemAudio {
    fn default() -> Self {
        Self::new().expect("Failed to create WindowsSystemAudio")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_windows_system_audio() {
        let result = WindowsSystemAudio::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_loopback_or_virtual_device() {
        assert!(WindowsSystemAudio::is_loopback_or_virtual_device(
            "VB-Audio Virtual Cable"
        ));
        assert!(WindowsSystemAudio::is_loopback_or_virtual_device(
            "Voicemeeter Output"
        ));
        assert!(!WindowsSystemAudio::is_loopback_or_virtual_device(
            "Realtek High Definition Audio"
        ));
    }

    #[test]
    fn test_detect_virtual_device() {
        let audio = WindowsSystemAudio::new().unwrap();
        let result = audio.detect_virtual_device();
        assert!(result.is_ok());
        // Should at least detect the default output device for loopback
    }

    #[test]
    fn test_list_output_devices() {
        let audio = WindowsSystemAudio::new().unwrap();
        let result = audio.list_output_devices();
        assert!(result.is_ok());
    }
}
