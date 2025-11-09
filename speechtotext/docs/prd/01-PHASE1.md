# Phase 1: Audio Capture & Transcription

## Overview

Phase 1 establishes the foundation by enabling continuous system audio capture from video conferencing applications (Zoom, Google Meet) and producing accurate, timestamped transcriptions with speaker identification.

**Timeline**: 4-6 weeks
**Priority**: P0 (Blocking for all other phases)

## Status Update (2025-11-05)

- Working on macOS:
  - System audio capture via BlackHole 2ch
  - Device-rate detection + robust resampling (44.1/48k → 16k)
  - Continuous meeting transcription loop (10s chunks)
  - Auto-save transcripts to `~/MeetingCoder/meetings/`
  - Frontend event emits for live segments
- Not yet complete:
  - Speaker diarization (placeholder speaker labels)
  - Markdown transcript export
  - Long-duration (60+ min) soak tests
  - Windows/Linux system audio capture

## Goals

1. Capture both sides of a video conference call (user + remote participants)
2. Continuously transcribe audio with minimal latency (<5 seconds)
3. Identify and label different speakers (speaker diarization)
4. Save transcriptions with timestamps to structured files
5. Provide simple UI for starting/stopping meeting capture

## Success Criteria

- [x] System audio capture works on macOS ✅ COMPLETE (2025-11-05) — Windows/Linux ❌ NOT STARTED
- [x] Transcription accuracy acceptable on clear audio ✅ COMPLETE (Whisper/Parakeet local)
- [ ] Speaker labels identify 2-3 participants 80%+ ❌ NOT STARTED (placeholder labels)
- [x] Streaming latency aligned to chunk size ✅ COMPLETE (10s chunks; configurable later)
- [ ] No audio dropouts during 60+ min calls ⚠️ NEEDS TESTING (not soak-tested)
- [ ] Transcripts saved with full metadata ⚠️ PARTIAL (JSON saved; Markdown pending)

**Phase 1 Progress**: ~60% Complete (macOS path working end-to-end)

## Features & Requirements

### 1.1 System Audio Capture

**User Story**: As a user, I want to capture both my voice and my stakeholder's voice from a Zoom/Google Meet call so that I have complete conversation context.

**Requirements**:

#### macOS Implementation
- Integrate with virtual audio devices (BlackHole or LoopbackAudio)
- Detect when virtual audio device is configured
- Capture mixed audio stream (input + output)
- Handle Core Audio API permissions

**Acceptance Criteria**:
- [x] Detects BlackHole/Loopback installation ✅ **COMPLETE** (Test UI implemented & working)
- [x] Guides user through audio routing setup ✅ **COMPLETE** (Setup instructions in test UI & docs)
- [x] Captures clear audio from both parties ✅ **COMPLETE** (2025-11-04 - Fully functional with resampling)
- [x] Audio quality sufficient for transcription (16kHz minimum) ✅ **COMPLETE** (48kHz → 16kHz resampling implemented)

**Implementation Details** (2025-11-04):
- **AudioSource enum** added for switching between Microphone and SystemAudio
- **SendableSystemAudio wrapper** created to solve `Send` trait issues with cpal streams
- **Thread-based architecture** with channel communication for audio capture
- **Automatic resampling** using rubato library (48kHz → 16kHz)
- **Real-time buffer monitoring** with visual feedback in test UI
- **WAV export functionality** for testing and verification
- **Files**:
  - `src-tauri/src/managers/audio.rs` - AudioRecordingManager extensions
  - `src-tauri/src/system_audio/sendable.rs` - Thread-safe audio wrapper
  - `src/components/settings/SystemAudioTest.tsx` - Test UI with recording controls
- **Documentation**:
  - `/Handy/SYSTEM_AUDIO_IMPLEMENTATION.md` - Technical implementation details
  - `/Handy/HOW_TO_TEST_YOUTUBE_RECORDING.md` - Testing guide

#### Windows Implementation
- Use WASAPI loopback recording API
- Capture desktop audio output
- Handle Windows audio permissions
- Support virtual cables (VB-Audio Cable)

**Acceptance Criteria**:
- [ ] WASAPI loopback captures system audio ❌ **NOT STARTED**
- [ ] Works with default audio devices ❌ **NOT STARTED**
- [ ] Handles device changes mid-call ❌ **NOT STARTED**
- [ ] No echo or feedback loops ❌ **NOT STARTED**

#### Linux Implementation
- Use PulseAudio monitor sources
- Support PipeWire audio subsystem
- Capture both input and output streams

**Acceptance Criteria**:
- [ ] Works with PulseAudio ❌ **NOT STARTED**
- [ ] Compatible with PipeWire ❌ **NOT STARTED**
- [ ] Detects and uses correct monitor source ❌ **NOT STARTED**

### 1.2 Continuous Transcription

**User Story**: As a user, I want the application to transcribe my meeting in real-time without me having to manually trigger recording segments.

**Requirements**:
- Modify Handy's on-demand transcription to continuous mode
- Chunk audio into processable segments (30-60 second chunks)
- Queue chunks for transcription to prevent blocking
- Use Whisper Large or Parakeet for highest accuracy
- Handle overlapping speech gracefully

**Technical Design**:
```rust
// Pseudo-code for continuous transcription loop
loop {
    let audio_chunk = audio_buffer.get_chunk(duration: 30s);
    let transcript_future = transcribe_async(audio_chunk);

    // Don't block - queue for processing
    transcript_queue.push(transcript_future);

    // Process completed transcriptions
    while let Some(result) = transcript_queue.try_pop() {
        handle_transcript(result);
    }
}
```

**Acceptance Criteria**:
- [x] Transcribes continuously without manual intervention ✅ COMPLETE (meeting loop)
- [x] Chunk size balances latency/accuracy ✅ COMPLETE (10s; setting pending)
- [ ] No memory leaks during 2+ hour calls ❌ NOT TESTED
- [ ] Graceful handling of silence periods ⚠️ PARTIAL (mic VAD; system-audio skips empty)
- [x] Keeps pace with real-time ✅ COMPLETE (10s processing stable)

### 1.3 Speaker Diarization

**User Story**: As a user, I want to know who said what so the AI can distinguish between my requirements and stakeholder feedback.

**Requirements**:
- Implement speaker diarization (2-3 speaker scenario)
- Label speakers as "Speaker 1", "Speaker 2", etc.
- Optionally allow user to rename speakers ("You", "Stakeholder", "Designer")
- Maintain speaker consistency throughout call

**Technical Options**:
1. **Whisper built-in** (if using Whisper Large v3)
2. **pyannote-audio** (separate pipeline, higher accuracy)
3. **Simple heuristics** (audio level + turn-taking patterns)

**Acceptance Criteria**:
- [ ] Identifies 2-3 distinct speakers ❌ **NOT STARTED**
- [ ] Speaker labels are consistent (same speaker = same label) ❌ **NOT STARTED**
- [ ] 80%+ accuracy on speaker identification ❌ **NOT STARTED**
- [ ] Handles overlapping speech (marks as "Multiple Speakers") ❌ **NOT STARTED**
- [ ] UI allows post-call speaker renaming ❌ **NOT STARTED**

### 1.4 Transcript Storage

**User Story**: As a user, I want my meeting transcripts saved in a structured, readable format so I can review them later or feed them to other tools.

**Requirements**:
- Save transcripts in JSON and Markdown formats
- Include metadata (meeting date, duration, participants)
- Timestamp each segment with millisecond precision
- Store in organized directory structure

**File Structure**:
```
~/MeetingCoder/
  meetings/
    2025-01-15_stakeholder-call/
      metadata.json
      transcript.json
      transcript.md
      audio.wav (optional raw audio backup)
```

**Transcript JSON Schema**:
```json
{
  "meeting_id": "uuid",
  "start_time": "2025-01-15T14:30:00Z",
  "end_time": "2025-01-15T15:15:00Z",
  "duration_seconds": 2700,
  "participants": ["Speaker 1", "Speaker 2"],
  "segments": [
    {
      "speaker": "Speaker 1",
      "start_time": 0.0,
      "end_time": 3.5,
      "text": "Thanks for joining. I wanted to discuss the new feature...",
      "confidence": 0.95
    }
  ]
}
```

**Acceptance Criteria**:
- [x] Transcripts saved automatically at meeting end ✅ COMPLETE (JSON)
- [ ] Both JSON and Markdown formats generated ⚠️ PARTIAL (JSON only)
- [ ] Timestamps accurate to <1 second ⚠️ PARTIAL (chunk-based timing)
- [x] Files are human-readable ✅ COMPLETE (JSON on disk)
- [ ] Metadata includes all relevant context ⚠️ PARTIAL (participants placeholder)

### 1.5 User Interface

**User Story**: As a user, I want a simple interface to start/stop meeting capture and see live transcription status.

**Requirements**:

#### Main Window Updates
- Add "Meeting Mode" toggle to settings
- Add "Start Meeting Capture" button
- Show live transcription status indicator
- Display real-time transcript preview (last 10 segments)

#### System Tray
- Keep existing tray icon
- Add "Start Meeting Capture" menu item
- Show notification when meeting capture starts/ends
- Indicate active capture with icon change

#### Settings Panel
- Audio device selection (system audio source)
- Transcription model selection (Whisper Large vs. Parakeet)
- Speaker diarization on/off toggle
- Auto-save location configuration
- Language selection (English default)

**Mockup** (text description):
```
┌─────────────────────────────────────┐
│ MeetingCoder                    [X] │
├─────────────────────────────────────┤
│ 🔴 Meeting in Progress (15:32)     │
│                                      │
│ Recent Transcript:                  │
│ ┌──────────────────────────────────┐│
│ │ Speaker 1: "So the user should   ││
│ │ be able to upload a CSV file..." ││
│ │                                   ││
│ │ Speaker 2: "Yes, and then we     ││
│ │ need to validate the columns..." ││
│ └──────────────────────────────────┘│
│                                      │
│ [Stop Meeting]  [View Full Transcript]│
│                                      │
│ Settings: ⚙️  History: 📁           │
└─────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] One-click start/stop meeting capture ❌ **NOT STARTED** (no meeting mode)
- [x] Real-time status visible at all times ✅ **COMPLETE** (has tray icon/overlay)
- [ ] Live transcript preview updates every 5-10 seconds ❌ **NOT STARTED**
- [x] Easy access to full transcript and settings ✅ **COMPLETE** (has settings UI)
- [x] Keyboard shortcut to start/stop (e.g., Cmd+Shift+M) ✅ **COMPLETE** (has global shortcuts)

## Technical Architecture Changes

### From Handy Foundation

**Keep** ✅ **ALL COMPLETE**:
- [x] Tauri application framework ✅
- [x] Whisper/Parakeet transcription engines ✅
- [x] VAD (Voice Activity Detection) for silence filtering ✅
- [x] Audio resampling pipeline (rubato) ✅
- [x] Settings management system ✅
- [x] System tray integration ✅

**Modify**:
- `AudioRecordingManager`: Change from on-demand to continuous mode
- `TranscriptionManager`: Add chunking and queuing logic
- UI: Add meeting-specific controls and live preview

**Add New**:
- `SystemAudioCapture` module (platform-specific)
- `SpeakerDiarization` module
- `TranscriptStorage` module
- `MeetingSession` state manager

### New Rust Modules

```rust
// src-tauri/src/system_audio.rs
pub struct SystemAudioCapture {
    // Platform-specific audio capture
}

// src-tauri/src/diarization.rs
pub struct SpeakerDiarizer {
    // Speaker identification logic
}

// src-tauri/src/meeting.rs
pub struct MeetingSession {
    pub id: String,
    pub start_time: SystemTime,
    pub transcript_segments: Vec<TranscriptSegment>,
    pub speakers: HashMap<String, Speaker>,
}

// src-tauri/src/storage.rs
pub struct TranscriptStorage {
    // Save/load transcript files
}
```

## Dependencies

**New Rust Crates**:
- `cpal` (already in Handy) - audio I/O
- `hound` - WAV file reading/writing
- `serde_json` - JSON serialization
- Platform-specific:
  - macOS: `coreaudio-sys`
  - Windows: `windows` crate for WASAPI
  - Linux: `libpulse-binding` or `pipewire-rs`

**New Frontend Dependencies**:
- None (use existing React/TypeScript stack)

## Testing Strategy

### Unit Tests
- Audio chunk processing
- Speaker diarization logic
- Transcript serialization/deserialization
- File I/O operations

### Integration Tests
- End-to-end audio capture → transcription → storage
- Multi-speaker scenarios
- Long-running calls (2+ hours)
- Device switching mid-call

### Manual Testing
- Real Zoom calls with 2-3 participants
- Google Meet compatibility
- Various audio quality conditions
- Background noise handling

### Performance Benchmarks
- Transcription latency (target: <5s)
- Memory usage during 2-hour call (target: <1GB)
- CPU usage (target: <50% on recommended hardware)

## Known Challenges & Mitigations

| Challenge | Mitigation |
|-----------|------------|
| System audio setup complexity | Detailed setup guides + auto-detection |
| Platform-specific audio APIs | Abstract behind common interface |
| Speaker diarization accuracy | Fallback to simple heuristics if needed |
| Transcription latency | Use GPU acceleration, optimize chunk size |
| Audio quality from compressed calls | Use Whisper Large for robustness |

## Documentation Deliverables

1. **User Setup Guide**: How to configure system audio routing
2. **API Documentation**: Internal module interfaces
3. **Testing Guide**: How to test with simulated meetings
4. **Troubleshooting**: Common audio capture issues

## Phase 1 Completion Checklist

- [ ] All acceptance criteria met for features 1.1-1.5 ⚠️ IN PROGRESS (~60%)
  - [x] 1.1 System Audio Capture ✅ macOS complete; Win/Linux pending
  - [x] 1.2 Continuous Transcription ✅ meeting loop (10s)
  - [ ] 1.3 Speaker Diarization ❌ not started
  - [ ] 1.4 Transcript Storage ⚠️ partial (JSON only)
  - [ ] 1.5 User Interface ⚠️ partial (basic controls + events)
- [ ] Unit test coverage >80% ❌ not measured
- [ ] Integration tests pass on all platforms ❌ not implemented
- [ ] Manual testing: 10+ real calls ❌ not started
- [ ] Performance benchmarks meet targets ⚠️ partial (baseline OK, soak pending)
- [ ] Documentation complete and reviewed ⚠️ in progress
- [ ] Ready to begin Phase 2 (LLM integration) ⚠️ partially (Phase 1 macOS path ready)

## Handoff to Phase 2

Phase 1 deliverables required for Phase 2:
1. Working continuous transcription system
2. Structured transcript files (JSON format)
3. Speaker-labeled segments
4. Stable API for accessing live transcript data

Phase 2 will consume the JSON transcript format and begin LLM integration for requirement extraction and code generation.
