use crate::managers::meeting::{MeetingManager, MeetingStatus, MeetingSummary, TranscriptSegment};
use crate::storage::transcript::{TranscriptMetadata, TranscriptStorage};
use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

/// Meeting history entry for the History UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingHistoryEntry {
    pub dir_name: String,
    pub dir_path: String,
    pub metadata: TranscriptMetadata,
}

#[tauri::command]
pub async fn start_meeting(
    meeting_name: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<String, String> {
    meeting_manager
        .start_meeting(meeting_name)
        .await
        .map_err(|e| format!("Failed to start meeting: {}", e))
}

#[tauri::command]
pub async fn end_meeting(
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<MeetingSummary, String> {
    meeting_manager
        .end_meeting(&meeting_id)
        .await
        .map_err(|e| format!("Failed to end meeting: {}", e))
}

#[tauri::command]
pub async fn pause_meeting(
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<(), String> {
    meeting_manager
        .pause_meeting(&meeting_id)
        .await
        .map_err(|e| format!("Failed to pause meeting: {}", e))
}

#[tauri::command]
pub async fn resume_meeting(
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<(), String> {
    meeting_manager
        .resume_meeting(&meeting_id)
        .await
        .map_err(|e| format!("Failed to resume meeting: {}", e))
}

#[tauri::command]
pub async fn get_live_transcript(
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<Vec<TranscriptSegment>, String> {
    meeting_manager
        .get_live_transcript(&meeting_id)
        .await
        .map_err(|e| format!("Failed to get transcript: {}", e))
}

#[tauri::command]
pub async fn update_speaker_labels(
    meeting_id: String,
    mapping: HashMap<String, String>,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<(), String> {
    meeting_manager
        .update_speaker_labels(&meeting_id, mapping)
        .await
        .map_err(|e| format!("Failed to update speaker labels: {}", e))
}

#[tauri::command]
pub async fn get_active_meetings(
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<Vec<String>, String> {
    Ok(meeting_manager.get_active_meetings().await)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingInfo {
    pub id: String,
    pub name: String,
    pub status: String,
}

#[tauri::command]
pub async fn get_meeting_info(
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<MeetingInfo, String> {
    let m = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .map_err(|e| format!("Failed to get meeting: {}", e))?;
    let status = match m.status {
        MeetingStatus::Recording => "recording",
        MeetingStatus::Paused => "paused",
        MeetingStatus::Completed => "completed",
    };
    Ok(MeetingInfo {
        id: m.id,
        name: m.name,
        status: status.to_string(),
    })
}

#[tauri::command]
pub async fn get_meeting_project_path(
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<Option<String>, String> {
    match meeting_manager.get_meeting(&meeting_id).await {
        Ok(meeting) => Ok(meeting.project_path.clone()),
        Err(e) => Err(format!("Failed to get meeting: {}", e)),
    }
}

/// Compute the transcript directory path for a given meeting name and start time.
/// start_time expects a Unix timestamp in seconds or milliseconds.
#[tauri::command]
pub fn get_transcript_dir_for(meeting_name: String, start_time: i64) -> Result<String, String> {
    // Determine if the timestamp is in ms or s
    let secs = if start_time > 1_000_000_000_000 {
        // > ~2001-09-09 in ms
        start_time / 1000
    } else {
        start_time
    };

    // Format date like TranscriptStorage (local time)
    let dt: DateTime<Local> = Local
        .timestamp_opt(secs, 0)
        .single()
        .ok_or_else(|| "Invalid timestamp".to_string())?;
    let date_str = dt.format("%Y-%m-%d").to_string();

    // Sanitize like TranscriptStorage::generate_meeting_dir_name
    let mut sanitized: String = meeting_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect();
    sanitized = sanitized.replace(' ', "-").to_lowercase();
    if sanitized.is_empty() {
        sanitized = "untitled".to_string();
    }
    if sanitized.len() > 100 {
        sanitized.truncate(100);
    }
    sanitized = sanitized.replace("..", "");

    let dir_name = format!("{}_{}", date_str, sanitized);
    let base = TranscriptStorage::default_path().map_err(|e| e.to_string())?;
    Ok(base.join(dir_name).to_string_lossy().to_string())
}

/// List all saved meeting transcripts
#[tauri::command]
pub fn list_saved_meetings() -> Result<Vec<MeetingHistoryEntry>, String> {
    let storage = TranscriptStorage::with_default_path().map_err(|e| e.to_string())?;
    let meeting_dirs = storage.list_meetings().map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    let base_path = TranscriptStorage::default_path().map_err(|e| e.to_string())?;

    for dir_name in meeting_dirs {
        match storage.load_transcript(&dir_name) {
            Ok((metadata, _transcript)) => {
                let dir_path = base_path.join(&dir_name);
                entries.push(MeetingHistoryEntry {
                    dir_name,
                    dir_path: dir_path.to_string_lossy().to_string(),
                    metadata,
                });
            }
            Err(e) => {
                log::warn!("Failed to load meeting {}: {}", dir_name, e);
                // Skip corrupted meetings
                continue;
            }
        }
    }

    Ok(entries)
}

/// Open a meeting folder in the file manager
#[tauri::command]
pub fn open_meeting_folder(dir_path: String) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&dir_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&dir_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&dir_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(())
}

/// Delete a saved meeting transcript
#[tauri::command]
pub fn delete_saved_meeting(dir_name: String) -> Result<(), String> {
    let storage = TranscriptStorage::with_default_path().map_err(|e| e.to_string())?;
    storage
        .delete_transcript(&dir_name)
        .map_err(|e| e.to_string())
}
