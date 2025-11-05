use anyhow::Result;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use tauri::Manager; // for app.path()

pub struct ProjectInitializer {
    base_path: PathBuf,
}

impl ProjectInitializer {
    pub fn new(base_path: PathBuf) -> Result<Self> {
        if !base_path.exists() {
            fs::create_dir_all(&base_path)?;
            log::info!("Created project base directory: {}", base_path.display());
        }
        Ok(Self { base_path })
    }

    pub fn with_default_path() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let base = home.join("MeetingCoder").join("projects");
        Self::new(base)
    }

    fn sanitize_name(name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == ' ' { c } else { ' ' })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-")
            .to_lowercase()
    }

    pub fn init_for_meeting(&self, meeting_name: &str) -> Result<String> {
        let dir_name = Self::sanitize_name(meeting_name);
        let project_dir = self.base_path.join(&dir_name);
        let claude_dir = project_dir.join(".claude");
        let commands_dir = claude_dir.join("commands");

        // Create directories
        fs::create_dir_all(&commands_dir)?;

        // Seed .meeting-updates.jsonl
        let updates_path = project_dir.join(".meeting-updates.jsonl");
        if !updates_path.exists() {
            OpenOptions::new().create(true).write(true).open(&updates_path)?;
        }

        // Seed .claude/.meeting-state.json
        let state_path = claude_dir.join(".meeting-state.json");
        if !state_path.exists() {
            let state = serde_json::json!({
                "last_update_id": 0u32,
                "last_summary_time": null,
            });
            fs::write(&state_path, serde_json::to_string_pretty(&state)?)?;
        }

        // Copy command template to .claude/commands/meeting.md (dev fallback)
        let dev_template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates").join("meeting_command.md");
        if dev_template_path.exists() {
            let content = fs::read_to_string(&dev_template_path)?;
            fs::write(commands_dir.join("meeting.md"), content)?;
        } else {
            log::warn!("Dev template not found at {}", dev_template_path.display());
        }

        // Optional README
        let readme_path = project_dir.join("README.md");
        if !readme_path.exists() {
            let mut readme = String::new();
            readme.push_str("# Meeting Project\n\n");
            readme.push_str("This folder contains meeting updates and commands.\n\n");
            readme.push_str("- Updates: .meeting-updates.jsonl (JSONL)\n");
            readme.push_str("- Commands: .claude/commands/meeting.md\n");
            fs::write(&readme_path, readme)?;
        }

        // Initialize git repo (best-effort)
        let git_dir = project_dir.join(".git");
        if !git_dir.exists() {
            match std::process::Command::new("git")
                .arg("init")
                .current_dir(&project_dir)
                .output()
            {
                Ok(out) => {
                    if !out.status.success() {
                        log::warn!(
                            "git init failed (status {}): {}",
                            out.status,
                            String::from_utf8_lossy(&out.stderr)
                        );
                    } else {
                        log::info!("Initialized git repository at {}", project_dir.display());
                    }
                }
                Err(e) => log::warn!("Failed to run git init: {}", e),
            }
        }

        Ok(project_dir.to_string_lossy().to_string())
    }

    /// Same as init_for_meeting, but attempts to load the template from bundled resources.
    pub fn init_for_meeting_with_app(&self, meeting_name: &str, app: &tauri::AppHandle) -> Result<String> {
        let path = self.init_for_meeting(meeting_name)?;

        let project_dir = PathBuf::from(&path);
        let claude_dir = project_dir.join(".claude");
        let commands_dir = claude_dir.join("commands");

        // Try to resolve bundled resource first
        let resource_path = app
            .path()
            .resolve("templates/meeting_command.md", tauri::path::BaseDirectory::Resource);

        match resource_path {
            Ok(p) => {
                match fs::read_to_string(&p) {
                    Ok(content) => {
                        if let Err(e) = fs::write(commands_dir.join("meeting.md"), content) {
                            log::warn!("Failed writing meeting.md from resource: {}", e);
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed reading template resource {}: {}", p.display(), e);
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to resolve template resource: {}", e);
            }
        }

        Ok(path)
    }

    /// Seed meeting scaffolding inside an existing directory (e.g., a cloned repo root)
    /// Creates `.transcript.jsonl`, `.claude/commands/meeting.md`, and `.claude/.meeting-state.json`
    pub fn seed_in_existing_dir_with_app(dir: &PathBuf, app: &tauri::AppHandle) -> Result<()> {
        if !dir.exists() {
            return Err(anyhow::anyhow!("Target directory does not exist: {}", dir.display()));
        }

        let claude_dir = dir.join(".claude");
        let commands_dir = claude_dir.join("commands");
        std::fs::create_dir_all(&commands_dir)?;

        // Seed transcript jsonl (append-only)
        let updates_path = dir.join(".transcript.jsonl");
        if !updates_path.exists() {
            OpenOptions::new().create(true).write(true).open(&updates_path)?;
        }

        // Seed .claude/.meeting-state.json if missing
        let state_path = claude_dir.join(".meeting-state.json");
        if !state_path.exists() {
            let state = serde_json::json!({
                "last_update_id": 0u32,
                "last_summary_time": null,
            });
            std::fs::write(&state_path, serde_json::to_string_pretty(&state)?)?;
        }

        // Try to resolve bundled command template first, else dev template fallback
        let resource_path = app
            .path()
            .resolve("templates/meeting_command.md", tauri::path::BaseDirectory::Resource);
        match resource_path {
            Ok(p) => {
                if let Ok(content) = std::fs::read_to_string(&p) {
                    let _ = std::fs::write(commands_dir.join("meeting.md"), content);
                }
            }
            Err(_) => {
                let dev_template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates").join("meeting_command.md");
                if dev_template_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&dev_template_path) {
                        let _ = std::fs::write(commands_dir.join("meeting.md"), content);
                    }
                }
            }
        }

        // Seed .claude/bin/meeting shim so `meeting` is a valid command in repo context
        let bin_dir = claude_dir.join("bin");
        std::fs::create_dir_all(&bin_dir)?;
        let meeting_bin = bin_dir.join("meeting");
        if !meeting_bin.exists() {
            let script = r#"#!/usr/bin/env bash
set -e
echo "[Handy] Meeting command invoked in $(pwd)"
if [ -f .claude/commands/meeting.md ]; then
  echo "- See .claude/commands/meeting.md for current prompts."
fi
# TODO: Future: dispatch to Handy via local IPC to apply planned actions.
exit 0
"#;
            std::fs::write(&meeting_bin, script)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&meeting_bin)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&meeting_bin, perms)?;
            }
        }

        Ok(())
    }
}
