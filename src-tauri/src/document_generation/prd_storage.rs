use super::types::*;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Get the PRD directory for a meeting
pub fn get_prd_directory(meeting_id: &str) -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("Failed to get home directory")?;
    let prd_dir = home_dir
        .join(".handy")
        .join("meetings")
        .join(meeting_id)
        .join("prds");

    // Create directory if it doesn't exist
    fs::create_dir_all(&prd_dir)
        .with_context(|| format!("Failed to create PRD directory: {:?}", prd_dir))?;

    Ok(prd_dir)
}

/// Save PRD version to disk
pub fn save_prd_version(
    meeting_id: &str,
    version: &PRDVersion,
    content_md: &str,
    content_json: &PRDContent,
) -> Result<String> {
    let prd_dir = get_prd_directory(meeting_id)?;

    // Determine filename based on version type
    let filename_base = match version.version_type.as_str() {
        "initial" => format!("v{}_initial", version.version),
        "incremental" => format!("v{}_incremental", version.version),
        "milestone" => format!("v{}_milestone", version.version),
        "final" => "final".to_string(),
        _ => format!("v{}", version.version),
    };

    // Save markdown
    let md_path = prd_dir.join(format!("{}.md", filename_base));
    fs::write(&md_path, content_md)
        .with_context(|| format!("Failed to write markdown to {:?}", md_path))?;

    // Save JSON
    let json_path = prd_dir.join(format!("{}.json", filename_base));
    let json_str = serde_json::to_string_pretty(content_json)
        .context("Failed to serialize PRD content to JSON")?;
    fs::write(&json_path, json_str)
        .with_context(|| format!("Failed to write JSON to {:?}", json_path))?;

    // Save version metadata
    let version_path = prd_dir.join(format!("{}_version.json", filename_base));
    let version_str =
        serde_json::to_string_pretty(version).context("Failed to serialize version metadata")?;
    fs::write(&version_path, version_str)
        .with_context(|| format!("Failed to write version metadata to {:?}", version_path))?;

    Ok(md_path.to_string_lossy().to_string())
}

/// Load PRD version from disk
pub fn load_prd_version(
    meeting_id: &str,
    version: u32,
) -> Result<(PRDVersion, PRDContent, String)> {
    let prd_dir = get_prd_directory(meeting_id)?;

    // Find the version file
    let version_files = find_version_files(&prd_dir, version)?;

    // Load version metadata
    let version_data: PRDVersion =
        serde_json::from_str(&fs::read_to_string(&version_files.version)?)
            .context("Failed to parse version metadata")?;

    // Load JSON content
    let content: PRDContent = serde_json::from_str(&fs::read_to_string(&version_files.json)?)
        .context("Failed to parse PRD content")?;

    // Load markdown
    let markdown =
        fs::read_to_string(&version_files.markdown).context("Failed to read markdown file")?;

    Ok((version_data, content, markdown))
}

/// Get all PRD versions for a meeting
pub fn get_all_versions(meeting_id: &str) -> Result<Vec<PRDVersion>> {
    let prd_dir = get_prd_directory(meeting_id)?;

    if !prd_dir.exists() {
        return Ok(Vec::new());
    }

    let mut versions = Vec::new();

    // Read all _version.json files
    for entry in fs::read_dir(&prd_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            let filename = path.file_name().unwrap().to_string_lossy();
            if filename.ends_with("_version.json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(version) = serde_json::from_str::<PRDVersion>(&content) {
                    versions.push(version);
                }
            }
        }
    }

    // Sort by version number
    versions.sort_by_key(|v| v.version);

    Ok(versions)
}

/// Save changelog
pub fn save_changelog(meeting_id: &str, changelog: &PRDChangelog) -> Result<()> {
    let prd_dir = get_prd_directory(meeting_id)?;
    let changelog_path = prd_dir.join("changelog.json");

    let json_str =
        serde_json::to_string_pretty(changelog).context("Failed to serialize changelog")?;

    fs::write(&changelog_path, json_str)
        .with_context(|| format!("Failed to write changelog to {:?}", changelog_path))?;

    Ok(())
}

/// Load changelog
pub fn load_changelog(meeting_id: &str) -> Result<PRDChangelog> {
    let prd_dir = get_prd_directory(meeting_id)?;
    let changelog_path = prd_dir.join("changelog.json");

    if !changelog_path.exists() {
        return Ok(PRDChangelog::default());
    }

    let content = fs::read_to_string(&changelog_path)
        .with_context(|| format!("Failed to read changelog from {:?}", changelog_path))?;

    let changelog: PRDChangelog =
        serde_json::from_str(&content).context("Failed to parse changelog JSON")?;

    Ok(changelog)
}

/// Save PRD metadata
pub fn save_metadata(meeting_id: &str, metadata: &PRDMetadata) -> Result<()> {
    let prd_dir = get_prd_directory(meeting_id)?;
    let metadata_path = prd_dir.join("metadata.json");

    let json_str =
        serde_json::to_string_pretty(metadata).context("Failed to serialize metadata")?;

    fs::write(&metadata_path, json_str)
        .with_context(|| format!("Failed to write metadata to {:?}", metadata_path))?;

    Ok(())
}

/// Load PRD metadata
pub fn load_metadata(meeting_id: &str) -> Result<Option<PRDMetadata>> {
    let prd_dir = get_prd_directory(meeting_id)?;
    let metadata_path = prd_dir.join("metadata.json");

    if !metadata_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&metadata_path)
        .with_context(|| format!("Failed to read metadata from {:?}", metadata_path))?;

    let metadata: PRDMetadata =
        serde_json::from_str(&content).context("Failed to parse metadata JSON")?;

    Ok(Some(metadata))
}

/// Update metadata after saving a new version
pub fn update_metadata(meeting_id: &str, meeting_name: &str, new_version: u32) -> Result<()> {
    let existing = load_metadata(meeting_id)?;

    let metadata = match existing {
        Some(mut meta) => {
            meta.total_versions = new_version;
            meta.latest_version = new_version;
            meta.last_updated_at = chrono::Utc::now().to_rfc3339();
            meta
        }
        None => PRDMetadata {
            meeting_id: meeting_id.to_string(),
            meeting_name: meeting_name.to_string(),
            total_versions: new_version,
            latest_version: new_version,
            first_generated_at: chrono::Utc::now().to_rfc3339(),
            last_updated_at: chrono::Utc::now().to_rfc3339(),
        },
    };

    save_metadata(meeting_id, &metadata)
}

// Helper structures

struct VersionFiles {
    version: PathBuf,
    json: PathBuf,
    markdown: PathBuf,
}

fn find_version_files(prd_dir: &Path, version: u32) -> Result<VersionFiles> {
    // Try different filename patterns
    let patterns = vec![
        format!("v{}_initial", version),
        format!("v{}_incremental", version),
        format!("v{}_milestone", version),
        format!("v{}", version),
        if version == get_latest_version_number(prd_dir).unwrap_or(0) {
            "final".to_string()
        } else {
            format!("v{}", version)
        },
    ];

    for pattern in patterns {
        let version_path = prd_dir.join(format!("{}_version.json", pattern));
        let json_path = prd_dir.join(format!("{}.json", pattern));
        let md_path = prd_dir.join(format!("{}.md", pattern));

        if version_path.exists() && json_path.exists() && md_path.exists() {
            return Ok(VersionFiles {
                version: version_path,
                json: json_path,
                markdown: md_path,
            });
        }
    }

    anyhow::bail!("Version {} not found for meeting", version)
}

fn get_latest_version_number(prd_dir: &Path) -> Result<u32> {
    let mut max_version = 0;

    for entry in fs::read_dir(prd_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            let filename = path.file_name().unwrap().to_string_lossy();
            if filename.ends_with("_version.json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(version) = serde_json::from_str::<PRDVersion>(&content) {
                    max_version = max_version.max(version.version);
                }
            }
        }
    }

    Ok(max_version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_prd_directory() {
        let result = get_prd_directory("test-meeting-123");
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains(".handy"));
        assert!(path.to_string_lossy().contains("meetings"));
        assert!(path.to_string_lossy().contains("test-meeting-123"));
        assert!(path.to_string_lossy().contains("prds"));
    }
}
