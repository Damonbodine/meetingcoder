# MeetingCoder Internal Production Readiness Plan

This document captures the concrete work required to make the internal MeetingCoder desktop build reliable for day-to-day team usage. It is distilled from the recent repo review and organized by focus area so we can assign owners and track progress.

## 1. Scope Guardrails

- **Clarify personas:** Ship a “Recorder” experience by default (live transcription, import, summaries). Gate GitHub/Claude/automation/IDE launchers behind an *Advanced* toggle with clear warnings so casual users do not need extra permissions.
- **Offline vs. Connected:** Add a real offline mode that disables Claude, GitHub, YouTube import, and any network calls. Update README/onboarding to explain which features depend on external services.

## 2. Packaging & Distribution

- **Finish `BUILD.md`:** Document Bun vs. npm usage, Tauri prerequisites per OS, model/VAD asset download, and how to run `bun tauri build` for each platform.
- **Release pipeline:** Define macOS codesign/notarization steps, update `tauri.conf.json` with real IDs, and document Windows signing. Capture how `latest.json` for the updater is produced and published.
- **External tools:** Provide guided setup or bundling for `yt-dlp` and `ffmpeg`, with pre-flight validation before enabling the import UI.
- **Repo hygiene:** Remove large binaries (`audio1466401210.m4a`, committed `node_modules`) from git history; store samples in Releases or object storage.

## 3. Security & Compliance

- **Secret storage:** Remove plaintext fallbacks in `~/.handy`; rely on OS keychains only. If a fallback is unavoidable, encrypt and make it opt-in.
- **GitHub auth:** Stop embedding PATs in clone URLs. Prefer the existing device-flow UI or require users to configure their own credential helper.
- **Automation safety:** Require explicit confirmation before auto-creating branches, running commits, or triggering PR creation. Add platform checks before exposing AppleScript automation buttons.
- **Privacy UI:** Add a consent reminder/indicator, retention controls (keep vs. delete on stop), and toggles that clearly state when cloud summarization/GitHub upload is active.

## 4. Reliability & Quality

- **Speaker accuracy:** Replace the current two-speaker toggle diarization with either a lightweight diarizer or manual labeling tools in the UI.
- **Transcript editing:** Finish the `TranscriptEditor` flow so imported meetings can be corrected and saved back to disk.
- **Storage hygiene:** Implement cleanup for `audio_segments/` and queue leftovers after a meeting finishes, and rotate/truncate `.meeting-updates.jsonl`.
- **Model delivery:** Host Whisper/Parakeet assets on a managed bucket with checksums; verify downloads before activation.
- **Performance instrumentation:** Expand Diagnostics to include long-session memory, queue backlog trends, and expose a log bundle for support.

## 5. Documentation & Testing

- **Brand alignment:** Update architecture/security docs, `ProjectInitializer`, and helper scripts to say “MeetingCoder” (not Handy) and to point to the correct default folders.
- **Setup guides:** Create explicit routing instructions for Windows (WASAPI loopback) and Linux (Pulse/PipeWire) similar to the macOS BlackHole guide.
- **Automated checks:** Add CI for `bun run build`, `bunx tsc --noEmit`, and `cargo test`. Capture a manual QA checklist for 2–4 hour soak tests, permissions, updater validation, and import flows.

## 6. Ownership & Next Actions

| Area | Owner | Notes |
| --- | --- | --- |
| Build/release docs | TBD | Extend `BUILD.md`, script asset downloads. |
| Security passes | TBD | Secrets, GitHub auth, automation confirmations. |
| UX/Consent | TBD | Offline toggle, consent UI, Advanced gating. |
| Storage cleanup | TBD | Audio segments + queue lifecycle, JSONL rotation. |
| Testing/CI | TBD | Add GitHub Actions/Bun + Cargo checks, document soak tests. |

> Once each bucket has an owner, cut issues with concrete acceptance criteria so we can track them alongside existing PRD phases.

