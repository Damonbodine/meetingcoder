# MeetingCoder

> **Transform meetings into working code.**
> Desktop app for real-time meeting transcription with system audio capture and AI-powered code generation. Built with Tauri (Rust + React/TypeScript).

---

## Credit & relationship to Handy

**MeetingCoder is built on top of [Handy](https://github.com/cjpais/Handy)** by [@cjpais](https://github.com/cjpais), with all of Handy's core speech-to-text infrastructure preserved. If you want a privacy-focused, offline speech-to-text utility, **use Handy** — it's the canonical project.

This repository extends Handy with meeting-centric features: persistent meeting history, GitHub repo integration, AI-driven summarization, PRD generation from transcripts, and audio import. The MeetingCoder layer is research-grade — Handy is the production-grade thing.

If you contribute improvements to the underlying transcription/audio pipeline, please send the PR to [`cjpais/Handy`](https://github.com/cjpais/Handy) so the broader community benefits.

---

## What MeetingCoder adds on top of Handy

| Capability | Where it lives |
|---|---|
| **Meeting history & persistent transcripts** | `src/components/Sidebar.tsx`, `src/components/onboarding/`, on-disk `.meeting-updates.jsonl` |
| **System audio capture** (record both mic and speaker output) | `SYSTEM_AUDIO_IMPLEMENTATION.md` documents the approach |
| **Audio file & YouTube URL import** | Backed by `yt-dlp` (must be on PATH for YouTube) |
| **AI summarization of transcripts** | `summarization/` modules |
| **Document generation (PRDs from meetings)** | `document_generation/prd_generator/` |
| **Codebase awareness** (link a transcript to a repo for context) | `codebase/` Rust modules |
| **GitHub integration** (link meetings to repos / issues) | Consolidated GitHub integration commit |
| **Automation rules** that fire on meeting events | `.meeting-updates.jsonl` event stream |

For a granular changelog of the MeetingCoder additions specifically, see [CHANGELOG.md](./CHANGELOG.md) and the `feat: complete MeetingCoder implementation` commits in the history.

---

## Quick start

> ⚠️ **First-time setup is non-trivial.** Read [BUILD.md](./BUILD.md) before attempting. macOS Sonoma users may hit a CMake version error — the workaround is in BUILD.md.

```bash
# Prerequisites: Rust (stable), Bun
bun install

# Download the required VAD model (one-time)
mkdir -p src-tauri/resources/models
curl -o src-tauri/resources/models/silero_vad_v4.onnx \
  https://blob.handy.computer/silero_vad_v4.onnx

# Run in dev mode
bun run tauri dev
# If CMake errors on macOS:
CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev

# Build for production
bun run tauri build
```

For YouTube import, install `yt-dlp`:
```bash
brew install yt-dlp     # macOS
```

---

## Architecture (inherited from Handy)

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │    Backend      │    │   ML Models     │
│  React + TS     │◄──►│   Rust + Tauri  │◄──►│ Whisper / VAD   │
│   Vite          │    │   Audio I/O     │    │   Silero        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

**Core libraries (from Handy):**
- `whisper-rs` — local Whisper inference (GPU when available)
- `transcription-rs` — Parakeet V3 (CPU-optimized)
- `cpal` — cross-platform audio I/O
- `vad-rs` — voice activity detection
- `rdev` — global keyboard shortcuts
- `rubato` — audio resampling

**MeetingCoder additions:**
- `summarization/` — meeting summary agent
- `document_generation/` — PRD generation pipeline
- `codebase/` — repo indexing for context-aware code generation

---

## How it works

1. **Press** the configurable shortcut to start recording (or use push-to-talk)
2. **Speak** through the meeting; both mic and system audio are captured
3. **Release** — Whisper transcribes locally, no audio leaves your machine
4. **Review** the transcript in the meeting history view
5. **Generate** — kick off summarization, PRD generation, or code suggestions tied to a linked repo

The transcription stage is entirely local (Handy's design). The MeetingCoder generation features call out to LLM APIs you configure.

---

## Platform support

Inherited from Handy:
- ✅ **macOS** (Intel + Apple Silicon)
- ✅ **x64 Windows**
- ⚠️ **Linux** — community-supported

System audio capture is platform-specific; see [SYSTEM_AUDIO_IMPLEMENTATION.md](./SYSTEM_AUDIO_IMPLEMENTATION.md) for current state.

---

## Status & maintenance

This is a **research project**, not a maintained product. It demonstrates a particular architecture (local STT + cloud LLM hybrid for meeting workflows) but I am not actively shipping releases or accepting bug reports here.

If you want a production-ready local STT tool, **[install Handy](https://handy.computer)** instead.

If you want to extend MeetingCoder for your own use, fork freely under the existing license.

---

## License

Same as Handy — see [LICENSE](./LICENSE).

---

## Links

- **Handy upstream** (the foundation this is built on): [github.com/cjpais/Handy](https://github.com/cjpais/Handy) · [handy.computer](https://handy.computer)
- **Damon Bodine** (MeetingCoder author): [damonbodine.vercel.app](https://damonbodine.vercel.app) · [GitHub](https://github.com/Damonbodine) · [LinkedIn](https://linkedin.com/in/damonbodine)
