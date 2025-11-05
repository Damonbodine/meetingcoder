//! System Audio Capture Module
//!
//! This module provides cross-platform system audio capture for recording
//! both sides of video conference calls (e.g., Zoom, Google Meet).
//!
//! # Platform Support
//!
//! - **macOS**: Uses Core Audio with virtual audio devices (BlackHole, Loopback)
//! - **Windows**: Uses WASAPI loopback recording
//! - **Linux**: Uses PulseAudio monitor sources / PipeWire
//!
//! # Architecture
//!
//! Each platform implementation provides a `SystemAudioCapture` struct that
//! implements the common `SystemAudioCaptureDevice` trait, allowing uniform
//! usage across all platforms.
//!
//! # Example
//!
//! ```rust,no_run
//! use system_audio::SystemAudioCapture;
//!
//! let mut capture = SystemAudioCapture::new()?;
//! capture.detect_virtual_device()?;
//! capture.start_capture(Box::new(|audio_chunk| {
//!     // Process audio chunk
//! }))?;
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};

// Platform-specific implementations
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

// Sendable wrapper for thread-safe audio capture
mod sendable;

// Re-export platform-specific implementation as SystemAudioCapture
#[cfg(target_os = "macos")]
pub use macos::MacOSSystemAudio as SystemAudioCapture;

#[cfg(target_os = "windows")]
pub use windows::WindowsSystemAudio as SystemAudioCapture;

#[cfg(target_os = "linux")]
pub use linux::LinuxSystemAudio as SystemAudioCapture;

// Export sendable wrapper
pub use sendable::SendableSystemAudio;

/// Information about a virtual audio device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualDeviceInfo {
    /// Device name
    pub name: String,
    /// Whether the device is currently available
    pub available: bool,
    /// Device-specific identifier
    pub device_id: String,
    /// Sample rate of the device
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
}

/// Audio chunk callback type
///
/// Receives audio samples in f32 mono format at 16kHz
pub type AudioChunkCallback = Box<dyn FnMut(Vec<f32>) + Send + 'static>;

/// Common trait for system audio capture across platforms
pub trait SystemAudioCaptureDevice {
    /// Detect if a virtual audio device is installed
    ///
    /// Returns information about available virtual audio devices
    fn detect_virtual_device(&self) -> Result<Option<VirtualDeviceInfo>>;

    /// List all available system audio output devices
    fn list_output_devices(&self) -> Result<Vec<VirtualDeviceInfo>>;

    /// Start capturing system audio
    ///
    /// # Arguments
    /// * `callback` - Function called with audio chunks (mono f32 at 16kHz)
    /// * `device_id` - Optional specific device ID to use
    fn start_capture(
        &mut self,
        callback: AudioChunkCallback,
        device_id: Option<String>,
    ) -> Result<()>;

    /// Stop capturing system audio
    fn stop_capture(&mut self) -> Result<()>;

    /// Check if currently capturing
    fn is_capturing(&self) -> bool;

    /// Get the current sample rate
    fn get_sample_rate(&self) -> u32;

    /// Handle device changes (e.g., device disconnected mid-call)
    fn handle_device_change(&mut self) -> Result<()>;
}

/// Helper function to detect if system audio capture is available on this platform
pub fn is_system_audio_available() -> bool {
    cfg!(any(target_os = "macos", target_os = "windows", target_os = "linux"))
}

/// Get platform-specific setup instructions for virtual audio devices
pub fn get_setup_instructions() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        r#"
macOS System Audio Setup:

To capture both sides of video calls, you need a virtual audio device:

Option 1: BlackHole (Free, Open Source)
1. Download from: https://existential.audio/blackhole/
2. Install BlackHole 2ch
3. Create a Multi-Output Device in Audio MIDI Setup:
   - Open "Audio MIDI Setup" (in Applications/Utilities)
   - Click the "+" button and select "Create Multi-Output Device"
   - Check both "BlackHole 2ch" and your output device (e.g., MacBook Speakers)
   - Right-click the Multi-Output Device and select "Use This Device For Sound Output"
4. In MeetingCoder settings, select "BlackHole 2ch" as the system audio source

Option 2: Loopback by Rogue Amoeba (Paid, More Features)
1. Purchase and install from: https://rogueamoeba.com/loopback/
2. Create a virtual device that captures system audio
3. Set the virtual device as your system output
4. Select it in MeetingCoder settings

Note: You'll need to set your Zoom/Meet audio output to the Multi-Output Device
or Loopback virtual device for this to work.
"#
    }

    #[cfg(target_os = "windows")]
    {
        r#"
Windows System Audio Setup:

Windows supports system audio capture natively through WASAPI loopback.

No additional software required! MeetingCoder can capture your system audio directly.

Optional: VB-Audio Virtual Cable (for advanced routing)
1. Download from: https://vb-audio.com/Cable/
2. Install VB-CABLE Driver
3. Set VB-CABLE Input as your default output device
4. Configure your meeting app to output to VB-CABLE

MeetingCoder will automatically detect and use WASAPI loopback recording.
"#
    }

    #[cfg(target_os = "linux")]
    {
        r#"
Linux System Audio Setup:

Linux supports system audio capture through PulseAudio or PipeWire.

For PulseAudio:
1. MeetingCoder will automatically use the monitor source of your output device
2. No additional setup required

For PipeWire:
1. PipeWire provides monitor sources automatically
2. MeetingCoder will detect and use them

To verify your setup:
- Run: pactl list sources | grep monitor
- You should see monitor sources for your output devices

Advanced: Create a virtual sink
1. Run: pactl load-module module-null-sink sink_name=MeetingCoder
2. Run: pactl load-module module-loopback source=MeetingCoder.monitor
3. Set your meeting app to output to "MeetingCoder"
"#
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "System audio capture is not supported on this platform."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_audio_available() {
        // Should return true on supported platforms
        assert!(is_system_audio_available());
    }

    #[test]
    fn test_setup_instructions_not_empty() {
        let instructions = get_setup_instructions();
        assert!(!instructions.is_empty());
    }
}
