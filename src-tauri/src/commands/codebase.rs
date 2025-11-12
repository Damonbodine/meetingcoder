use crate::codebase::{analyze_codebase, save_manifest_to_state, CodebaseManifest};
use std::path::PathBuf;

/// Analyzes a codebase and returns a manifest
#[tauri::command]
pub async fn analyze_project_codebase(project_path: String) -> Result<CodebaseManifest, String> {
    let path = PathBuf::from(project_path);

    analyze_codebase(&path)
        .await
        .map_err(|e| format!("Failed to analyze codebase: {}", e))
}

/// Analyzes a codebase and saves the manifest to .meeting-state.json
#[tauri::command]
pub async fn analyze_and_save_codebase(project_path: String) -> Result<CodebaseManifest, String> {
    let path = PathBuf::from(project_path);

    let manifest = analyze_codebase(&path)
        .await
        .map_err(|e| format!("Failed to analyze codebase: {}", e))?;

    save_manifest_to_state(&path, &manifest)
        .await
        .map_err(|e| format!("Failed to save manifest: {}", e))?;

    Ok(manifest)
}
