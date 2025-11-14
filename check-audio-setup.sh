#!/bin/bash

echo "===== CURRENT AUDIO SETUP ====="
echo ""

echo "📢 Current System Output Device:"
system_profiler SPAudioDataType 2>/dev/null | grep -A 3 "Default Output Device: Yes" | grep -E "(Name:|Manufacturer:|Transport:)"
echo ""

echo "🎧 Available Audio Devices:"
system_profiler SPAudioDataType 2>/dev/null | grep -E "^\s+[A-Za-z].*:" | grep -v "Devices:" | head -10
echo ""

echo "🔍 BlackHole Status:"
if system_profiler SPAudioDataType 2>/dev/null | grep -q "BlackHole"; then
    echo "✅ BlackHole is installed"
    system_profiler SPAudioDataType 2>/dev/null | grep -A 2 "BlackHole"
else
    echo "❌ BlackHole not found"
fi
echo ""

echo "🔊 Multi-Output Device:"
if system_profiler SPAudioDataType 2>/dev/null | grep -q "Multi-Output"; then
    echo "✅ Multi-Output Device exists"
else
    echo "❌ No Multi-Output Device found"
    echo "   👉 You need to create one in Audio MIDI Setup"
fi
echo ""

echo "===== WHAT YOU NEED TO DO ====="
echo ""
echo "For system audio capture to work:"
echo "1. Create Multi-Output Device in Audio MIDI Setup"
echo "2. Include both BlackHole 2ch AND your speakers"
echo "3. Set it as system output (right-click → Use This Device For Sound Output)"
echo "4. Then audio will go through BOTH BlackHole (for recording) and speakers (for hearing)"
echo ""
echo "Free volume control tip: install Background Music (brew install --cask background-music) to keep"
echo "BlackHole selected while still adjusting Zoom/system output levels with per-app sliders."
