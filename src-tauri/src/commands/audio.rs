use crate::audio_feedback;
use crate::audio_toolkit::audio::{list_input_devices, list_output_devices};
use crate::managers::audio::{AudioRecordingManager, AudioSource, MicrophoneMode};
use crate::settings::{get_settings, write_settings};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[cfg(target_os = "macos")]
use std::{path::PathBuf, process::Command};
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
pub struct CustomSounds {
    start: bool,
    stop: bool,
}

fn custom_sound_exists(app: &AppHandle, sound_type: &str) -> bool {
    app.path()
        .resolve(
            format!("custom_{}.wav", sound_type),
            tauri::path::BaseDirectory::AppData,
        )
        .map_or(false, |path| path.exists())
}

#[tauri::command]
pub fn check_custom_sounds(app: AppHandle) -> CustomSounds {
    CustomSounds {
        start: custom_sound_exists(&app, "start"),
        stop: custom_sound_exists(&app, "stop"),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioDevice {
    pub index: String,
    pub name: String,
    pub is_default: bool,
}

#[tauri::command]
pub fn update_microphone_mode(app: AppHandle, always_on: bool) -> Result<(), String> {
    // Update settings
    let mut settings = get_settings(&app);
    settings.always_on_microphone = always_on;
    write_settings(&app, settings);

    // Update the audio manager mode
    let rm = app.state::<Arc<AudioRecordingManager>>();
    let new_mode = if always_on {
        MicrophoneMode::AlwaysOn
    } else {
        MicrophoneMode::OnDemand
    };

    rm.update_mode(new_mode)
        .map_err(|e| format!("Failed to update microphone mode: {}", e))
}

#[tauri::command]
pub fn get_microphone_mode(app: AppHandle) -> Result<bool, String> {
    let settings = get_settings(&app);
    Ok(settings.always_on_microphone)
}

#[tauri::command]
pub fn get_available_microphones() -> Result<Vec<AudioDevice>, String> {
    let devices =
        list_input_devices().map_err(|e| format!("Failed to list audio devices: {}", e))?;

    let mut result = vec![AudioDevice {
        index: "default".to_string(),
        name: "Default".to_string(),
        is_default: true,
    }];

    result.extend(devices.into_iter().map(|d| AudioDevice {
        index: d.index,
        name: d.name,
        is_default: false, // The explicit default is handled separately
    }));

    Ok(result)
}

#[tauri::command]
pub fn set_selected_microphone(app: AppHandle, device_name: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.selected_microphone = if device_name == "default" {
        None
    } else {
        Some(device_name)
    };
    write_settings(&app, settings);

    // Update the audio manager to use the new device
    let rm = app.state::<Arc<AudioRecordingManager>>();
    rm.update_selected_device()
        .map_err(|e| format!("Failed to update selected device: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_selected_microphone(app: AppHandle) -> Result<String, String> {
    let settings = get_settings(&app);
    Ok(settings
        .selected_microphone
        .unwrap_or_else(|| "default".to_string()))
}

#[tauri::command]
pub fn get_available_output_devices() -> Result<Vec<AudioDevice>, String> {
    let devices =
        list_output_devices().map_err(|e| format!("Failed to list output devices: {}", e))?;

    let mut result = vec![AudioDevice {
        index: "default".to_string(),
        name: "Default".to_string(),
        is_default: true,
    }];

    result.extend(devices.into_iter().map(|d| AudioDevice {
        index: d.index,
        name: d.name,
        is_default: false, // The explicit default is handled separately
    }));

    Ok(result)
}

#[tauri::command]
pub fn set_selected_output_device(app: AppHandle, device_name: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.selected_output_device = if device_name == "default" {
        None
    } else {
        Some(device_name)
    };
    write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
pub fn get_selected_output_device(app: AppHandle) -> Result<String, String> {
    let settings = get_settings(&app);
    Ok(settings
        .selected_output_device
        .unwrap_or_else(|| "default".to_string()))
}

#[tauri::command]
pub fn play_test_sound(app: AppHandle, sound_type: String) {
    let sound = match sound_type.as_str() {
        "start" => audio_feedback::SoundType::Start,
        "stop" => audio_feedback::SoundType::Stop,
        _ => {
            eprintln!("Unknown sound type: {}", sound_type);
            return;
        }
    };
    audio_feedback::play_test_sound(&app, sound);
}

/* ──────────────────────────────────────────────────────────────── */
/* Audio Source Switching Commands                                    */
/* ──────────────────────────────────────────────────────────────── */

#[tauri::command]
pub fn set_system_audio_source(app: AppHandle, device_name: String) -> Result<(), String> {
    let rm = app.state::<Arc<AudioRecordingManager>>();

    rm.set_audio_source(AudioSource::SystemAudio(device_name))
        .map_err(|e| format!("Failed to set system audio source: {}", e))
}

#[tauri::command]
pub fn set_microphone_source(app: AppHandle) -> Result<(), String> {
    let rm = app.state::<Arc<AudioRecordingManager>>();

    rm.set_audio_source(AudioSource::Microphone)
        .map_err(|e| format!("Failed to set microphone source: {}", e))
}

#[tauri::command]
pub fn get_current_audio_source(app: AppHandle) -> Result<String, String> {
    let rm = app.state::<Arc<AudioRecordingManager>>();

    match rm.get_audio_source() {
        AudioSource::Microphone => Ok("microphone".to_string()),
        AudioSource::SystemAudio(device) => Ok(format!("system:{}", device)),
    }
}

#[tauri::command]
pub fn get_system_audio_buffer_size(app: AppHandle) -> Result<usize, String> {
    let rm = app.state::<Arc<AudioRecordingManager>>();
    Ok(rm.get_system_audio_buffer_size())
}

#[tauri::command]
pub fn save_system_audio_buffer_to_wav(app: AppHandle, filename: String) -> Result<String, String> {
    let rm = app.state::<Arc<AudioRecordingManager>>();
    rm.save_system_audio_buffer_to_wav(&filename)
        .map_err(|e| format!("Failed to save WAV: {}", e))
}

#[tauri::command]
pub fn clear_system_audio_buffer(app: AppHandle) -> Result<(), String> {
    let rm = app.state::<Arc<AudioRecordingManager>>();
    rm.clear_system_audio_buffer();
    Ok(())
}

#[derive(Serialize)]
pub struct AudioMetrics {
    buffer_size_samples: usize,
    buffer_capacity_samples: usize,
    buffer_fill_percent: f32,
    overwritten_samples: u64,
    device_name: String,
    device_sample_rate: u32,
    resample_ratio: f32,
    silent_chunks: u64,
    restart_attempts_total: u64,
    restarts_last_hour: u64,
    restart_cooldown_remaining_secs: u64,
    restart_successes: u64,
    last_restart_error: Option<String>,
    // New diagnostics
    is_system_capturing: bool,
    backlog_seconds_estimate: f32,
    queue_queued: i64,
    queue_processing: i64,
    queue_backlog_seconds: f32,
}

#[derive(Serialize)]
pub struct BackgroundMusicStatus {
    pub supported: bool,
    pub installed: bool,
    pub running: bool,
    pub install_paths: Vec<String>,
}

#[tauri::command]
pub fn get_audio_metrics(app: AppHandle) -> Result<AudioMetrics, String> {
    let rm = app.state::<Arc<AudioRecordingManager>>();
    let size = rm.get_system_audio_buffer_size();
    let cap = rm.get_system_audio_buffer_capacity();
    let percent = if cap > 0 {
        (size as f32 / cap as f32) * 100.0
    } else {
        0.0
    };
    let overwritten = rm.get_system_audio_overwritten_count();
    let dev_rate = rm.get_device_sample_rate();
    let ratio = rm.get_resample_ratio();
    let silent = rm.get_silent_chunks_count();
    let r_attempts = rm.get_restart_attempts_total();
    let r_last_hour = rm.get_restart_attempts_last_hour();
    let cooldown_secs = rm.get_restart_cooldown_remaining_secs();
    let r_success = rm.get_restart_successes();
    let r_error = rm.get_last_restart_error();
    let device_name = rm.get_current_device_name();
    let is_capturing = rm.is_system_audio_capturing();
    let backlog_secs_estimate = size as f32 / 16_000.0; // mono 16kHz
                                                        // Queue metrics
    let (q_queued, q_processing, q_backlog_secs) =
        if let Some(q) = app.try_state::<Arc<crate::queue::Queue>>() {
            match (q.counts(), q.backlog_seconds()) {
                (Ok((a, b, _)), Ok(backlog)) => (a, b, backlog),
                _ => (0, 0, 0.0),
            }
        } else {
            (0, 0, 0.0)
        };

    Ok(AudioMetrics {
        buffer_size_samples: size,
        buffer_capacity_samples: cap,
        buffer_fill_percent: percent,
        overwritten_samples: overwritten,
        device_name,
        device_sample_rate: dev_rate,
        resample_ratio: ratio,
        silent_chunks: silent,
        restart_attempts_total: r_attempts,
        restarts_last_hour: r_last_hour,
        restart_cooldown_remaining_secs: cooldown_secs,
        restart_successes: r_success,
        last_restart_error: r_error,
        is_system_capturing: is_capturing,
        backlog_seconds_estimate: backlog_secs_estimate,
        queue_queued: q_queued,
        queue_processing: q_processing,
        queue_backlog_seconds: q_backlog_secs,
    })
}

#[tauri::command]
pub fn get_audio_errors(app: AppHandle) -> Result<Vec<String>, String> {
    let rm = app.state::<Arc<AudioRecordingManager>>();
    Ok(rm.get_recent_audio_errors())
}

#[tauri::command]
pub fn get_background_music_status() -> BackgroundMusicStatus {
    background_music_status_impl()
}

#[cfg(target_os = "macos")]
fn background_music_status_impl() -> BackgroundMusicStatus {
    let install_paths = background_music_candidate_paths()
        .into_iter()
        .filter(|path| path.exists())
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();

    let installed = !install_paths.is_empty();
    let running = is_background_music_running();

    BackgroundMusicStatus {
        supported: true,
        installed,
        running,
        install_paths,
    }
}

#[cfg(not(target_os = "macos"))]
fn background_music_status_impl() -> BackgroundMusicStatus {
    BackgroundMusicStatus {
        supported: false,
        installed: false,
        running: false,
        install_paths: Vec::new(),
    }
}

#[cfg(target_os = "macos")]
fn background_music_candidate_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("/Applications/Background Music.app"),
        PathBuf::from("/Applications/BackgroundMusic.app"),
    ];

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join("Applications/Background Music.app"));
        paths.push(home.join("Applications/BackgroundMusic.app"));
    }

    paths
}

#[cfg(target_os = "macos")]
fn is_background_music_running() -> bool {
    for cmd in ["/usr/bin/pgrep", "/bin/pgrep", "pgrep"] {
        if let Ok(status) = Command::new(cmd).args(["-f", "Background Music"]).status() {
            if status.success() {
                return true;
            }
        }
    }
    false
}
