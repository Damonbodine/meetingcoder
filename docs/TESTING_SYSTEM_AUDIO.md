# Testing System Audio Capture

This guide will help you test the newly implemented system audio capture functionality.

## Prerequisites

Before testing, you need to install a virtual audio device on macOS:

### Option 1: BlackHole (Recommended for Testing)
1. Download from: https://existential.audio/blackhole/
2. Install "BlackHole 2ch" (the 2-channel version)
3. After installation, it will appear in your audio devices

### Option 2: Loopback by Rogue Amoeba (Professional Option)
1. Download from: https://rogueamoeba.com/loopback/
2. Free trial available for testing
3. More features but not required for basic testing

## Running the Test Interface

### Step 1: Build and Run the App
```bash
cd /Users/damonbodine/speechtotext/Handy
bun run tauri dev
```

If you get a cmake error on macOS:
```bash
CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev
```

### Step 2: Enable Debug Mode
Once the app is running:
1. Press **Cmd+Shift+D** (macOS) or **Ctrl+Shift+D** (Windows/Linux)
2. This toggles debug mode and reveals the "Debug" section in the sidebar

### Step 3: Access System Audio Test
1. Click the "Debug" section in the left sidebar (Flask icon)
2. Scroll down to the "System Audio Testing" section
3. You should see the test interface load automatically

## What the Test Interface Shows

The test interface will display:

### 1. Platform Support ✅
- Shows whether system audio capture is supported on your platform
- Should show "Supported" on macOS, Windows, and Linux

### 2. Virtual Device Detection 🔍
- Auto-detects installed virtual audio devices
- **With BlackHole installed**: Should show BlackHole 2ch device details
- **Without virtual device**: Shows "Not Found" with installation instructions

### 3. All Available Devices 📋
- Lists all audio input devices detected on your system
- Shows device name, sample rate, and channel count
- BlackHole should appear in this list if installed

### 4. Setup Instructions 📖
- Platform-specific instructions for configuring virtual audio
- Copy this for future reference

### 5. Test Summary 📊
- Quick overview of which tests passed
- All checkmarks = everything working correctly!

## Expected Results

### ✅ Successful Test (with BlackHole installed)
You should see:
- ✅ Platform support: **Supported**
- ✅ Virtual device: **Detected** (BlackHole 2ch)
- ✅ Device details:
  - Name: BlackHole 2ch
  - Sample Rate: 48000 Hz or 44100 Hz
  - Channels: 2
- ✅ All devices list shows 2+ devices (including BlackHole)
- ✅ Setup instructions displayed

### ⚠️ Partial Success (without BlackHole)
You should see:
- ✅ Platform support: **Supported**
- ⚠️ Virtual device: **Not Found**
- ✅ All devices list shows your microphone(s)
- ✅ Setup instructions displayed with installation steps

## Testing Actions

### Quick Test Checklist
1. [ ] App starts in dev mode
2. [ ] Debug mode toggles with Cmd+Shift+D
3. [ ] System Audio Testing section appears
4. [ ] Platform support shows "Supported"
5. [ ] Device detection runs without errors
6. [ ] If BlackHole installed: device is detected
7. [ ] All devices list populates
8. [ ] Setup instructions load
9. [ ] "Refresh Tests" button works
10. [ ] No console errors in terminal

### Testing Device Detection
1. **Without BlackHole**: Should show "Not Found"
2. **Install BlackHole**: Download and install from link above
3. **Refresh Tests**: Click the "Refresh Tests" button
4. **Verify Detection**: Should now show BlackHole device

### Testing on Different Platforms
If you have access to other platforms:

**Windows**:
- Should show WASAPI loopback available
- No virtual device required!
- Should list all output devices

**Linux**:
- Should detect PulseAudio/PipeWire monitor sources
- May show multiple monitor devices

## Troubleshooting

### "Platform not supported"
- Check that you're running on macOS, Windows, or Linux
- Verify the Rust code compiled correctly

### "Failed to detect device" error
- Check terminal/console for detailed error messages
- Ensure audio permissions are granted (macOS System Preferences > Security & Privacy > Microphone)
- Try restarting the app

### BlackHole installed but not detected
- Verify installation: Open "Audio MIDI Setup" app (macOS)
- Look for "BlackHole 2ch" in device list
- If present, click "Refresh Tests" in the test UI
- May need to restart the app

### No devices listed
- Check audio permissions
- Verify cpal can access audio devices
- Look for errors in terminal output

## Next Steps After Testing

Once the test interface shows everything working:

1. **Report Results**: Note which tests passed/failed
2. **Check Console**: Look for any Rust warnings/errors
3. **Screenshot Results**: Capture the test summary for reference

### What Works Now
- ✅ System audio device detection
- ✅ Device enumeration
- ✅ Platform support checking
- ✅ Setup instruction generation

### What's Next (Not Yet Implemented)
- ⏳ Actually capturing audio from devices
- ⏳ Integration with meeting mode
- ⏳ Continuous recording
- ⏳ Audio chunking and transcription

## Questions to Answer While Testing

1. Does device detection work correctly?
2. Are all your audio devices listed?
3. Do the setup instructions make sense?
4. Any errors in the terminal?
5. Does the UI respond smoothly?

## Reporting Issues

If you encounter issues, please provide:
- Platform (macOS version, etc.)
- Whether BlackHole is installed
- Screenshot of the test interface
- Terminal error messages (if any)
- Which tests passed/failed

---

**Happy Testing! 🚀**

The system audio capture foundation is complete. This test interface verifies that all the backend commands work correctly before we build the full meeting capture feature.
