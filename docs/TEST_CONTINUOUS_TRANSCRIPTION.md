# Testing Continuous Transcription (Step 3)

## Quick Test via Browser Console

### Prerequisites
1. Ensure BlackHole 2ch is installed (see `HOW_TO_TEST_YOUTUBE_RECORDING.md`)
2. Set up Multi-Output Device in Audio MIDI Setup
3. Have a Whisper model downloaded (Small, Medium, or Turbo)

### 1. Start the Application
```bash
cd /Users/damonbodine/speechtotext/Handy
bun run tauri dev
```

### 2. Open Browser Console
Press `Cmd+Option+I` (macOS) or `F12` (Windows/Linux)

### 3. Set System Audio Source
```javascript
const { invoke } = window.__TAURI__.core;

// Set to BlackHole for system audio capture
await invoke('set_system_audio_source', { deviceName: 'BlackHole 2ch' });
console.log('Audio source set to BlackHole');
```

### 4. Start a Meeting
```javascript
// Start a new meeting
const meetingId = await invoke('start_meeting', { 
  meetingName: 'Test Meeting' 
});
console.log('Meeting started:', meetingId);

// Verify it's active
const activeMeetings = await invoke('get_active_meetings');
console.log('Active meetings:', activeMeetings);
```

### 5. Play Audio (YouTube, Apple Music, etc.)
- The transcription loop will capture 30-second chunks
- First transcription happens after 30 seconds

### 6. Listen for Transcript Events
```javascript
const { listen } = window.__TAURI__.event;

// Listen for new transcript segments
await listen('transcript-segment-added', (event) => {
  console.log('New segment received:', event.payload);
  console.log('Speaker:', event.payload.segment.speaker);
  console.log('Text:', event.payload.segment.text);
  console.log('Time:', event.payload.segment.start_time, '-', event.payload.segment.end_time);
});
```

### 7. Check Live Transcript
```javascript
// Get current transcript
const transcript = await invoke('get_live_transcript', { meetingId });
console.log('Current transcript:', transcript);

// Display formatted
transcript.forEach(seg => {
  console.log(`[${seg.start_time.toFixed(0)}s] ${seg.speaker}: ${seg.text}`);
});
```

### 8. Pause/Resume (Optional)
```javascript
// Pause the meeting
await invoke('pause_meeting', { meetingId });
console.log('Meeting paused');

// Resume
await invoke('resume_meeting', { meetingId });
console.log('Meeting resumed');
```

### 9. End the Meeting
```javascript
// End the meeting and get summary
const summary = await invoke('end_meeting', { meetingId });
console.log('Meeting summary:', summary);
console.log('Duration:', summary.duration_seconds, 'seconds');
console.log('Total segments:', summary.total_segments);
console.log('Participants:', summary.participants);
```

### 10. Check Saved Transcript
```bash
# Open the saved transcript
open ~/MeetingCoder/meetings/
```

Look for a folder named `YYYY-MM-DD_test-meeting/` containing:
- `metadata.json` - Meeting info
- `transcript.json` - Structured data
- `transcript.md` - Human-readable format

---

## Expected Behavior

### Timing
- **First segment**: Appears ~30 seconds after starting meeting
- **Subsequent segments**: Every 30 seconds if audio is present
- **Empty chunks**: Skipped (no segment created)

### Audio Buffer
```javascript
// Check buffer size
const bufferSize = await invoke('get_system_audio_buffer_size');
console.log('Buffer size:', bufferSize, 'samples');
console.log('Seconds:', (bufferSize / 16000).toFixed(1));
```

### Logs
Check the console output (where you ran `bun run tauri dev`) for:
- "Starting transcription loop for meeting: {id}"
- "Processing audio chunk with X samples"
- "Added segment N to meeting: {id}"
- "Transcription loop ended for meeting: {id}"

---

## Troubleshooting

### No Transcription Happening
1. **Check audio source**:
   ```javascript
   const source = await invoke('get_current_audio_source');
   console.log('Current audio source:', source);
   ```
   Should show: `{"SystemAudio":"BlackHole 2ch"}`

2. **Check buffer accumulation**:
   ```javascript
   // Wait a few seconds, then check
   await new Promise(r => setTimeout(r, 5000));
   const size = await invoke('get_system_audio_buffer_size');
   console.log('Buffer growing?', size > 0);
   ```

3. **Check model loaded**:
   ```javascript
   const modelStatus = await invoke('get_transcription_model_status');
   console.log('Model status:', modelStatus);
   ```

### Transcription Too Slow
- Using Large model? Switch to Small or Medium
- Check CPU usage - transcription is CPU-intensive
- First transcription loads model (takes longer)

### Meeting Not Ending
```javascript
// Force end if needed
await invoke('end_meeting', { meetingId });

// Or check if it's actually ended
const active = await invoke('get_active_meetings');
console.log('Still active?', active.includes(meetingId));
```

---

## Integration Test Script

```javascript
// Complete test flow
async function testMeetingFlow() {
  const { invoke } = window.__TAURI__.core;
  
  console.log('🎬 Starting meeting flow test...');
  
  // 1. Set audio source
  await invoke('set_system_audio_source', { deviceName: 'BlackHole 2ch' });
  console.log('✅ Audio source set');
  
  // 2. Start meeting
  const meetingId = await invoke('start_meeting', { 
    meetingName: 'Integration Test' 
  });
  console.log('✅ Meeting started:', meetingId);
  
  // 3. Wait for first transcription (30s)
  console.log('⏳ Waiting 35 seconds for first transcription...');
  await new Promise(r => setTimeout(r, 35000));
  
  // 4. Check transcript
  const transcript = await invoke('get_live_transcript', { meetingId });
  console.log('✅ Transcript segments:', transcript.length);
  
  if (transcript.length > 0) {
    console.log('📝 First segment:', transcript[0].text);
  }
  
  // 5. End meeting
  const summary = await invoke('end_meeting', { meetingId });
  console.log('✅ Meeting ended');
  console.log('📊 Summary:', summary);
  
  console.log('🎉 Test complete!');
}

// Run the test
testMeetingFlow();
```

---

## Next Steps

After verifying continuous transcription works:
1. Build frontend UI (Step 5)
2. Add real-time display of segments
3. Add speaker labeling
4. Add export functionality

**Ready for Step 5: Frontend Meeting UI** 🚀
