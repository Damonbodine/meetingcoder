# Debugging Log - Meeting Transcription Feature

**Date**: November 5, 2025
**Issue**: Meeting transcription returning 0 segments despite audio being captured
**Project**: Handy - MeetingCoder Feature

---

## Original Bug Report

From `HANDOFF_FINAL_BUG_FIX.md`:
- **Error**: `Model is not loaded for transcription.`
- **Root Cause**: Whisper model was not being loaded when meeting started
- **Expected Fix**: Add `initiate_model_load()` call in `start_meeting()` function

---

## Changes Made

### 1. Fixed Model Loading (COMPLETED ✅)

**File**: `src-tauri/src/managers/meeting.rs:134-154`

**Change**: Added model loading logic to `start_meeting()` function:
```rust
// Load the transcription model before starting transcription
log::info!("Loading transcription model...");
self.transcription_manager.initiate_model_load();

// Wait for model to load in background task
let transcription_manager = self.transcription_manager.clone();
tokio::spawn(async move {
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
```

**Result**: Model now loads successfully - confirmed by log: `Transcription model loaded successfully`

### 2. Fixed Frontend Event Listener (COMPLETED ✅)

**File**: `src/components/meeting/MeetingView.tsx:17-60`

**Problem**: Browser console errors: `undefined is not an object (evaluating 'listeners[eventId].handlerId')`

**Change**: Added proper cleanup and mounted state tracking:
```typescript
let unlisten: (() => void) | undefined;
let isMounted = true;

const setupListener = async () => {
  try {
    const unlistenFn = await listen<{ meeting_id: string; segment: TranscriptSegment }>(
      "transcript-segment-added",
      (event) => {
        if (event.payload.meeting_id === activeMeetingId && isMounted) {
          setTranscriptSegments((prev) => [...prev, event.payload.segment]);
          // ... toast notification
        }
      }
    );
    if (isMounted) {
      unlisten = unlistenFn;
    } else {
      unlistenFn();
    }
  } catch (error) {
    console.error("Failed to setup event listener:", error);
  }
};

return () => {
  isMounted = false;
  if (unlisten) {
    try {
      unlisten();
    } catch (error) {
      console.error("Error cleaning up listener:", error);
    }
  }
};
```

**Result**: No more event listener errors in browser console

### 3. Added Debug Logging (IN PROGRESS 🔄)

**Files Modified**:
- `src-tauri/src/managers/meeting.rs:372-391`
- `src-tauri/src/managers/meeting.rs:403`

**Added Logs**:
1. System audio buffer size before reading
2. Audio chunk statistics (RMS, Peak levels)
3. Transcription result text and length

---

## Current Status

### What's Working ✅

1. **Meeting Infrastructure**:
   - Meeting starts successfully
   - Meeting ID generated and tracked
   - Meeting ends successfully with summary

2. **Model Loading**:
   - Model loads successfully: `Transcription model loaded successfully`
   - Model is available when transcription is requested

3. **Audio Capture**:
   - System audio (BlackHole 2ch) is capturing
   - Audio buffer accumulating samples
   - Buffer sizes: 590,400 - 889,600 samples before reads
   - 480,000 samples (30 seconds at 16kHz) extracted per chunk

4. **Transcription Processing**:
   - Transcription loop runs every 30 seconds
   - Audio sent to model (480,000 samples)
   - Model processes audio (~1.7-2.3 seconds per chunk)
   - No errors during transcription

### What's NOT Working ❌

**Transcription Returns Empty Strings**:
```
[2025-11-05T16:18:35Z INFO] Transcription result: '' (length: 0 chars)
[2025-11-05T16:18:35Z WARN] Empty transcription returned from model, skipping segment 0
```

**Final Meeting Summary**:
```
Ended meeting: Testers - Duration: 69s, Segments: 0
```

---

## Debugging Steps Performed

### Step 1: Verified Commands Are Exposed
- Checked `src-tauri/src/lib.rs` - all meeting commands registered
- Confirmed MeetingManager initialized and managed by Tauri state

### Step 2: Verified Audio Capture
**Logs Show**:
- Device: BlackHole 2ch
- Sample Rate: 16000 Hz (matches target)
- Channels: 2 (stereo)
- Format: F32
- System audio capture started successfully

### Step 3: Verified Audio Buffer Accumulation
**Multiple test runs showed**:
- Buffer accumulating samples: 590K - 889K samples
- Consistent 480K sample extractions (30 seconds)
- No "No audio captured" warnings

### Step 4: Verified Model Processing
**Logs Show**:
- "Audio vector length: 480000" (from transcription.rs:317)
- Processing time: 1.6-2.4 seconds per chunk
- No transcription errors
- Model returns successfully (no exceptions)

### Step 5: Investigated Audio Pipeline
**Checked**:
1. **Stereo to Mono Conversion**: ✅ Working
   - Code in `system_audio/macos.rs:268-275` averages channels

2. **Sample Rate**: ✅ Correct
   - Device: 16000 Hz
   - Target: 16000 Hz (WHISPER_SAMPLE_RATE)
   - No resampling needed

3. **Audio Format**: ✅ Correct
   - F32 samples
   - Proper conversion from cpal format

---

## Current Hypothesis

The Parakeet TDT model is receiving audio but returning empty transcriptions. Possible causes:

### Theory 1: Audio Amplitude Too Low
- Audio might be captured but too quiet
- Model might have silence detection threshold
- **Next**: Check RMS and peak levels (debug logging added)

### Theory 2: Audio Preprocessing Issue
- Model might expect specific normalization
- Audio might need Voice Activity Detection (VAD)
- Sample format might be incorrect for Parakeet

### Theory 3: Model Configuration Issue
- Parakeet inference params might be incorrect
- Timestamp granularity setting might cause issues
- Model might need specific input format

### Theory 4: Chunk Duration Issue
- 30 seconds might be too long for the model
- Model might have maximum input length
- Shorter chunks (10-15 seconds) might work better

---

## Technical Context

### System Information
- **OS**: macOS (Darwin 24.6.0)
- **Audio Device**: BlackHole 2ch (virtual audio device)
- **Model**: parakeet-tdt-0.6b-v3
- **Sample Rate**: 16000 Hz
- **Audio Format**: F32, Mono (converted from stereo)

### Audio Flow
```
BlackHole 2ch (Stereo, 16kHz, F32)
    ↓
MacOSSystemAudio::build_stream()
    ↓
Stereo to Mono Conversion (averaging)
    ↓
System Audio Buffer (Arc<Mutex<Vec<f32>>>)
    ↓
MeetingManager::transcription_loop() [every 30s]
    ↓
get_system_audio_buffer(30.0) -> 480,000 samples
    ↓
TranscriptionManager::transcribe()
    ↓
Parakeet Engine::transcribe_samples()
    ↓
Result: "" (empty string)
```

### Code Locations
- **Meeting Manager**: `src-tauri/src/managers/meeting.rs`
- **Transcription Manager**: `src-tauri/src/managers/transcription.rs`
- **Audio Manager**: `src-tauri/src/managers/audio.rs`
- **macOS Audio Capture**: `src-tauri/src/system_audio/macos.rs`
- **Frontend**: `src/components/meeting/MeetingView.tsx`

---

## Log Examples

### Successful Meeting Start
```
[2025-11-05T16:18:02Z INFO] Started meeting: Testers (ID: 63d22c92-e510-4293-992a-3eb858788a1a)
[2025-11-05T16:18:02Z INFO] Loading transcription model...
[2025-11-05T16:18:02Z INFO] Transcription task spawned for meeting: 63d22c92-...
[2025-11-05T16:18:02Z INFO] Starting transcription loop for meeting: 63d22c92-...
[2025-11-05T16:18:04Z INFO] Transcription model loaded successfully
```

### Transcription Attempt (Empty Result)
```
[2025-11-05T16:18:32Z INFO] System audio buffer size before read: 590400 samples
Audio vector length: 480000
took 2363ms
[2025-11-05T16:18:35Z INFO] Transcription result: '' (length: 0 chars)
[2025-11-05T16:18:35Z WARN] Empty transcription returned from model, skipping segment 0
```

### Meeting End
```
[2025-11-05T16:19:12Z INFO] Transcript saved for meeting: Testers
[2025-11-05T16:19:12Z INFO] Ended meeting: Testers - Duration: 69s, Segments: 0
```

---

## Pending Investigation

### Not Yet Checked ⏳

1. **Audio Level Analysis**:
   - RMS (Root Mean Square) amplitude
   - Peak levels
   - Whether audio is actually silent or just low volume

2. **Model Input Requirements**:
   - Does Parakeet TDT expect specific audio preprocessing?
   - Does it need VAD filtering?
   - Input format requirements (sample rate, normalization)

3. **Alternative Chunk Durations**:
   - Test with 10 seconds instead of 30
   - Test with 15 seconds
   - Check model maximum input length

4. **Compare with Regular Transcribe**:
   - Does the Option+Space shortcut transcribe work?
   - If yes, what's different between that and meeting transcription?

5. **Test with Different Audio Sources**:
   - Try with microphone instead of system audio
   - Test if issue is specific to BlackHole/virtual devices

---

## Next Steps (Recommendations)

### High Priority
1. **Check audio levels** - Run test with latest logging to see RMS/peak
2. **Compare with working transcription** - Test Option+Space shortcut
3. **Try shorter chunks** - Change from 30s to 10s chunks
4. **Test with microphone** - Verify not a BlackHole-specific issue

### Medium Priority
5. **Add VAD preprocessing** - Check if silence detection helps
6. **Review Parakeet docs** - Verify correct usage of parakeet-tdt model
7. **Check model input preprocessing** - May need normalization

### Low Priority
8. **Test different model** - Try with Whisper instead of Parakeet
9. **Increase logging in transcribe-rs** - Add debug output in the engine
10. **Check for known issues** - Search for Parakeet TDT empty output bugs

---

## Files Modified Summary

```
src-tauri/src/managers/meeting.rs          - Model loading + debug logging
src/components/meeting/MeetingView.tsx     - Event listener fix
```

## Files To Review

```
src-tauri/src/managers/transcription.rs    - Check Parakeet inference params
src-tauri/src/managers/audio.rs            - Verify audio buffer handling
src-tauri/src/system_audio/macos.rs        - Verify audio capture/conversion
```

---

## Questions for Next Agent

1. Why is Parakeet returning empty strings with valid audio input?
2. Should we add VAD (Voice Activity Detection) before transcription?
3. Is 30 seconds too long for a single transcription chunk?
4. Are there any preprocessing steps missing for the Parakeet TDT model?
5. Should we test with the regular Whisper transcription to rule out audio pipeline issues?

---

**Status**: Need fresh investigation into why model returns empty transcription despite receiving valid audio data.
