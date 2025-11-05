use crate::settings;
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::AppHandle;

static LAST_TRIGGERS: Lazy<Mutex<HashMap<String, (u32, Instant)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Serialize, Deserialize, Default, Clone)]
struct AutomationState {
    last_trigger_update_id: u32,
    last_trigger_time: Option<String>,
}

fn read_automation_state(project_path: &str) -> AutomationState {
    let path = Path::new(project_path).join(".claude/.automation-state.json");
    if let Ok(bytes) = fs::read(path) {
        if let Ok(state) = serde_json::from_slice::<AutomationState>(&bytes) {
            return state;
        }
    }
    AutomationState::default()
}

fn write_automation_state(project_path: &str, state: &AutomationState) -> Result<()> {
    let p = Path::new(project_path).join(".claude/.automation-state.json");
    if let Some(parent) = p.parent() { fs::create_dir_all(parent)?; }
    fs::write(p, serde_json::to_vec_pretty(state)?)?;
    Ok(())
}

fn validate_project_path(path: &str) -> Result<String> {
    // Security: Validate and canonicalize path to prevent command injection
    use std::path::Path;

    // Reject empty paths
    if path.is_empty() {
        return Err(anyhow!("Project path cannot be empty"));
    }

    // Canonicalize the path (resolves symlinks and validates existence)
    let path_obj = Path::new(path);
    let canonical = path_obj.canonicalize()
        .map_err(|_| anyhow!("Invalid project path: {}", path))?;

    let path_str = canonical.to_str()
        .ok_or_else(|| anyhow!("Path contains invalid UTF-8 characters"))?;

    // Security: Reject paths containing characters that could break shell escaping
    // Even with proper escaping, we defensively reject suspicious paths
    let dangerous_chars = ['"', '\'', '`', '$', '\\', '\n', '\r', ';', '&', '|', '<', '>'];
    if path_str.chars().any(|c| dangerous_chars.contains(&c)) {
        return Err(anyhow!("Path contains unsafe characters: {}", path_str));
    }

    Ok(path_str.to_string())
}

fn escape_path_for_applescript(path: &str) -> String {
    // POSIX path quoting for AppleScript shell commands
    // Use single quotes which are safer than double quotes for shell commands
    let mut s = String::from("'");
    s.push_str(&path.replace("'", "'\\''"));
    s.push('\'');
    s
}

fn run_osascript(script: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()?;
        if !output.status.success() {
            return Err(anyhow!("osascript failed: {}",
                String::from_utf8_lossy(&output.stderr)));
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(anyhow!("AppleScript automation only supported on macOS"))
    }
}

pub fn trigger_meeting_update(
    app: &AppHandle,
    project_path: &str,
    meeting_id: &str,
    update_id: u32,
) -> Result<bool> {
    let settings = settings::get_settings(app);
    if !settings.auto_trigger_meeting_command {
        return Ok(false);
    }

    // Manual trigger: update_id == 0 forces execution (bypass checks)
    let is_forced = update_id == 0;

    // Effective debounce interval
    let min_interval = settings
        .auto_trigger_min_interval_seconds
        .clamp(30, 600) as u64;

    // Merge in-memory tracker with persisted automation state
    let mut map = LAST_TRIGGERS.lock().unwrap();
    let now = Instant::now();
    let mut persisted = read_automation_state(project_path);

    if !is_forced {
        // Check newness vs persisted
        if persisted.last_trigger_update_id >= update_id {
            log::info!("AUTOMATION skip: no new update (last={}, got={})", persisted.last_trigger_update_id, update_id);
            return Ok(false);
        }
        // Check debounce vs in-memory
        if let Some((_, last_time)) = map.get(meeting_id) {
            if now.duration_since(*last_time) < Duration::from_secs(min_interval) {
                log::info!("AUTOMATION skip: debounce active ({}s)", min_interval);
                return Ok(false);
            }
        }
    }

    // Security: Validate and sanitize project path to prevent command injection
    let validated_path = validate_project_path(project_path)?;
    let escaped = escape_path_for_applescript(&validated_path);

    // Prepare AppleScript for Terminal
    // Prefer reusing the selected tab if a window exists, else open new.
    let script_terminal = format!(
        r#"tell application "Terminal"
activate
if (count of windows) > 0 then
    do script "cd {} && export PATH=\"$PWD/.claude/bin:$PATH\"; meeting" in selected tab of front window
else
    do script "cd {} && export PATH=\"$PWD/.claude/bin:$PATH\"; meeting"
end if
end tell"#,
        escaped, escaped
    );

    let mut used_app = "Terminal";
    if let Err(e) = run_osascript(&script_terminal) {
        log::warn!("AUTOMATION Terminal script failed: {}", e);
        // Try iTerm as fallback
        let script_iterm = format!(
            r#"tell application "iTerm"
activate
try
    set newWindow to (create window with default profile)
on error
    set newWindow to current window
end try
tell current session of newWindow
    write text "cd {} && export PATH=\"$PWD/.claude/bin:$PATH\"; meeting"
end tell
end tell"#,
            escaped
        );
        run_osascript(&script_iterm)?;
        used_app = "iTerm";
    }

    // Optional auto-accept via System Events
    if settings.auto_accept_changes {
        let script_accept = format!(
            r#"tell application "{}" to activate
delay 0.6
tell application "System Events"
    keystroke "y"
    delay 0.2
    key code 36 -- Return
end tell"#,
            used_app
        );
        match run_osascript(&script_accept) {
            Ok(_) => log::info!("AUTOMATION auto-accept sent ({}).", used_app),
            Err(e) => log::warn!("AUTOMATION auto-accept failed: {}", e),
        }
    }

    // Update in-memory and persisted state
    map.insert(meeting_id.to_string(), (update_id, now));
    if !is_forced {
        persisted.last_trigger_update_id = update_id;
    }
    persisted.last_trigger_time = Some(chrono::Utc::now().to_rfc3339());
    if let Err(e) = write_automation_state(project_path, &persisted) {
        log::warn!("AUTOMATION state write failed: {}", e);
    }

    log::info!(
        "AUTOMATION sent /meeting (app={}, forced={}, update_id={})",
        used_app,
        is_forced,
        update_id
    );
    Ok(true)
}

pub fn open_project_in_terminal(_app: &AppHandle, project_path: &str) -> Result<()> {
    // Security: Validate and sanitize project path to prevent command injection
    let validated_path = validate_project_path(project_path)?;
    let escaped = escape_path_for_applescript(&validated_path);

    // Open Terminal or iTerm in the project directory without running /meeting
    let script = format!(
        r#"tell application "Terminal"
activate
do script "cd {} && export PATH=\"$PWD/.claude/bin:$PATH\""
end tell"#,
        escaped
    );
    if let Err(e) = run_osascript(&script) {
        log::warn!("AUTOMATION Terminal open failed: {}", e);
        let script_iterm = format!(
            r#"tell application "iTerm"
activate
try
    set newWindow to (create window with default profile)
on error
    set newWindow to current window
end try
tell current session of newWindow
    write text "cd {} && export PATH=\"$PWD/.claude/bin:$PATH\""
end tell
end tell"#,
            escaped
        );
        run_osascript(&script_iterm)?;
    }
    Ok(())
}

pub fn open_project_in_vscode(project_path: &str) -> Result<()> {
    // Reuse path validation for safety
    let validated_path = validate_project_path(project_path)?;

    // macOS: prefer open -a "Visual Studio Code" <path>
    #[cfg(target_os = "macos")]
    {
        let out = Command::new("open")
            .arg("-a")
            .arg("Visual Studio Code")
            .arg(&validated_path)
            .output()?;
        if out.status.success() {
            return Ok(());
        }
        // Fallback to CLI if available
        if let Ok(out2) = Command::new("code").arg(&validated_path).output() {
            if out2.status.success() { return Ok(()); }
        }
        return Err(anyhow!("Failed to open VS Code: {}", String::from_utf8_lossy(&out.stderr)));
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Try VS Code CLI on other platforms
        let out = Command::new("code").arg(&validated_path).output();
        match out {
            Ok(o) if o.status.success() => Ok(()),
            Ok(o) => Err(anyhow!("Failed to open VS Code: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => Err(anyhow!("Failed to run code: {}", e)),
        }
    }
}

pub fn open_project_in_cursor(project_path: &str) -> Result<()> {
    let validated_path = validate_project_path(project_path)?;

    #[cfg(target_os = "macos")]
    {
        // Try Cursor via open -a "Cursor"
        if let Ok(out) = Command::new("open").arg("-a").arg("Cursor").arg(&validated_path).output() {
            if out.status.success() { return Ok(()); }
        }
        // Fallback to possible CLI names
        if let Ok(out2) = Command::new("cursor").arg(&validated_path).output() {
            if out2.status.success() { return Ok(()); }
        }
        Err(anyhow!("Failed to open Cursor editor"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Non-macOS: attempt a 'cursor' CLI if present
        if let Ok(out2) = Command::new("cursor").arg(&validated_path).output() {
            if out2.status.success() { return Ok(()); }
        }
        Err(anyhow!("Cursor opening not implemented on this OS"))
    }
}

pub fn open_vscode_with_meeting(project_path: &str) -> Result<()> {
    let validated_path = validate_project_path(project_path)?;
    let escaped = escape_path_for_applescript(&validated_path);

    #[cfg(target_os = "macos")]
    {
        // Use menu navigation to open an integrated terminal and run the /meeting command
        let script = format!(
            r#"tell application "Visual Studio Code" to activate
delay 0.4
tell application "System Events"
  tell process "Visual Studio Code"
    try
      click menu item "New Terminal" of menu 1 of menu bar item "Terminal" of menu bar 1
    on error
      keystroke "`" using {{command down}}
    end try
  end tell
  delay 0.4
  keystroke "cd {} && export PATH=\"$PWD/.claude/bin:$PATH\"; meeting"
  key code 36 -- Return
end tell"#,
            escaped
        );
        run_osascript(&script)
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Non-macOS: best-effort using CLI to open; user can run /meeting manually
        open_project_in_vscode(project_path)
    }
}

pub fn open_cursor_with_meeting(project_path: &str) -> Result<()> {
    let validated_path = validate_project_path(project_path)?;
    let escaped = escape_path_for_applescript(&validated_path);

    #[cfg(target_os = "macos")]
    {
        // Use menu to open integrated terminal then run the /meeting command
        let script = format!(
            r#"tell application "Cursor" to activate
delay 0.4
tell application "System Events"
  tell process "Cursor"
    try
      click menu item "New Terminal" of menu 1 of menu bar item "Terminal" of menu bar 1
    on error
      keystroke "`" using {{command down}}
    end try
  end tell
  delay 0.4
  keystroke "cd {} && export PATH=\"$PWD/.claude/bin:$PATH\"; meeting"
  key code 36 -- Return
end tell"#,
            escaped
        );
        run_osascript(&script)
    }
    #[cfg(not(target_os = "macos"))]
    {
        open_project_in_cursor(project_path)
    }
}
