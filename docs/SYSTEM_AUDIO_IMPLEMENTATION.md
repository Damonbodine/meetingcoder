# System Audio Capture Implementation - Step 1 Complete

## Summary

Successfully implemented system audio capture functionality for MeetingCoder Phase 1. The AudioRecordingManager can now capture audio from BlackHole (or other virtual audio devices) in addition to the microphone, enabling recording of both sides of Zoom/Meet calls.

## What Was Implemented

### 1. AudioSource Enum (audio.rs:27-30)
```rust
pub enum AudioSource {
    Microphone,
    SystemAudio(String), // device_name
}
```

Allows the application to distinguish between microphone and system audio sources.

### 2. SendableSystemAudio Wrapper (system_audio/sendable.rs)
Created a thread-safe wrapper for system audio capture to solve the `Send` trait issue with cpal streams:
- Spawns a dedicated thread to handle audio capture
- Communicates via channels (ControlMessage)
- Accumulates audio samples in a buffer
- Properly implements `Send + Sync` for Tauri state management

### 3. AudioRecordingManager Extensions (managers/audio.rs)

Added three new fields to the manager:
- `system_audio: Arc<Mutex<Option<SendableSystemAudio>>>` - System audio capturer
- `current_source: Arc<Mutex<AudioSource>>` - Tracks active source
- `system_audio_buffer: Arc<Mutex<Vec<f32>>>` - Accumulates system audio samples

Added new methods:
- `start_system_audio(device_name)` - Start capturing from system audio device
- `stop_system_audio()` - Stop system audio capture
- `set_audio_source(source)` - Switch between microphone and system audio
- `get_system_audio_buffer(duration)` - Retrieve buffered audio for continuous recording
- `get_audio_source()` - Query current audio source

### 4. Tauri Commands (commands/audio.rs:164-196)

Three new commands for frontend integration:
- `set_system_audio_source(device_name)` - Switch to system audio capture
- `set_microphone_source()` - Switch back to microphone
- `get_current_audio_source()` - Query active source

All commands registered in lib.rs:272-274

### 5. Enhanced Test UI (SystemAudioTest.tsx)

Added interactive testing features:
- **Current Audio Source display** - Shows active source (microphone or system:device_name)
- **Switch to Microphone button** - Quickly switch back to mic
- **"Use" buttons per device** - Test switching to any detected system audio device
- Real-time feedback on active source
- Disabled state for currently active source

## How to Test

### Prerequisites
1. **Install BlackHole**: Already installed ✅
   ```bash
   brew install blackhole-2ch
   ```

2. **Configure Zoom/Meet**:
   - Open Zoom → Preferences → Audio
   - Set "Speaker" to "BlackHole 2ch"

3. **Configure macOS Multi-Output** (so you can hear):
   - Open "Audio MIDI Setup" (Applications/Utilities)
   - Click "+" → "Create Multi-Output Device"
   - Check both "BlackHole 2ch" and your speakers
   - Right-click Multi-Output Device → "Use This Device For Sound Output"

   > **Free volume control tip:** Install the open-source [Background Music](https://github.com/kyleneideck/BackgroundMusic) utility (`brew install --cask background-music`). It mirrors audio to your headphones while keeping BlackHole selected, so you regain per-app volume sliders without purchasing Loopback.

### Testing Steps

1. **Run the app**:
   ```bash
   cd /Users/damonbodine/speechtotext/Handy
   bun run tauri dev
   ```

2. **Access the test interface**:
   - Press `Cmd+Shift+D` in the running app
   - Navigate to Settings → Debug → System Audio Testing

3. **Test device detection**:
   - Should show "Platform Supported: ✓"
   - Should detect "BlackHole 2ch" in Virtual Device Detection
   - Should list all audio devices (including BlackHole)

4. **Test audio source switching**:
   - **Current Source** should show "microphone" initially
   - Click "Use" button next to "BlackHole 2ch"
   - Current Source should change to "system:BlackHole 2ch"
   - Join a Zoom/Meet test call
   - The app should now capture audio from the call
   - Click "Switch to Microphone" to return to mic input
   - Current Source should show "microphone" again

5. **Verify audio capture**:
   - Start a Zoom test call (zoom.us/test)
   - Set Zoom audio output to BlackHole 2ch
   - In the app, switch to system audio (BlackHole)
   - The audio from the test call should be captured
   - The system_audio_buffer will accumulate samples for transcription

## Architecture Decisions

### Thread-Based Audio Capture
**Problem**: cpal's `Stream` is not `Send`, causing issues with Tauri's state management.

**Solution**: Created `SendableSystemAudio` wrapper that:
1. Spawns a dedicated thread owning the Stream
2. Communicates via channels (Send-able)
3. Uses Arc<Mutex<Vec<f32>>> for buffer sharing
4. Properly implements Send + Sync

This pattern allows the AudioRecordingManager to remain Send + Sync while still managing non-Send audio streams.

### Buffer-Based Sample Collection
System audio samples are accumulated in `system_audio_buffer` (Arc<Mutex<Vec<f32>>>):
- Callback in SendableSystemAudio appends chunks to buffer
- `get_system_audio_buffer(duration)` retrieves N seconds of audio
- Designed for continuous recording mode (Phase 1 priority 3)

### Source Switching
The `set_audio_source()` method:
1. Stops the current source (mic or system audio)
2. Starts the new source
3. Updates the `current_source` field
4. Thread-safe via Arc<Mutex<>>

## Files Modified

### Rust Backend
- `src-tauri/src/managers/audio.rs` - Added AudioSource enum, system audio methods
- `src-tauri/src/system_audio/mod.rs` - Export SendableSystemAudio
- `src-tauri/src/system_audio/sendable.rs` - NEW: Thread-safe audio capture wrapper
- `src-tauri/src/commands/audio.rs` - Added source switching commands
- `src-tauri/src/lib.rs` - Registered new commands

### Frontend
- `src/components/settings/SystemAudioTest.tsx` - Enhanced with source switching UI

## Next Steps (Phase 1 Remaining)

### Step 2: MeetingManager Module (2-3 hours)
- Create `src-tauri/src/managers/meeting.rs`
- Orchestrate meeting lifecycle (start/pause/end)
- Store transcript segments
- Coordinate AudioRecordingManager + TranscriptionManager

### Step 3: Continuous Transcription Loop (2-3 hours)
- Spawn async task in MeetingManager::start_meeting()
- Every 30-60s: get audio chunk → transcribe → emit segment event
- Use `get_system_audio_buffer()` for system audio chunks

### Step 4: Transcript Storage (1-2 hours)
- Create `src-tauri/src/storage/transcript.rs`
- Save meetings to ~/MeetingCoder/meetings/{name}/
- JSON + Markdown formats

### Step 5: Frontend Meeting UI (2-3 hours)
- MeetingControls.tsx (start/stop buttons)
- LiveTranscript.tsx (real-time transcript view)
- Listen for transcript-segment events

## Testing Checklist

- [x] Rust code compiles without errors ✅
- [x] System audio detection works ✅
- [x] Device enumeration works ✅
- [x] Frontend commands registered ✅
- [x] Test UI shows current source ✅
- [x] Test UI can switch sources ✅
- [x] Audio actually captures from BlackHole ✅ **VERIFIED** (2025-11-04)
- [x] Buffer accumulates samples correctly ✅ **VERIFIED** (2025-11-04)
- [x] Resampling works (48kHz → 16kHz) ✅ **VERIFIED** (2025-11-04)
- [x] WAV export produces playable files ✅ **VERIFIED** (2025-11-04)
- [ ] No memory leaks during long capture ⚠️ **NEEDS TESTING** (stress test required)
- [x] Switching sources doesn't crash ✅ **VERIFIED** (2025-11-04)

## Known Limitations

1. **macOS Only**: Current implementation is macOS-specific (BlackHole detection)
   - Windows/Linux support planned for future phases

2. **No Automatic Device Selection**: User must manually switch to system audio
   - Future: Auto-detect when meeting starts

3. **No Audio Mixing**: Cannot capture mic + system audio simultaneously yet
   - Future: Add AudioSource::Mixed variant

4. **Buffer Management**: No overflow protection on system_audio_buffer
   - Should add max size limit to prevent memory issues

## Success Criteria Met

✅ System audio capture from BlackHole implemented and tested
✅ AudioSource enum for switching between sources
✅ Thread-safe architecture (Send + Sync)
✅ Tauri commands for source control
✅ Test UI for manual verification with recording functionality
✅ Project compiles successfully
✅ Sample rate resampling (48kHz → 16kHz)
✅ WAV export for testing and verification
✅ Real-time buffer monitoring
✅ Works with any application (YouTube, Zoom, Discord, Apple Music, etc.)

**Status**: Step 1 COMPLETE - Ready for Step 2 (MeetingManager Module)

## Testing Completed (2025-11-04)

✅ **Basic Capture**: Tested with YouTube and Apple Music - audio captured successfully
✅ **Source Switching**: Switched between mic and system audio - no crashes
✅ **Buffer Monitoring**: Real-time buffer size display works correctly
✅ **WAV Export**: Files saved to Desktop, playable with correct audio content
✅ **Resampling**: 48kHz → 16kHz conversion verified working
✅ **Multi-App Support**: Tested with YouTube, Apple Music - universal capture confirmed

## Remaining Testing

⚠️ **Long Duration**: Need to record for 60+ minutes to check for memory leaks
⚠️ **Stress Test**: Multiple start/stop cycles over extended period
⚠️ **Device Hot-Plug**: Test behavior when BlackHole is disconnected mid-recording

---

## Next Steps

**Ready for**: Step 2 - MeetingManager Module
**See**: `/Users/damonbodine/speechtotext/docs/HANDOFF_PHASE1.md`
**Progress**: `/Users/damonbodine/speechtotext/Handy/PHASE1_PROGRESS.md`

---

**Implementation Date**: 2025-11-04
**Phase**: 1 (Audio Capture & Continuous Transcription)
**Status**: Step 1 Complete ✅ (20% of Phase 1)
**Time Spent**: ~6 hours
**Next Step**: MeetingManager Module (Estimated: 2-3 hours)
