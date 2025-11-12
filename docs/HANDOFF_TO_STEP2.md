# Agent Handoff: Phase 1 Step 2 - MeetingManager Module

**Copy the prompt below and paste it to the next agent/session:**

---

## Context

You're building **MeetingCoder** - a desktop app that transforms stakeholder Zoom/Meet calls into working code in real-time.

**Project Location**: `/Users/damonbodine/speechtotext/Handy/`

**Current Status**: Phase 1, Step 1 COMPLETE (System Audio Capture ✅)
**Your Task**: Phase 1, Step 2 - Build the MeetingManager Module

---

## What's Been Completed (Step 1)

✅ **System Audio Capture** - Fully functional (2025-11-04)
- Audio can be captured from BlackHole (virtual audio device)
- Works with ANY application (Zoom, Meet, Discord, YouTube, Apple Music, etc.)
- 48kHz → 16kHz resampling implemented
- Test UI with recording functionality (`Cmd+Shift+D` → System Audio Testing)
- WAV export working (saves to Desktop)

**Key Files**:
- `src-tauri/src/managers/audio.rs` - AudioRecordingManager with system audio support
- `src-tauri/src/system_audio/sendable.rs` - Thread-safe audio capture wrapper
- `src/components/settings/SystemAudioTest.tsx` - Test UI

**How to Test Current Functionality**:
```bash
cd /Users/damonbodine/speechtotext/Handy
bun run tauri dev
# Press Cmd+Shift+D, go to System Audio Testing, test recording
```

---

## Your Task: Build MeetingManager Module

**Estimated Time**: 2-3 hours
**Goal**: Create a manager that orchestrates meeting lifecycle (start/pause/end) and coordinates between AudioRecordingManager and TranscriptionManager.

### What to Build

#### 1. Create `src-tauri/src/managers/meeting.rs`

**Data Structures**:
```rust
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeetingSession {
    pub id: String,                           // UUID
    pub name: String,                         // User-provided or auto-generated
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub transcript_segments: Vec<TranscriptSegment>,
    pub status: MeetingStatus,
    pub participants: Vec<String>,            // Speaker labels
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MeetingStatus {
    Recording,
    Paused,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub speaker: String,                      // "Speaker 1", "Speaker 2", etc.
    pub start_time: f64,                      // Seconds from meeting start
    pub end_time: f64,
    pub text: String,
    pub confidence: f32,
    pub timestamp: SystemTime,
}

pub struct MeetingManager {
    active_meetings: Arc<Mutex<HashMap<String, MeetingSession>>>,
    app_handle: AppHandle,
}

impl MeetingManager {
    pub fn new() -> Result<Self> {
        // Initialize with empty meetings map
    }

    pub async fn start_meeting(&self, name: String) -> Result<String> {
        // 1. Generate UUID for meeting_id
        // 2. Create MeetingSession with Recording status
        // 3. Store in active_meetings map
        // 4. Return meeting_id
    }

    pub async fn end_meeting(&self, meeting_id: &str) -> Result<MeetingSession> {
        // 1. Get meeting from active_meetings
        // 2. Set status to Completed
        // 3. Set end_time
        // 4. Remove from active_meetings
        // 5. Return the completed session (for saving)
    }

    pub async fn pause_meeting(&self, meeting_id: &str) -> Result<()> {
        // Set status to Paused
    }

    pub async fn resume_meeting(&self, meeting_id: &str) -> Result<()> {
        // Set status back to Recording
    }

    pub async fn add_segment(&self, meeting_id: &str, segment: TranscriptSegment) -> Result<()> {
        // Add segment to meeting's transcript_segments vec
        // Emit event to frontend
    }

    pub async fn get_live_transcript(&self, meeting_id: &str) -> Result<Vec<TranscriptSegment>> {
        // Return current transcript_segments for a meeting
    }

    pub async fn get_active_meetings(&self) -> Result<Vec<MeetingSession>> {
        // Return list of all active meetings
    }

    pub async fn update_speaker_labels(&self, meeting_id: &str, mapping: HashMap<String, String>) -> Result<()> {
        // Update speaker labels (e.g., "Speaker 1" -> "Me", "Speaker 2" -> "Stakeholder")
    }
}
```

#### 2. Add Tauri Commands (`src-tauri/src/commands/meeting.rs`)

Create a new file:
```rust
use crate::managers::meeting::{MeetingManager, TranscriptSegment};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn start_meeting(
    name: String,
    state: State<'_, Arc<MeetingManager>>,
) -> Result<String, String> {
    state.start_meeting(name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn end_meeting(
    meeting_id: String,
    state: State<'_, Arc<MeetingManager>>,
) -> Result<serde_json::Value, String> {
    let session = state.end_meeting(&meeting_id)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(session)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pause_meeting(
    meeting_id: String,
    state: State<'_, Arc<MeetingManager>>,
) -> Result<(), String> {
    state.pause_meeting(&meeting_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_meeting(
    meeting_id: String,
    state: State<'_, Arc<MeetingManager>>,
) -> Result<(), String> {
    state.resume_meeting(&meeting_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_live_transcript(
    meeting_id: String,
    state: State<'_, Arc<MeetingManager>>,
) -> Result<Vec<TranscriptSegment>, String> {
    state.get_live_transcript(&meeting_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_active_meetings(
    state: State<'_, Arc<MeetingManager>>,
) -> Result<Vec<serde_json::Value>, String> {
    let meetings = state.get_active_meetings()
        .await
        .map_err(|e| e.to_string())?;

    meetings.into_iter()
        .map(|m| serde_json::to_value(m).map_err(|e| e.to_string()))
        .collect()
}

#[tauri::command]
pub async fn update_speaker_labels(
    meeting_id: String,
    mapping: HashMap<String, String>,
    state: State<'_, Arc<MeetingManager>>,
) -> Result<(), String> {
    state.update_speaker_labels(&meeting_id, mapping)
        .await
        .map_err(|e| e.to_string())
}
```

#### 3. Register Manager and Commands

**In `src-tauri/src/managers/mod.rs`**:
```rust
pub mod meeting;  // ADD THIS LINE
```

**In `src-tauri/src/commands/mod.rs`**:
```rust
pub mod meeting;  // ADD THIS LINE
```

**In `src-tauri/src/lib.rs`**:

Add to imports:
```rust
use managers::meeting::MeetingManager;
```

In `initialize_core_logic()`:
```rust
let meeting_manager = Arc::new(
    MeetingManager::new().expect("Failed to initialize meeting manager")
);
app_handle.manage(meeting_manager.clone());
```

In `.invoke_handler()`:
```rust
commands::meeting::start_meeting,
commands::meeting::end_meeting,
commands::meeting::pause_meeting,
commands::meeting::resume_meeting,
commands::meeting::get_live_transcript,
commands::meeting::get_active_meetings,
commands::meeting::update_speaker_labels,
```

#### 4. Add Dependencies

**In `src-tauri/Cargo.toml`** (if not already present):
```toml
[dependencies]
uuid = { version = "1.0", features = ["v4", "serde"] }
```

---

## Testing Your Implementation

### Test 1: Basic Meeting Lifecycle
```bash
# Run the app
cd /Users/damonbodine/speechtotext/Handy
bun run tauri dev
```

In the browser console:
```javascript
// Start a meeting
const meetingId = await invoke('start_meeting', { name: 'Test Meeting' });
console.log('Meeting ID:', meetingId);

// Get active meetings
const meetings = await invoke('get_active_meetings');
console.log('Active meetings:', meetings);

// End the meeting
const session = await invoke('end_meeting', { meetingId });
console.log('Completed session:', session);
```

### Test 2: Add Segments
```javascript
// Start meeting
const meetingId = await invoke('start_meeting', { name: 'Test Meeting' });

// Manually add a test segment
await invoke('add_segment', {
    meetingId,
    segment: {
        speaker: 'Speaker 1',
        start_time: 0.0,
        end_time: 3.5,
        text: 'Hello, this is a test.',
        confidence: 0.95,
        timestamp: Date.now()
    }
});

// Get transcript
const transcript = await invoke('get_live_transcript', { meetingId });
console.log('Transcript:', transcript);
```

---

## Success Criteria

When you're done, the following should work:

✅ Can start a meeting and get a valid UUID back
✅ Meeting appears in active_meetings list
✅ Can add transcript segments to a meeting
✅ Can retrieve live transcript for a meeting
✅ Can pause/resume a meeting
✅ Can end a meeting and get the complete session back
✅ Can update speaker labels (e.g., "Speaker 1" → "Me")
✅ Rust code compiles without errors
✅ All Tauri commands are registered and callable from frontend

---

## Key Files to Reference

**Existing Manager Patterns**:
- `src-tauri/src/managers/audio.rs` - Example of manager structure
- `src-tauri/src/managers/history.rs` - Example of storing/retrieving sessions
- `src-tauri/src/managers/transcription.rs` - Example of async operations

**Coding Standards**:
- `src-tauri/agents.md` - Tauri-specific coding standards
- Max 500 lines per Rust file
- Use `Result<T>` for error handling
- Avoid `unwrap()` in production code

**Documentation**:
- `/docs/HANDOFF_PHASE1.md` - Detailed build guide (see "Iteration 1")
- `/docs/prd/01-PHASE1.md` - Requirements
- `/Handy/PHASE1_PROGRESS.md` - Current progress

---

## What NOT to Build Yet

❌ **Don't integrate with AudioRecordingManager yet** - That's Step 3
❌ **Don't integrate with TranscriptionManager yet** - That's Step 3
❌ **Don't build the frontend UI yet** - That's Step 5
❌ **Don't implement transcript storage yet** - That's Step 4

This step is JUST the manager module with basic meeting lifecycle management.

---

## After Completion

When you're done:

1. **Test all commands** via browser console
2. **Verify Rust compilation** with `cargo check`
3. **Document any issues** you encountered
4. **Update progress**: Mark Step 2 as complete in `/Handy/PHASE1_PROGRESS.md`
5. **Prepare for Step 3**: Continuous transcription loop (next step)

---

## Questions to Ask if Stuck

1. **Where is MeetingManager stored?** → In Tauri's managed state (see lib.rs)
2. **How do I emit events?** → `app_handle.emit("event-name", &payload)`
3. **How do other managers work?** → Check `managers/audio.rs` as reference
4. **UUID not found?** → Add `uuid` crate to Cargo.toml dependencies

---

## Quick Start Commands

```bash
# Navigate to project
cd /Users/damonbodine/speechtotext/Handy

# Run dev server
bun run tauri dev

# Check Rust code
cargo check --manifest-path src-tauri/Cargo.toml

# Build for production (when done testing)
cargo build --manifest-path src-tauri/Cargo.toml
```

---

**Ready to start? Begin with creating `src-tauri/src/managers/meeting.rs` and follow the structure above!** 🚀

Good luck! Remember: This step is about meeting lifecycle management only. Audio capture and transcription integration comes in Step 3.
