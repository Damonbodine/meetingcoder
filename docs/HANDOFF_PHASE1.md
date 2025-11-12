# MeetingCoder Phase 1 Build - Agent Handoff

## Project Context

You are building **MeetingCoder**, a desktop application that transforms stakeholder meetings into working code in real-time. The app captures audio from Zoom/Google Meet calls, transcribes conversations with speaker identification, and feeds structured requirements to Claude Code CLI, which generates MVP code as the meeting progresses.

### Key Innovation

Instead of implementing custom code generation, MeetingCoder leverages **Claude Code CLI** (you!) through a novel integration pattern:
1. Meeting transcription → Summarization agent → Structured requirements
2. Write requirements to `.meeting-updates.jsonl` file
3. Custom `/meeting` slash command in Claude Code reads updates and generates code
4. User (or automation) triggers `/meeting` periodically during the meeting

## Current State

### What Exists
- **Base**: Open-source Handy app (Tauri + Rust + React/TypeScript)
  - Location: `/Users/damonbodine/speechtotext/Handy/`
  - Already has: Whisper/Parakeet transcription, VAD, audio recording (microphone only)
  - Managers: `AudioRecordingManager`, `TranscriptionManager`, `ModelManager`

### What We've Completed (Documentation Phase)
- ✅ Updated PRD Phase 2 with Claude Code integration approach
- ✅ Updated PRD Phase 3 with automation strategy
- ✅ Created `/meeting` slash command template at `Handy/src-tauri/templates/meeting_command.md`
- ✅ Updated coding standards in `Handy/src-tauri/agents.md` (Tauri-specific)

### Key Documents to Reference
1. **PRD Overview**: `/Users/damonbodine/speechtotext/docs/prd/00-OVERVIEW.md`
2. **Phase 1 PRD**: `/Users/damonbodine/speechtotext/docs/prd/01-PHASE1.md`
3. **Phase 2 PRD (Updated)**: `/Users/damonbodine/speechtotext/docs/prd/02-PHASE2.md`
4. **Technical Architecture**: `/Users/damonbodine/speechtotext/docs/prd/TECHNICAL_ARCHITECTURE.md`
5. **API Specs**: `/Users/damonbodine/speechtotext/docs/prd/API_SPECIFICATIONS.md`
6. **Coding Standards**: `/Users/damonbodine/speechtotext/Handy/src-tauri/agents.md`
7. **Meeting Command Template**: `/Users/damonbodine/speechtotext/Handy/src-tauri/templates/meeting_command.md`

## Phase 1 Scope: Audio Capture & Continuous Transcription

### Success Criteria
- [ ] macOS system audio capture works (both sides of Zoom/Meet call)
- [ ] Continuous transcription (not push-to-talk) with 30-60s chunks
- [ ] Speaker diarization identifies 2-3 participants
- [ ] Transcripts saved in JSON + Markdown formats
- [ ] Meeting UI for start/stop/status
- [ ] No audio dropouts during 60+ minute calls
- [ ] Transcription lag < 5 seconds

### Platform Focus
- **macOS only** for Phase 1 (Windows/Linux in future phases)
- Requires BlackHole virtual audio device for system audio capture

## Recommended Build Order

### 1. MeetingManager Module (START HERE)
Create `src-tauri/src/managers/meeting.rs`

**Purpose**: Orchestrate meeting lifecycle and coordinate between audio/transcription managers

**Key Responsibilities**:
- Start/pause/end meeting sessions
- Store meeting metadata (name, participants, start time)
- Accumulate transcript segments during meeting
- Coordinate AudioRecordingManager and TranscriptionManager
- Trigger transcript saves

**Data Structures**:
```rust
pub struct MeetingSession {
    pub id: String,                           // UUID
    pub name: String,                         // User-provided or auto-generated
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub transcript_segments: Vec<TranscriptSegment>,
    pub status: MeetingStatus,
    pub participants: Vec<String>,            // Speaker labels
}

pub enum MeetingStatus {
    Recording,
    Paused,
    Completed,
}

pub struct TranscriptSegment {
    pub speaker: String,                      // "Speaker 1", "Speaker 2", etc.
    pub start_time: f64,                      // Seconds from meeting start
    pub end_time: f64,
    pub text: String,
    pub confidence: f32,
    pub timestamp: SystemTime,
}
```

**Key Methods**:
```rust
impl MeetingManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self>;
    pub fn start_meeting(&mut self, name: String) -> Result<String>; // Returns meeting_id
    pub fn pause_meeting(&mut self, meeting_id: &str) -> Result<()>;
    pub fn resume_meeting(&mut self, meeting_id: &str) -> Result<()>;
    pub fn end_meeting(&mut self, meeting_id: &str) -> Result<MeetingSummary>;
    pub fn add_segment(&mut self, meeting_id: &str, segment: TranscriptSegment);
    pub fn get_live_transcript(&self, meeting_id: &str) -> Vec<TranscriptSegment>;
}
```

**Integration Points**:
- Use existing `AudioRecordingManager` for audio capture
- Use existing `TranscriptionManager` for transcription
- Emit Tauri events: `transcript-segment`, `meeting-started`, `meeting-ended`

---

### 2. Transcript Storage Module
Create `src-tauri/src/storage/transcript.rs`

**Purpose**: Save meeting transcripts to disk in JSON and Markdown formats

**File Structure**:
```
~/MeetingCoder/
  meetings/
    2025-01-30_stakeholder-call/
      metadata.json              # Meeting metadata
      transcript.json            # Full transcript with timestamps
      transcript.md              # Human-readable markdown
      audio.wav                  # Optional: raw audio backup
```

**Formats**:

**metadata.json**:
```json
{
  "meeting_id": "uuid",
  "name": "Stakeholder Call - Q1 Roadmap",
  "start_time": "2025-01-30T14:30:00Z",
  "end_time": "2025-01-30T15:15:00Z",
  "duration_seconds": 2700,
  "participants": ["Speaker 1", "Speaker 2"]
}
```

**transcript.json**:
```json
{
  "meeting_id": "uuid",
  "segments": [
    {
      "speaker": "Speaker 1",
      "start_time": 0.0,
      "end_time": 3.5,
      "text": "Thanks for joining. I wanted to discuss...",
      "confidence": 0.95,
      "timestamp": "2025-01-30T14:30:00Z"
    }
  ]
}
```

**transcript.md**:
```markdown
# Stakeholder Call - Q1 Roadmap
**Date**: January 30, 2025
**Duration**: 45 minutes
**Participants**: Speaker 1, Speaker 2

---

**[00:00:00] Speaker 1:**
Thanks for joining. I wanted to discuss the new feature...

**[00:00:03] Speaker 2:**
Sounds good. Let's start with the dashboard requirements...
```

---

### 3. macOS System Audio Capture
Modify `src-tauri/src/managers/audio.rs`

**Goal**: Capture both user's voice AND remote participant's voice (system audio)

**Approach**: Integrate with BlackHole virtual audio device
- BlackHole creates a virtual output device
- User configures Zoom/Meet to output to BlackHole
- App captures audio from BlackHole input
- Optionally: also capture microphone and mix streams

**Key Changes**:
```rust
// Add to AudioRecordingManager
pub enum AudioSource {
    Microphone(String),        // Existing: device name
    SystemAudio(String),       // NEW: virtual device name (e.g., "BlackHole 2ch")
    Mixed,                     // NEW: Both mic + system audio
}

impl AudioRecordingManager {
    // NEW: Detect BlackHole installation
    pub fn detect_system_audio_device() -> Option<String>;

    // MODIFY: Support system audio source
    pub fn start_recording(&mut self, source: AudioSource) -> Result<()>;
}
```

**Detection Logic**:
```rust
pub fn detect_system_audio_device() -> Option<String> {
    let devices = list_audio_devices();
    devices.iter()
        .find(|d| d.name.contains("BlackHole") || d.name.contains("Loopback"))
        .map(|d| d.name.clone())
}
```

**User Setup Required** (document in UI):
1. Install BlackHole: `brew install blackhole-2ch`
2. Configure Zoom/Meet: Audio Settings → Output → BlackHole 2ch
3. Mac Audio MIDI Setup: Create "Multi-Output Device" (BlackHole + Speakers) so user can hear
4. Grant microphone permissions to MeetingCoder

---

### 4. Continuous Recording Mode
Modify `src-tauri/src/managers/audio.rs` and `src-tauri/src/managers/transcription.rs`

**Current Behavior**: Push-to-talk (user holds shortcut key)
**Target Behavior**: Always-on recording during meeting, chunk every 30-60s

**Implementation**:
```rust
// In MeetingManager
pub struct MeetingManager {
    audio_manager: Arc<AudioRecordingManager>,
    transcription_manager: Arc<TranscriptionManager>,
    chunk_timer: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl MeetingManager {
    pub fn start_meeting(&mut self, name: String) -> Result<String> {
        // Start audio recording
        self.audio_manager.start_recording(AudioSource::SystemAudio("BlackHole 2ch"))?;

        // Start chunking loop
        let audio_mgr = self.audio_manager.clone();
        let trans_mgr = self.transcription_manager.clone();
        let meeting_id = Uuid::new_v4().to_string();

        let handle = tokio::spawn(async move {
            loop {
                // Wait 30 seconds
                tokio::time::sleep(Duration::from_secs(30)).await;

                // Get audio chunk
                let audio_chunk = audio_mgr.get_buffered_audio(30.0);

                // Transcribe
                let segment = trans_mgr.transcribe(audio_chunk).await;

                // Add to meeting
                self.add_segment(&meeting_id, segment);

                // Emit event to frontend
                emit("transcript-segment", segment);
            }
        });

        // Store handle
        *self.chunk_timer.lock().unwrap() = Some(handle);

        Ok(meeting_id)
    }
}
```

**VAD Filtering**: Keep existing VAD to filter silence, only transcribe non-silent chunks

---

### 5. Speaker Diarization
Enhance `src-tauri/src/managers/transcription.rs`

**Options** (pick simplest first):

**Option A: Whisper Built-in** (if using Whisper Large v3)
```rust
impl TranscriptionManager {
    pub async fn transcribe_with_speakers(&self, audio: Vec<f32>) -> Result<Vec<SpeakerSegment>> {
        // Whisper Large v3 has diarization support
        let params = WhisperInferenceParams {
            enable_diarization: true,
            // ...
        };

        let result = self.engine.transcribe_samples(audio, Some(params))?;
        Ok(result.segments) // Includes speaker labels
    }
}
```

**Option B: Simple Heuristics** (fallback if Whisper doesn't support)
```rust
pub struct SimpleDiarizer {
    speakers: Vec<SpeakerProfile>,
}

impl SimpleDiarizer {
    pub fn identify_speaker(&self, audio_chunk: &[f32]) -> String {
        // Heuristic based on audio characteristics:
        // - Volume level (speaker 1 vs speaker 2)
        // - Frequency profile (voice pitch)
        // - Turn-taking pattern (alternating speakers)

        let energy = calculate_energy(audio_chunk);
        let pitch = estimate_pitch(audio_chunk);

        // Simple clustering: assign to nearest speaker profile
        self.speakers.iter()
            .min_by_key(|s| distance(energy, pitch, s))
            .map(|s| s.label.clone())
            .unwrap_or("Speaker 1".to_string())
    }
}
```

**UI Feature**: Allow user to rename speakers
- "Speaker 1" → "Me"
- "Speaker 2" → "Stakeholder"

---

### 6. Tauri Commands
Create/modify `src-tauri/src/commands.rs`

**Add these commands**:
```rust
#[tauri::command]
pub async fn start_meeting(
    meeting_name: String,
    state: State<'_, Arc<Mutex<MeetingManager>>>
) -> Result<String, String> {
    let mut manager = state.lock().unwrap();
    manager.start_meeting(meeting_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn end_meeting(
    meeting_id: String,
    state: State<'_, Arc<Mutex<MeetingManager>>>
) -> Result<MeetingSummary, String> {
    let mut manager = state.lock().unwrap();
    manager.end_meeting(&meeting_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_live_transcript(
    meeting_id: String,
    state: State<'_, Arc<Mutex<MeetingManager>>>
) -> Result<Vec<TranscriptSegment>, String> {
    let manager = state.lock().unwrap();
    Ok(manager.get_live_transcript(&meeting_id))
}

#[tauri::command]
pub async fn pause_meeting(
    meeting_id: String,
    state: State<'_, Arc<Mutex<MeetingManager>>>
) -> Result<(), String> {
    let mut manager = state.lock().unwrap();
    manager.pause_meeting(&meeting_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_speaker_labels(
    meeting_id: String,
    mapping: HashMap<String, String>,
    state: State<'_, Arc<Mutex<MeetingManager>>>
) -> Result<(), String> {
    let mut manager = state.lock().unwrap();
    manager.update_speaker_labels(&meeting_id, mapping)
        .map_err(|e| e.to_string())
}
```

**Register in `lib.rs`**:
```rust
fn main() {
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(MeetingManager::new())))
        .invoke_handler(tauri::generate_handler![
            start_meeting,
            end_meeting,
            get_live_transcript,
            pause_meeting,
            update_speaker_labels,
            // ... existing commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### 7. Frontend Meeting UI
Create `src/components/Meeting.tsx`

**UI Components**:

**MeetingControls.tsx**:
```typescript
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function MeetingControls() {
  const [isRecording, setIsRecording] = useState(false);
  const [meetingId, setMeetingId] = useState<string | null>(null);
  const [meetingName, setMeetingName] = useState('');

  const startMeeting = async () => {
    const id = await invoke<string>('start_meeting', {
      meetingName: meetingName || 'Untitled Meeting'
    });
    setMeetingId(id);
    setIsRecording(true);
  };

  const endMeeting = async () => {
    if (meetingId) {
      await invoke('end_meeting', { meetingId });
      setIsRecording(false);
      setMeetingId(null);
    }
  };

  return (
    <div className="meeting-controls">
      {!isRecording ? (
        <>
          <input
            type="text"
            placeholder="Meeting name"
            value={meetingName}
            onChange={(e) => setMeetingName(e.target.value)}
          />
          <button onClick={startMeeting}>
            Start Meeting
          </button>
        </>
      ) : (
        <>
          <div className="recording-indicator">
            🔴 Recording ({meetingName})
          </div>
          <button onClick={endMeeting}>
            End Meeting
          </button>
        </>
      )}
    </div>
  );
}
```

**LiveTranscript.tsx**:
```typescript
import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

interface TranscriptSegment {
  speaker: string;
  start_time: number;
  end_time: number;
  text: string;
  confidence: number;
}

export function LiveTranscript() {
  const [segments, setSegments] = useState<TranscriptSegment[]>([]);

  useEffect(() => {
    const unlisten = listen<TranscriptSegment>('transcript-segment', (event) => {
      setSegments((prev) => [...prev, event.payload]);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className="transcript-view">
      <h3>Live Transcript</h3>
      <div className="segments">
        {segments.map((seg, i) => (
          <div key={i} className="segment">
            <strong>{seg.speaker}:</strong> {seg.text}
          </div>
        ))}
      </div>
    </div>
  );
}
```

**StatusIndicator.tsx**:
```typescript
export function StatusIndicator({ isRecording, duration }: { isRecording: boolean; duration: number }) {
  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div className={`status ${isRecording ? 'recording' : 'idle'}`}>
      {isRecording ? (
        <>
          <span className="pulse">🔴</span>
          <span>{formatDuration(duration)}</span>
        </>
      ) : (
        <span>Ready to record</span>
      )}
    </div>
  );
}
```

---

### 8. Testing & Validation
Create `src-tauri/tests/meeting_test.rs`

**Test Cases**:
```rust
#[tokio::test]
async fn test_meeting_lifecycle() {
    let mut manager = MeetingManager::new().unwrap();

    // Start meeting
    let meeting_id = manager.start_meeting("Test Meeting".to_string()).unwrap();
    assert!(!meeting_id.is_empty());

    // Add some segments
    manager.add_segment(&meeting_id, TranscriptSegment {
        speaker: "Speaker 1".to_string(),
        start_time: 0.0,
        end_time: 3.0,
        text: "Hello world".to_string(),
        confidence: 0.95,
        timestamp: SystemTime::now(),
    });

    // Get transcript
    let transcript = manager.get_live_transcript(&meeting_id);
    assert_eq!(transcript.len(), 1);

    // End meeting
    let summary = manager.end_meeting(&meeting_id).unwrap();
    assert_eq!(summary.total_segments, 1);
}

#[tokio::test]
async fn test_system_audio_detection() {
    let device = detect_system_audio_device();
    // Should find BlackHole if installed
    assert!(device.is_some() || cfg!(not(target_os = "macos")));
}

#[tokio::test]
async fn test_transcript_storage() {
    let storage = TranscriptStorage::new();
    let meeting = create_test_meeting();

    storage.save_transcript(&meeting).unwrap();

    // Verify files exist
    let json_path = storage.get_json_path(&meeting.id);
    let md_path = storage.get_markdown_path(&meeting.id);

    assert!(json_path.exists());
    assert!(md_path.exists());
}
```

**Manual Testing Checklist**:
- [ ] Start meeting → audio captures
- [ ] Speak into mic → transcription appears in UI
- [ ] Run for 5+ minutes → no memory leaks or crashes
- [ ] End meeting → files saved correctly
- [ ] Restart app → can load previous transcripts
- [ ] Speaker diarization labels different speakers correctly

---

## Technical Constraints & Reminders

### Coding Standards (from agents.md)
- **Rust**: 500-line file limit, use `Result<T>`, avoid `unwrap()` in production
- **TypeScript**: 300-line file limit, strict mode, no `any`
- **Manager Pattern**: All managers follow consistent API (new, shutdown)
- **Error Handling**: Use `thiserror` for Rust errors, emit user-friendly messages
- **Testing**: >75% coverage for managers, integration tests for end-to-end flows

### Dependencies Already Available
- `tauri` - Framework
- `tokio` - Async runtime
- `serde`, `serde_json` - Serialization
- `cpal` - Audio I/O
- `whisper-rs` / `transcribe-rs` - Transcription engines
- `vad-rs` - Voice Activity Detection
- `hound` - WAV file handling
- `chrono` - Timestamps

### New Dependencies Needed
```toml
# Add to Cargo.toml
uuid = { version = "1.0", features = ["v4", "serde"] }
```

### File Organization
```
src-tauri/src/
  managers/
    meeting.rs           # NEW
    audio.rs             # MODIFY
    transcription.rs     # MODIFY
    model.rs             # KEEP
    history.rs           # KEEP
  storage/               # NEW DIRECTORY
    mod.rs
    transcript.rs
  commands/              # NEW DIRECTORY (or modify existing commands.rs)
    mod.rs
    meeting.rs
```

---

## Handoff Checklist - What to Build First

### Iteration 1: Core Meeting Infrastructure (2-3 hours)
1. Create `managers/meeting.rs` with MeetingManager struct
2. Implement `start_meeting`, `end_meeting`, `add_segment` methods
3. Create Tauri commands: `start_meeting`, `end_meeting`
4. Register commands in `lib.rs`
5. Test: Can start/end meeting via Tauri command

### Iteration 2: Transcript Storage (1-2 hours)
1. Create `storage/transcript.rs`
2. Implement JSON and Markdown serialization
3. Hook into `end_meeting` to save files
4. Test: Files saved correctly to ~/MeetingCoder/meetings/

### Iteration 3: Continuous Audio + Transcription (2-3 hours)
1. Modify `AudioRecordingManager` for continuous mode
2. Implement chunking loop in `MeetingManager`
3. Connect to `TranscriptionManager` for processing
4. Test: Audio recorded continuously, transcribed every 30s

### Iteration 4: System Audio Capture (2-3 hours)
1. Add BlackHole detection to `AudioRecordingManager`
2. Modify recording to support system audio source
3. Document setup steps for user
4. Test: Can capture both sides of Zoom call

### Iteration 5: Speaker Diarization (2-3 hours)
1. Add diarization to `TranscriptionManager`
2. Implement simple speaker labeling
3. Add UI for renaming speakers
4. Test: Different speakers labeled correctly

### Iteration 6: Frontend UI (2-3 hours)
1. Create MeetingControls component
2. Create LiveTranscript component
3. Add StatusIndicator
4. Wire up Tauri events
5. Test: UI updates in real-time

### Iteration 7: Integration & Polish (2-3 hours)
1. End-to-end testing with real Zoom call
2. Fix bugs and edge cases
3. Add error messages and loading states
4. Performance testing (memory, CPU during long meeting)

---

## Expected Outcome

After completing Phase 1, you should have:
- ✅ A working meeting recorder that captures system audio from Zoom/Meet
- ✅ Continuous transcription with <5s lag
- ✅ Speaker diarization (basic, 2-3 speakers)
- ✅ Transcripts saved in JSON + Markdown
- ✅ Clean UI for starting/stopping meetings and viewing live transcripts
- ✅ No crashes or memory leaks during 60+ minute meetings

**Phase 2** (next): Build summarization agent and `.meeting-updates.jsonl` protocol to feed requirements to Claude Code.

---

## Questions? Issues?

- **BlackHole not working?** Check Audio MIDI Setup, verify Multi-Output Device configured
- **No transcription?** Verify Whisper model is downloaded, check `TranscriptionManager` logs
- **Speaker diarization poor?** Start with simple heuristics, can improve in Phase 2
- **Memory issues?** Use ring buffers for audio, limit transcript history in memory

## Ready to Start?

Begin with **Iteration 1: Core Meeting Infrastructure**. Create the `MeetingManager` module and basic Tauri commands. Once that's working, proceed to transcript storage, then continuous audio, etc.

Good luck! 🚀
