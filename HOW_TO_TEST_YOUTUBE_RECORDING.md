# How to Test System Audio Recording with YouTube

## Step-by-Step Guide to Record YouTube Audio

### 1. Configure macOS Audio (One-Time Setup)

**Set up BlackHole so you can both HEAR and RECORD:**

1. Open **Audio MIDI Setup** (Applications → Utilities → Audio MIDI Setup)
2. Click the **"+"** button at bottom left
3. Select **"Create Multi-Output Device"**
4. In the Multi-Output Device:
   - ✅ Check **"BlackHole 2ch"**
   - ✅ Check your **speakers/headphones** (e.g., "MacBook Pro Speakers")
5. Right-click the Multi-Output Device → **"Use This Device For Sound Output"**

**Why?** Without this, when you route audio to BlackHole, you won't hear anything!

---

### 2. Run the App

```bash
cd /Users/damonbodine/speechtotext/Handy
bun run tauri dev
```

Wait for the app to open.

---

### 3. Access Debug Mode

In the running app:
- Press **`Cmd + Shift + D`**
- Navigate to **Settings → Debug** (or System Audio Testing section)

---

### 4. Verify BlackHole is Detected

In the test UI, you should see:
- ✅ **Platform Support**: Supported
- ✅ **Virtual Device Detection**: "BlackHole 2ch" detected
- 📋 **All Available Devices**: List including "BlackHole 2ch"

---

### 5. Switch to System Audio

1. Find **"BlackHole 2ch"** in the device list
2. Click the **"Use"** button next to it
3. **Current Audio Source** should change to: `system:BlackHole 2ch`

---

### 6. Start Test Recording

In the **"Test Recording"** section:

1. Click **"Start Test Recording"** (green button)
   - You should see: 🔴 **RECORDING** indicator
   - Buffer counter starts at **0 samples**

---

### 7. Play YouTube Video

1. Open your browser
2. Go to YouTube (e.g., https://www.youtube.com/watch?v=dQw4w9WgXcQ)
3. Play a video
4. **You should hear the audio** (because of Multi-Output Device setup)
5. Watch the buffer counter in the app **increasing in real-time**:
   ```
   16,000 samples (1.0s)
   32,000 samples (2.0s)
   48,000 samples (3.0s)
   ...
   ```

**This proves audio is being captured!** 🎉

---

### 8. Stop & Save Recording

1. After 5-10 seconds, click **"Stop & Save Recording"** (red button)
2. You'll see an alert with:
   ```
   Recording saved!

   Location: /Users/YOUR_NAME/Library/Application Support/com.handy.app/recordings/test_recording_2025-11-04T12-34-56.wav

   Samples: 80,000 (5.0s)
   ```

---

### 9. Verify the Recording

**Option 1: Quick play in terminal**
```bash
# Get the file path from the alert
cd ~/Library/Application\ Support/com.handy.app/recordings/
ls -lh
# Play the most recent file
afplay test_recording_*.wav
```

**Option 2: Open in Finder**
```bash
open ~/Library/Application\ Support/com.handy.app/recordings/
```

Double-click the `.wav` file to play it in QuickTime.

**You should hear the YouTube audio you just recorded!** ✅

---

## Troubleshooting

### Buffer stays at 0 samples

**Problem**: Recording button is pressed, but buffer doesn't increase.

**Solutions**:
1. Check that **Current Audio Source** shows `system:BlackHole 2ch` (not `microphone`)
2. Verify YouTube audio is playing (check volume)
3. Restart the app and try again
4. Check Console logs (in terminal running the app) for errors

---

### Can't hear YouTube audio

**Problem**: Audio is silent when playing YouTube.

**Solution**: You forgot to set up Multi-Output Device!
1. Go back to Step 1
2. Make sure both BlackHole AND your speakers are checked
3. Select Multi-Output Device as system output

---

### "Use" button is disabled

**Problem**: Can't click "Use" next to BlackHole.

**Solution**: That device is already active! Check **Current Audio Source** - it should show `system:BlackHole 2ch`.

---

## What About Other Applications?

This works for **ANY audio output**:

### Discord Call
1. Set Discord audio output to Multi-Output Device
2. Switch Handy to BlackHole
3. Start recording
4. Join Discord voice channel
5. Records the conversation ✅

### Zoom Call
1. Zoom → Preferences → Audio → Speaker → Multi-Output Device
2. Switch Handy to BlackHole
3. Start recording
4. Join Zoom call
5. Records the call ✅

### Spotify, Podcasts, etc.
Same process - it captures **any** system audio!

---

## Current Limitations (Will be fixed in Phase 1 Steps 2-5)

❌ **No transcription yet** - Only saves raw WAV audio
❌ **No automatic meeting mode** - Manual start/stop only
❌ **No speaker diarization** - Can't distinguish speakers yet
❌ **No live transcript view** - Can't see text in real-time
❌ **Only remote audio** - Your microphone voice not included

These will all be implemented in the next steps!

---

## File Locations

**Recordings**: `~/Library/Application Support/com.handy.app/recordings/`

Each file is named with a timestamp:
```
test_recording_2025-11-04T12-34-56-789Z.wav
```

**Format**:
- 1 channel (mono)
- 16,000 Hz sample rate
- 32-bit float samples
- Standard WAV format

You can play these files in:
- QuickTime Player
- VLC
- Audacity (for waveform analysis)
- Any audio player

---

## Next Steps

Once you've verified recording works:

1. **Step 2**: Build MeetingManager (orchestrate meeting lifecycle)
2. **Step 3**: Connect to TranscriptionManager (audio → text)
3. **Step 4**: Add transcript storage (save JSON + Markdown)
4. **Step 5**: Build Meeting UI (start/stop, live view)

Then you'll have full meeting recording + transcription! 🚀
