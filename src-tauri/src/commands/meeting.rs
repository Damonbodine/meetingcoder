use crate::managers::meeting::{MeetingManager, MeetingSummary, TranscriptSegment};
use crate::storage::transcript::TranscriptStorage;
use chrono::{DateTime, Local, TimeZone};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

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
    let secs = if start_time > 1_000_000_000_000 { // > ~2001-09-09 in ms
        start_time / 1000
    } else {
        start_time
    };

    // Format date like TranscriptStorage (local time)
    let dt: DateTime<Local> = Local.timestamp_opt(secs, 0).single().ok_or_else(|| "Invalid timestamp".to_string())?;
    let date_str = dt.format("%Y-%m-%d").to_string();

    // Sanitize like TranscriptStorage::generate_meeting_dir_name
    let mut sanitized: String = meeting_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect();
    sanitized = sanitized.replace(' ', "-").to_lowercase();
    if sanitized.is_empty() { sanitized = "untitled".to_string(); }
    if sanitized.len() > 100 { sanitized.truncate(100); }
    sanitized = sanitized.replace("..", "");

    let dir_name = format!("{}_{}", date_str, sanitized);
    let base = TranscriptStorage::default_path().map_err(|e| e.to_string())?;
    Ok(base.join(dir_name).to_string_lossy().to_string())
}
