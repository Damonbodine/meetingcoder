use crate::document_generation::*;
use crate::managers::meeting::MeetingManager;
use std::sync::Arc;
use tauri::{AppHandle, State};

/// Generate a PRD now for a meeting (manually triggered)
#[tauri::command]
pub async fn generate_prd_now(
    meeting_id: String,
    _app: AppHandle,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<PRDVersion, String> {
    log::info!("Manual PRD generation requested for meeting: {}", meeting_id);

    // Get meeting data
    let meeting_session = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .map_err(|e| format!("Meeting not found: {}", e))?;

    // Get transcript
    let transcript = meeting_session.transcript_segments.clone();

    // Create or load PRD generator
    let mut prd_generator = PRDGenerator::load(meeting_id.clone(), meeting_session.name.clone())
        .unwrap_or_else(|_| PRDGenerator::new(meeting_id.clone(), meeting_session.name.clone()));

    // Generate PRD based on current state
    let version = if prd_generator.get_all_versions().is_empty() {
        // Generate initial PRD
        prd_generator
            .generate_initial_prd(
                &transcript,
                &[], // TODO: Get feature extractions
                None,
            )
            .await
            .map_err(|e| format!("Failed to generate initial PRD: {}", e))?
    } else {
        // Generate incremental update
        prd_generator
            .generate_incremental_update(
                &transcript,
                &[], // TODO: Get new extractions since last version
            )
            .await
            .map_err(|e| format!("Failed to generate PRD update: {}", e))?
    };

    Ok(version)
}

/// Get all PRD versions for a meeting
#[tauri::command]
pub async fn get_prd_versions(meeting_id: String) -> Result<Vec<PRDVersion>, String> {
    get_all_versions(&meeting_id).map_err(|e| format!("Failed to get PRD versions: {}", e))
}

/// Get PRD content for a specific version
#[tauri::command]
pub async fn get_prd_content(meeting_id: String, version: u32) -> Result<String, String> {
    let (_, _, markdown) = load_prd_version(&meeting_id, version)
        .map_err(|e| format!("Failed to load PRD version: {}", e))?;

    Ok(markdown)
}

/// Get PRD content as JSON for a specific version
#[tauri::command]
pub async fn get_prd_content_json(meeting_id: String, version: u32) -> Result<PRDContent, String> {
    let (_, content, _) = load_prd_version(&meeting_id, version)
        .map_err(|e| format!("Failed to load PRD version: {}", e))?;

    Ok(content)
}

/// Get the changelog for a meeting's PRD
#[tauri::command]
pub async fn get_prd_changelog(meeting_id: String) -> Result<PRDChangelog, String> {
    load_changelog(&meeting_id).map_err(|e| format!("Failed to load changelog: {}", e))
}

/// Get a specific change between two versions
#[tauri::command]
pub async fn get_prd_change(
    meeting_id: String,
    from_version: u32,
    to_version: u32,
) -> Result<PRDChange, String> {
    let changelog = load_changelog(&meeting_id)
        .map_err(|e| format!("Failed to load changelog: {}", e))?;

    changelog
        .changes
        .iter()
        .find(|c| c.from_version == from_version && c.to_version == to_version)
        .cloned()
        .ok_or_else(|| format!("Change not found: {} -> {}", from_version, to_version))
}

/// Export PRD to a file (markdown, PDF, HTML)
#[tauri::command]
pub async fn export_prd(
    meeting_id: String,
    version: u32,
    format: String,
) -> Result<String, String> {
    let (version_data, _content, _markdown) = load_prd_version(&meeting_id, version)
        .map_err(|e| format!("Failed to load PRD version: {}", e))?;

    match format.as_str() {
        "markdown" => {
            // Already have markdown, just return the path
            Ok(version_data.file_path)
        }
        "pdf" => {
            // TODO: Implement PDF export
            Err("PDF export not yet implemented".to_string())
        }
        "html" => {
            // TODO: Implement HTML export
            Err("HTML export not yet implemented".to_string())
        }
        _ => Err(format!("Unsupported export format: {}", format)),
    }
}

/// Get PRD metadata for a meeting
#[tauri::command]
pub async fn get_prd_metadata(meeting_id: String) -> Result<Option<PRDMetadata>, String> {
    load_metadata(&meeting_id).map_err(|e| format!("Failed to load metadata: {}", e))
}

/// Delete a PRD version
#[tauri::command]
pub async fn delete_prd_version(_meeting_id: String, _version: u32) -> Result<(), String> {
    // TODO: Implement deletion
    Err("PRD deletion not yet implemented".to_string())
}
