# MeetingCoder

**A free, open source, and extensible speech-to-text application that works completely offline.**

> **Based on [Handy](https://github.com/cjpais/handy)** - forked and enhanced for meeting transcription and coding workflows.

MeetingCoder is a cross-platform desktop application built with Tauri (Rust + React/TypeScript) that provides simple, privacy-focused speech transcription. Press a shortcut, speak, and have your words appear in any text field—all without sending your voice to the cloud.

## Why MeetingCoder?

MeetingCoder builds on the foundation of [Handy](https://github.com/cjpais/handy), an open source speech-to-text tool, with enhanced features for meetings and coding:

- **Free**: Accessibility tooling belongs in everyone's hands, not behind a paywall
- **Open Source**: Together we can build further. Extend MeetingCoder for yourself and contribute to something bigger
- **Private**: Your voice stays on your computer. Get transcriptions without sending audio to the cloud
- **Meeting-Focused**: Import audio files and YouTube videos for meeting transcription and analysis
- **Simple**: One tool, focused on transcription and meeting workflows

## How It Works

1. **Press** a configurable keyboard shortcut to start/stop recording (or use push-to-talk mode)
2. **Speak** your words while the shortcut is active
3. **Release** and MeetingCoder processes your speech using Whisper
4. **Get** your transcribed text pasted directly into whatever app you're using

The process is entirely local:
- Silence is filtered using VAD (Voice Activity Detection) with Silero
- Transcription uses your choice of models:
  - **Whisper models** (Small/Medium/Turbo/Large) with GPU acceleration when available
  - **Parakeet V3** - CPU-optimized model with excellent performance and automatic language detection
- Works on Windows, macOS, and Linux

## Modes & Privacy Guardrails

MeetingCoder now ships with two first-class personas so casual users don't have to worry about automation surprises:

- **Recorder Mode (default):** Focuses on live transcription, local audio imports, and summaries. GitHub automation, `/meeting` triggers, and IDE launchers stay hidden.
- **Advanced Automations:** Flip the toggle in *Settings → General → Modes* to unlock GitHub pushes, Claude-powered summaries, and other scripted workflows. Every action is gated with confirmations and status indicators.

You can also enable **Offline Mode** in the same section. While offline we block all network calls—Claude, GitHub, and YouTube import are paused until you opt back in—so regulated environments can keep data air-gapped.

## Quick Start

### Installation

1. Download the latest release from the [releases page](https://github.com/Damonbodine/meetingcoder/releases)
2. Install the application following platform-specific instructions
3. Launch MeetingCoder and grant necessary system permissions (microphone, accessibility)
4. Configure your preferred keyboard shortcuts in Settings
5. Start transcribing!

### Development Setup

For detailed build instructions including platform-specific requirements, see [BUILD.md](BUILD.md).

### Import Audio/YouTube into MeetingCoder

- Import a local file: Open Meetings → "Import Audio into MeetingCoder", enter a meeting name, and choose an audio file (wav/mp3/m4a/ogg/flac).
- Import a YouTube URL: Paste the URL and click Import. MeetingCoder now runs a pre-flight check to confirm both `yt-dlp` and `ffmpeg` are available before enabling the button.
- The app creates a transcript and appends `.meeting-updates.jsonl`; automation can run if enabled.
- Need a sample clip? Download `audio1466401210.m4a` from the repository Releases page—the large file is no longer checked into git.

## External Dependencies

Most of MeetingCoder works completely offline, but two optional helpers make imports reliable:

- **ffmpeg** – used to normalize imported recordings.
  - macOS: `brew install ffmpeg`
  - Windows: `winget install Gyan.FFmpeg`
  - Ubuntu/Debian: `sudo apt install ffmpeg`
- **yt-dlp** – required only for YouTube imports.
  - macOS: `brew install yt-dlp`
  - Windows: `winget install yt-dlp.yt-dlp`
  - Linux: `pipx install yt-dlp`

The Transcription view shows the current status of these tools and offers a one-click re-check so the UI stays in sync with your PATH.

## Architecture

MeetingCoder is built as a Tauri application combining:

- **Frontend**: React + TypeScript with Tailwind CSS for the settings UI
- **Backend**: Rust for system integration, audio processing, and ML inference
- **Core Libraries**:
  - `whisper-rs`: Local speech recognition with Whisper models
  - `transcription-rs`: CPU-optimized speech recognition with Parakeet models
  - `cpal`: Cross-platform audio I/O
  - `vad-rs`: Voice Activity Detection
  - `rdev`: Global keyboard shortcuts and system events
  - `rubato`: Audio resampling

### Debug Mode

MeetingCoder includes an advanced debug mode for development and troubleshooting. Access it by pressing:
- **macOS**: `Cmd+Shift+D`
- **Windows/Linux**: `Ctrl+Shift+D`

## Known Issues & Current Limitations

This project is actively being developed and has some [known issues](https://github.com/Damonbodine/meetingcoder/issues). We believe in transparency about the current state:

### Platform Support
- **macOS (both Intel and Apple Silicon)**
- **x64 Windows**
- **x64 Linux**

### System Requirements/Recommendations

The following are recommendations for running MeetingCoder on your own machine. If you don't meet the system requirements, the performance of the application may be degraded. We are working on improving the performance across all kinds of computers and hardware.

**For Whisper Models:**
- **macOS**: M series Mac, Intel Mac
- **Windows**: Intel, AMD, or NVIDIA GPU
- **Linux**: Intel, AMD, or NVIDIA GPU
  * Ubuntu 22.04, 24.04

**For Parakeet V3 Model:**
- **CPU-only operation** - runs on a wide variety of hardware
- **Minimum**: Intel Skylake (6th gen) or equivalent AMD processors
- **Performance**: ~5x real-time speed on mid-range hardware (tested on i5)
- **Automatic language detection** - no manual language selection required

### How to Contribute

1. **Check existing issues** at [github.com/Damonbodine/meetingcoder/issues](https://github.com/Damonbodine/meetingcoder/issues)
2. **Fork the repository** and create a feature branch
3. **Test thoroughly** on your target platform
4. **Submit a pull request** with clear description of changes

The goal is to create both a useful tool and a foundation for others to build upon—a well-patterned, simple codebase that serves the community.

## Related Projects

- **[Handy](https://github.com/cjpais/handy)** - The original project this is based on
- **[Handy CLI](https://github.com/cjpais/handy-cli)** - Python command-line version

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- **[Handy](https://github.com/cjpais/handy)** by cjpais - the foundation of this project
- **Whisper** by OpenAI for the speech recognition model
- **whisper.cpp and ggml** for amazing cross-platform whisper inference/acceleration
- **Silero** for great lightweight VAD
- **Tauri** team for the excellent Rust-based app framework
- **Community contributors** helping make open source speech-to-text better

---

*"Your search for the right speech-to-text tool can end here—not because MeetingCoder is perfect, but because you can make it perfect for you."*
## Project Phases

- Phase 1: System audio capture and continuous transcription foundations
- Phase 6: Import audio and YouTube into MeetingCoder (see `docs/prd/06-PHASE6.md`)
