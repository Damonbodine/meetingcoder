use crate::summarization::agent::SummarizationOutput;
use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// Process-local lock to serialize state file reads/writes
static STATE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Serialize)]
struct JsonlRecord<'a> {
    meeting_id: &'a str,
    meeting_name: &'a str,
    model: &'a str,
    source: &'a str,
    update_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tech_stack: Option<&'a str>,
    #[serde(flatten)]
    update: &'a SummarizationOutput,
}

#[derive(Serialize, Deserialize)]
struct MeetingState {
    last_update_id: u32,
    last_summary_time: Option<String>,
}

fn read_state(path: &Path) -> MeetingState {
    if let Ok(bytes) = fs::read(path) {
        if let Ok(state) = serde_json::from_slice::<MeetingState>(&bytes) {
            return state;
        }
    }
    MeetingState {
        last_update_id: 0,
        last_summary_time: None,
    }
}

fn write_state(path: &Path, state: &MeetingState) -> Result<()> {
    let data = serde_json::to_vec_pretty(state)?;
    fs::create_dir_all(path.parent().unwrap_or(&PathBuf::from(".")))?;
    fs::write(path, data)?;
    Ok(())
}

/// Append a JSONL update to the project's .meeting-updates.jsonl
pub fn append_update(
    project_path: &str,
    meeting_id: &str,
    meeting_name: &str,
    model: &str,
    source: &str,
    update: &SummarizationOutput,
) -> Result<u32> {
    // Use .claude/.meeting-state.json for persistent update_id
    let claude_dir = Path::new(project_path).join(".claude");
    let state_path = claude_dir.join(".meeting-state.json");

    let (update_id, is_first) = {
        let _guard = STATE_LOCK.lock().unwrap();
        let mut state = read_state(&state_path);
        state.last_update_id = state.last_update_id.saturating_add(1);
        state.last_summary_time = Some(chrono::Utc::now().to_rfc3339());
        write_state(&state_path, &state)?;
        (state.last_update_id, state.last_update_id == 1)
    };

    let record = JsonlRecord {
        meeting_id,
        meeting_name,
        model,
        source,
        update_id: format!("u{}", update_id),
        project_name: if is_first { Some(meeting_name) } else { None },
        project_type: None,
        tech_stack: None,
        update,
    };
    let line = serde_json::to_string(&record)? + "\n";

    let updates_path = Path::new(project_path).join(".meeting-updates.jsonl");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&updates_path)?;
    file.write_all(line.as_bytes())?;
    file.flush()?;

    Ok(update_id)
}
