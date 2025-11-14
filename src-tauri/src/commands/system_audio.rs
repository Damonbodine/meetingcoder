//! Tauri commands for system audio capture functionality

use crate::system_audio::{
    get_setup_instructions, is_system_audio_available, SystemAudioCapture,
    SystemAudioCaptureDevice, VirtualDeviceInfo,
};
use serde::{Deserialize, Serialize};

/// Response for device detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectDeviceResponse {
    pub available: bool,
    pub device: Option<VirtualDeviceInfo>,
}

/// Check if system audio capture is available on this platform
#[tauri::command]
pub fn is_system_audio_supported() -> bool {
    is_system_audio_available()
}

/// Get setup instructions for the current platform
#[tauri::command]
pub fn get_system_audio_setup_instructions() -> String {
    get_setup_instructions().to_string()
}

/// Detect if a virtual audio device is installed
#[tauri::command]
pub fn detect_virtual_audio_device() -> Result<DetectDeviceResponse, String> {
    let capture = SystemAudioCapture::new().map_err(|e| e.to_string())?;

    match capture.detect_virtual_device() {
        Ok(Some(device)) => Ok(DetectDeviceResponse {
            available: true,
            device: Some(device),
        }),
        Ok(None) => Ok(DetectDeviceResponse {
            available: false,
            device: None,
        }),
        Err(e) => Err(format!("Failed to detect virtual device: {}", e)),
    }
}

/// List all available system audio output devices
#[tauri::command]
pub fn list_system_audio_devices() -> Result<Vec<VirtualDeviceInfo>, String> {
    let capture = SystemAudioCapture::new().map_err(|e| e.to_string())?;

    capture
        .list_output_devices()
        .map_err(|e| format!("Failed to list devices: {}", e))
}
