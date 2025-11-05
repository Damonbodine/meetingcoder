use crate::managers::meeting::{MeetingManager, MeetingSummary, TranscriptSegment};
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
