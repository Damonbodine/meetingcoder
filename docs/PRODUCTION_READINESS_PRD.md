# Production Readiness PRD — Next Phase

Owner: Damon Bodine  
Repo: Handy (Tauri app)  
Date: 2025-11-11  
Status: Draft for implementation

## 1. Overview

Goal: Harden Handy for reliable, privacy‑conscious, team‑wide use in live meetings (Google Meet, Zoom, etc.). This phase focuses on reliability, resource control, UX clarity, and transcription quality with pragmatic scope.

Primary risks to address:
- Memory/CPU spikes during long sessions (system audio buffering, resampling, batch size). 
- Audio device fragility (virtual device setup, hot‑plug, drift). 
- Poor runtime transparency (no in‑app diagnostics or clear failure states). 
- Basic transcription params and diarization not exposed → inconsistent quality.

## 2. Goals / Non‑Goals

Goals
- Stable, low‑resource continuous recording (2–4 hour sessions). 
- Fast, configurable transcription with clear model lifecycle. 
- Clear UX for setup, status, and error recovery. 
- Basic compliance posture: consent, privacy, data handling clarity.

Non‑Goals
- Full cloud backend and multi‑tenant admin. 
- Perfect diarization or ASR parity across all languages. 
- Enterprise SSO, centralized policy enforcement.

## 3. Personas & Use Cases

Personas
- IC Engineer: records team meetings, wants low friction and fast summaries. 
- Manager: records standups/retros, expects stability and correct export formats. 
- Admin: validates privacy/consent and installation steps.

Use Cases
- Live record Meet/Zoom calls with system audio capture. 
- Continuous rolling transcript (5–10s chunks), optional summarization. 
- Export transcript (Markdown, JSONL, SRT/VTT). 
- Recover from device changes/sleep without data loss.

## 4. Scope (This Phase)

Reliability & Performance
- Replace shared `Mutex<Vec<f32>>` with a bounded ring buffer (SPSC, overwrite oldest). 
- Reduce chunk size to 5–10s; add system‑audio silence gating (VAD/energy threshold). 
- Tune resampler to lower CPU (lighter filter or alternative). 
- Add device restart hardening and drift checks.

UX & Diagnostics
- Diagnostics panel: buffer size/age, trims, restart attempts, CPU (approx), model state, last errors. 
- Guided audio setup wizard (virtual device verify + 5s test). 
- Clear status indicators and toasts for restarts/failures.

Transcription Quality
- Expose ASR params in settings (language, greedy/beam, temperature if available). 
- Short‑chunk partials option (interim text) for responsiveness (non‑blocking if time‑boxed). 
- Basic diarization plan: keep placeholder labels now; integrate real diarization in next cycle.

Privacy & Compliance
- Consent reminder banner; toggleable. 
- Clarify on‑device vs. cloud summarization (opt‑in). 
- Data location and retention settings; export paths.

## 5. Requirements

### 5.1 Audio Capture & Buffering
- R1: System audio buffer is a bounded SPSC ring buffer sized for 20–60 seconds; overwrites oldest frames. 
- R2: Buffer exposes non‑blocking read for N seconds (5–10s) with zero‑copy where possible. 
- R3: Silence gating for system audio (energy threshold, configurable; default conservative). 
- R4: Resampler runs under a 10% CPU budget on a 4‑core laptop during a call; choose lighter filter or alternative library. 
- R5: Automatic restart logic rate‑limited; emits user‑visible notifications and logs.

### 5.2 Transcription
- R6: Settings for language auto/fixed, greedy vs. beam (if supported), temperature, and translate‑to‑English. 
- R7: Chunk size configurable, default 5–10s; option for partial (interim) transcripts. 
- R8: Custom words post‑correction retained; expose threshold in settings.

### 5.3 UX & Diagnostics
- R9: Diagnostics panel shows: capture source/device, sample rate detected, buffer usage (% and seconds), trim count, restart attempts, last 10 errors, model load state/time. 
- R10: Setup wizard validates virtual device presence and performs a 5s loopback test with RMS/peak and sample rate detection. 
- R11: Clear persistent status area with recording state, audio health, and model state. 
- R12: Users can export transcript in Markdown, JSONL, and SRT/VTT.

### 5.4 Privacy & Compliance
- R13: Consent reminder and indicator while recording; document recommended policy text. 
- R14: Summarization: explicit on‑device vs. cloud; off by default. 
- R15: Local data paths visible; retention setting (keep/delete on stop) with confirmation.

## 6. Acceptance Criteria

Reliability
- A1: 2‑hour soak test on macOS and Windows: 
  - Peak RSS < 1.2 GB, delta growth < 200 MB after first 10 minutes. 
  - No unhandled panics; 0 fatal crashes. 
  - Audio buffer never exceeds configured seconds; trims logged ≥ 1 per 10 minutes if consumer lags.
- A2: Sleep/wake and device hot‑plug are handled; app resumes or guides user.

Performance
- A3: CPU average < 25% on mid‑range laptop during capture+transcription (Whisper small/medium quant). 
- A4: Resampler < 10% CPU on average; no glitching.

Quality & UX
- A5: With default settings, WER on an internal 10‑minute English sample ≤ baseline target for chosen model (document baseline). 
- A6: Setup wizard completes in < 90 seconds and confirms usable device. 
- A7: Diagnostics panel surfaces real‑time indicators; restart events show a toast and a log line.

Privacy
- A8: Consent banner enabled by default for first‑time use; cloud summarization off by default; data paths clearly shown.

## 7. Metrics & Instrumentation

- Buffer: current size (samples/seconds), trim count, max observed size. 
- Audio restarts: attempts, success/failure, last error. 
- Model lifecycle: load/unload counts, load time ms, active duration. 
- CPU snapshot (coarse): periodic 1s AVG per main worker threads (if feasible cross‑platform). 
- Transcription latency per chunk; partial availability time (if enabled). 
- Errors: categorized counts; last N messages.

All metrics stay local; optional export to a local JSONL for debugging.

## 8. Milestones & Timeline

Phase A (Week 1–2): Reliability & Performance
- Implement ring buffer; shorten chunks to 5–10s; system‑audio silence gating. 
- Tune resampler; add buffer trim logging; tighten restart backoff. 
- Soak tests (60/120 minutes) on macOS + Windows.

Phase B (Week 3): Diagnostics & Setup
- Diagnostics panel and status area. 
- Guided audio setup wizard + loopback test.

Phase C (Week 4): Transcription Quality & Exports
- Expose Whisper/Parakeet params; add SRT/VTT/JSONL exports. 
- Optional: interim partial transcript path behind a flag.

Phase D (Week 5): Privacy & Release Prep
- Consent banner, retention controls, summarization defaults. 
- Packaging/signing checklist (macOS notarization, Windows signing); auto‑update plan.

## 9. Work Breakdown (Epics → Stories)

E1: Audio Buffering & Resampling
- S1: Introduce SPSC ring buffer with overwrite window (configurable seconds). 
- S2: Migrate producer/consumer to ring buffer; remove `Mutex<Vec<f32>>`. 
- S3: Implement energy‑based silence gating (threshold in settings). 
- S4: Resampler tuning; benchmark and document CPU on a sample machine. 
- S5: Drift monitoring: log device sample rate and resampling ratio; warn on anomalies.

E2: Transcription Controls
- S6: Settings for language, greedy/beam, temperature, translate flag. 
- S7: Chunk size toggle 5/10/20/30s (default 10s). 
- S8: Partial transcripts option (if time allows); event stream to UI.

E3: Diagnostics & Setup
- S9: Diagnostics view with metrics and error log. 
- S10: Status bar indicators (recording, audio health, model). 
- S11: Setup wizard for virtual device validation + 5s test with RMS/peak and text result.

E4: Exports & Data
- S12: Export transcript as Markdown, JSONL, SRT/VTT (with timestamps). 
- S13: Retention setting (delete on stop vs. keep) with confirmation.

E5: Privacy & Packaging
- S14: Consent banner and policy hint; cloud summarization off by default. 
- S15: Packaging/signing doc + scripts; auto‑update plan documented.

## 10. Risks & Mitigations

- Cross‑platform audio quirks: Validate with vendor devices; provide manual override. 
- CPU spikes from resampler/ASR: Add lower‑cost modes; expose presets. 
- User confusion on audio routing: Setup wizard and safe‑defaults button. 
- Long‑session memory creep: Ring buffer hard cap; segment rotation; periodic sanity checks.

## 11. Test Plan

Automated
- Unit tests for ring buffer (wrap/overwrite semantics). 
- Integration: 5–10s chunk flow, restart path, export formats. 
- Snapshot tests for settings and diagnostics rendering.

Manual/Soak
- 60/120‑minute Meet recordings on macOS and Windows; log memory/CPU each minute. 
- Sleep/wake, device unplug/replug; verify auto‑recovery or clear prompts. 
- Setup wizard negative paths (no virtual device, permission denied). 
- Exports open in common players (VLC for SRT/VTT).

## 12. Acceptance Review Checklist

- [ ] A1–A8 criteria met on both macOS and Windows. 
- [ ] Documentation updated (Setup, Privacy, Exports, Troubleshooting). 
- [ ] Packaging/signing verified; safe defaults confirmed. 
- [ ] Known issues captured with mitigations.

## 13. Open Questions

- Which diarization approach next: on‑device lightweight vs. cloud post‑hoc? 
- Minimum supported OS versions and CPU architecture (Intel vs. Apple Silicon tuning)? 
- Team policy for consent banner defaults and storage retention?

---

Appendix: Engineering Notes
- Current patch capped system audio buffer growth and optimized reads. This PRD replaces the buffer with a bounded ring to eliminate contention and reduce copies. 
- Keep logs local by default; provide a one‑click bundle for bug reports (redact paths/PII).
