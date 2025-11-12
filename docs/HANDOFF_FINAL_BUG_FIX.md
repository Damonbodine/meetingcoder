# Agent Handoff: Final Bug Fix - Model Loading

**Project**: MeetingCoder - Meeting transcription feature
**Location**: `/Users/damonbodine/speechtotext/Handy/`
**Status**: Phase 1 is 95% complete, ONE bug to fix
**Estimated Time**: 30 minutes

---

## Current Status

### ✅ What's Working

**All infrastructure is complete:**
1. ✅ System audio capture from BlackHole
2. ✅ MeetingManager with lifecycle management
3. ✅ Continuous transcription loop (30-second chunks)
4. ✅ Transcript storage to disk
5. ✅ Full React UI with real-time updates

**Meeting flow works:**
- User starts meeting → Meeting created ✅
- Audio captured from BlackHole → 30 seconds buffered (480,000 samples) ✅
- Transcription loop runs → Audio sent to TranscriptionManager ✅
- TranscriptionManager receives audio ✅

### ❌ What's Broken

**One error prevents transcription:**
```
[2025-11-05T01:32:27Z ERROR handy_app_lib::managers::meeting] Transcription error: Model is not loaded for transcription.
```

**Root Cause**: 
The Whisper model must be loaded into memory before transcription, but it's not being loaded when the meeting starts.

---

## The Problem

### Error Location
File: `src-tauri/src/managers/transcription.rs`
Line: ~333 (in `transcribe()` method)

```rust
pub fn transcribe(&self, audio: Vec<f32>) -> Result<String> {
    // ...
    
    let engine_guard = self.engine.lock().unwrap();
    if engine_guard.is_none() {
        return Err(anyhow::anyhow!("Model is not loaded for transcription."));
    }
    
    // Transcription code...
}
```

### Why It Fails

**Current Flow:**
1. User clicks "Start Meeting"
2. `MeetingManager::start_meeting()` creates meeting
3. Transcription loop spawns and starts running
4. After 30 seconds, loop calls `transcription_manager.transcribe(audio)`
5. `transcribe()` checks if model is loaded
6. Model is NOT loaded → **Error**

**Missing Step**: No one is calling `transcription_manager.initiate_model_load()` before transcription

---

## The Solution

### Add Model Loading on Meeting Start

**File to modify**: `src-tauri/src/managers/meeting.rs`
**Function**: `start_meeting()` (around line 114)

### Code Change Needed

```rust
pub async fn start_meeting(&self, name: String) -> Result<String> {
    let meeting_id = Uuid::new_v4().to_string();
    let meeting = MeetingSession {
        id: meeting_id.clone(),
        name: name.clone(),
        start_time: SystemTime::now(),
        end_time: None,
        transcript_segments: Vec::new(),
        status: MeetingStatus::Recording,
        participants: Vec::new(),
    };

    // Insert meeting into active meetings
    {
        let mut meetings = self.active_meetings.lock().await;
        meetings.insert(meeting_id.clone(), meeting);
    }

    log::info!("Started meeting: {} (ID: {})", name, meeting_id);

    // ⭐ ADD THIS: Load the transcription model before starting
    log::info!("Loading transcription model...");
    self.transcription_manager.initiate_model_load();
    
    // Optional: Wait for model to load (recommended)
    // This ensures first transcription succeeds
    let transcription_manager = self.transcription_manager.clone();
    tokio::spawn(async move {
        // Wait up to 30 seconds for model to load
        let mut waited = 0;
        while !transcription_manager.is_model_loaded() && waited < 30 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            waited += 1;
        }
        
        if transcription_manager.is_model_loaded() {
            log::info!("Transcription model loaded successfully");
        } else {
            log::error!("Transcription model failed to load within 30 seconds");
        }
    });

    // Spawn transcription loop task
    let task_handle = tokio::spawn(Self::transcription_loop(
        meeting_id.clone(),
        self.active_meetings.clone(),
        self.audio_manager.clone(),
        self.transcription_manager.clone(),
        self.app_handle.clone(),
    ));

    // Store task handle for cleanup
    {
        let mut handles = self.task_handles.lock().await;
        handles.insert(meeting_id.clone(), task_handle);
    }

    log::info!("Transcription task spawned for meeting: {}", meeting_id);

    Ok(meeting_id)
}
```

### Key Changes

1. **Call `initiate_model_load()`** - Triggers async model loading
2. **Optional waiting task** - Ensures model is loaded before first transcription
3. **Log messages** - Helps debug model loading status

---

## Testing After Fix

### 1. Rebuild and Start App
```bash
cd /Users/damonbodine/speechtotext/Handy
cargo build --manifest-path src-tauri/Cargo.toml
bun run tauri dev
```

### 2. Test Meeting Flow
1. Navigate to "Meetings" section
2. Enter meeting name: "Test Meeting"
3. Click "Start Meeting"
4. Check logs - should see:
   ```
   Started meeting: Test Meeting (ID: xxx)
   Loading transcription model...
   Transcription model loaded successfully
   Starting transcription loop for meeting: xxx
   ```

### 3. Play Audio
1. Open YouTube: https://www.youtube.com/watch?v=dQw4w9WgXcQ
2. Wait 30 seconds
3. **Check logs** - should see:
   ```
   Processing audio chunk with 480000 samples
   [Whisper transcription output]
   Added segment 0 to meeting: xxx
   ```

### 4. Verify UI Updates
- First transcript segment should appear after ~30 seconds
- Segments should continue appearing every 30 seconds
- UI should auto-scroll to show new segments

### 5. End Meeting
1. Click "End Meeting"
2. See summary toast with duration and segment count
3. Check `~/MeetingCoder/meetings/` for saved transcript

---

## Expected Results

### Logs Should Show
```
[INFO] Started meeting: Test Meeting (ID: abc-123)
[INFO] Loading transcription model...
[INFO] Transcription model loaded successfully
[INFO] Transcription task spawned for meeting: abc-123
[INFO] Starting transcription loop for meeting: abc-123
[DEBUG] Processing audio chunk with 480000 samples
[INFO] Added segment 0 to meeting: abc-123
[DEBUG] Processing audio chunk with 480000 samples
[INFO] Added segment 1 to meeting: abc-123
...
[INFO] Ended meeting: Test Meeting - Duration: 120s, Segments: 4
[INFO] Transcript saved for meeting: Test Meeting
```

### No More Errors
The error `"Model is not loaded for transcription."` should be **gone**.

---

## Alternative Solutions (If Issues Persist)

### If Model Loading is Slow

**Option A**: Add UI feedback
```typescript
// In MeetingView.tsx
toast.info("Loading transcription model...", { duration: 5000 });
```

**Option B**: Check model loaded before starting
```rust
// In start_meeting(), before spawning loop
if !self.transcription_manager.is_model_loaded() {
    self.transcription_manager.initiate_model_load();
    // Wait synchronously
    tokio::time::sleep(Duration::from_secs(5)).await;
}
```

### If Model Load Fails

Check settings:
```rust
// Model must be downloaded
let settings = get_settings(&self.app_handle);
let model_id = settings.selected_model; // e.g., "parakeet-tdt-0.6b-v3"
```

Ensure model exists in:
`~/.handy/models/[model_id]/`

---

## Reference Files

### Key Files
- **Fix location**: `src-tauri/src/managers/meeting.rs:114` (start_meeting function)
- **TranscriptionManager API**: `src-tauri/src/managers/transcription.rs`
  - `initiate_model_load()` - line 281
  - `is_model_loaded()` - line 130
  - `transcribe()` - line 305

### Related Files
- `src-tauri/src/managers/audio.rs` - Audio capture (working)
- `src-tauri/src/storage/transcript.rs` - Transcript storage (working)
- `src/components/meeting/MeetingView.tsx` - Frontend UI (working)

---

## Success Criteria

When complete, you should be able to:
1. ✅ Start a meeting without errors
2. ✅ See "Transcription model loaded successfully" in logs
3. ✅ Play audio and see first segment after 30 seconds
4. ✅ See new segments every 30 seconds
5. ✅ End meeting and see summary
6. ✅ Find saved transcript in `~/MeetingCoder/meetings/`

---

## Quick Start Commands

```bash
# Navigate to project
cd /Users/damonbodine/speechtotext/Handy

# Make the fix in meeting.rs
# (Add initiate_model_load() call)

# Rebuild
cargo build --manifest-path src-tauri/Cargo.toml

# Run app
bun run tauri dev

# Test with browser console
# Navigate to Meetings → Start meeting → Play audio
```

---

**Time Estimate**: 30 minutes
- Code change: 5 minutes
- Build/compile: 5 minutes
- Testing: 15 minutes
- Verification: 5 minutes

**After this fix, Phase 1 will be 100% complete!** 🎉
