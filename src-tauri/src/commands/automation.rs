use crate::managers::meeting::MeetingManager;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn trigger_meeting_command_now(
    app: AppHandle,
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<bool, String> {
    let meeting = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .map_err(|e| format!("{}", e))?;
    let Some(path) = meeting.project_path.clone() else {
        return Ok(false);
    };
    if path.is_empty() {
        return Ok(false);
    }
    crate::automation::claude_trigger::trigger_meeting_update(&app, &path, &meeting_id, 0)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_meeting_terminal(
    app: AppHandle,
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<(), String> {
    let meeting = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .map_err(|e| format!("{}", e))?;
    let Some(path) = meeting.project_path.clone() else {
        return Ok(());
    };
    if path.is_empty() {
        return Ok(());
    }
    crate::automation::claude_trigger::open_project_in_terminal(&app, &path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_meeting_vscode(
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<(), String> {
    let meeting = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .map_err(|e| format!("{}", e))?;
    let Some(path) = meeting.project_path.clone() else {
        return Ok(());
    };
    if path.is_empty() {
        return Ok(());
    }
    crate::automation::claude_trigger::open_project_in_vscode(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_meeting_cursor(
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<(), String> {
    let meeting = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .map_err(|e| format!("{}", e))?;
    let Some(path) = meeting.project_path.clone() else {
        return Ok(());
    };
    if path.is_empty() {
        return Ok(());
    }
    crate::automation::claude_trigger::open_project_in_cursor(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_meeting_vscode_with_meeting(
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<(), String> {
    let meeting = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .map_err(|e| format!("{}", e))?;
    let Some(path) = meeting.project_path.clone() else {
        return Ok(());
    };
    if path.is_empty() {
        return Ok(());
    }
    // Ensure the correct project window is opened/focused first, then run the meeting command
    // Open project first; then run meeting in the correct workspace
    let _ = crate::automation::claude_trigger::open_project_in_vscode(&path);
    crate::automation::claude_trigger::open_vscode_with_meeting(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_meeting_cursor_with_meeting(
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<(), String> {
    let meeting = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .map_err(|e| format!("{}", e))?;
    let Some(path) = meeting.project_path.clone() else {
        return Ok(());
    };
    if path.is_empty() {
        return Ok(());
    }
    // Ensure the correct project window is opened/focused first, then run the meeting command
    // Open project first; then run meeting in the correct workspace
    let _ = crate::automation::claude_trigger::open_project_in_cursor(&path);
    crate::automation::claude_trigger::open_cursor_with_meeting(&path).map_err(|e| e.to_string())
}
