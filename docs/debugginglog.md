# Debugging Log — Phase 6: Import Audio/YouTube (Offline)

Date: 2025-11-06
Owners: Phase 6 agents
Scope: Offline import of local audio and YouTube into MeetingCoder sessions

## Summary

- User imports a 35-minute Zoom M4A (AAC in MP4 container).
- Transcript produced does not fully cover the conversation and seems to miss words.
- Initial truncation was due to decoder caps; we added a Symphonia path (MP4/M4A/AAC) and raised caps.
- Current run shows full decode via rodio (Symphonia fell back due to unknown channels) and long processing, but still perceived gaps.
- We added longer chunks + overlap and then upgraded to VAD-based segmentation for imports to improve context.
- summary.md now generated; UI shows Saved-to path + Open buttons.

## Repro

1) Launch
- cd Handy && bun run tauri dev

2) Import
- Meetings → "Import Audio into MeetingCoder"
- Enter name (e.g., "Brijay")
- Browse… and select one of:
  - Preferred sample for repro: `/Users/damonbodine/speechtotext/Handy/audio1466401210.m4a`
  - Or your Zoom file path if testing your own meeting
- Click Import

3) Observe
- Progress shows decoding → transcribing → finalizing
- On completion: toast with segments; Import card shows Saved-to path with Open Folder / Open Transcript / Open Summary

## Expected vs Observed

Expected
- Full coverage of 35 minutes
- No material gaps at segment boundaries
- Zoom-like summary in summary.md

Observed
- Initial builds truncated around ~8–9 minutes due to decoder cap (fixed)
- Current builds decode full stream (Symphonia when channel metadata present; otherwise rodio fallback). Coverage still feels incomplete with boundary words occasionally clipped.

## Key Logs (user-provided)

- Symphonia unknown channels → fallback to rodio:
  - WARN … audio::loader Symphonia decode failed … Unknown channels — falling back to rodio
- Symphonia successful path (when channel metadata available):
  - INFO … Decoded with symphonia: … (samples=NNNNNNNN)
- Rodio decoded entire audio:
  - INFO … Decoded with rodio: … (interleaved_samples=202049536)
- Chunk processing prints:
  - "Audio vector length: 320000" — 20s @ 16k mono
- Meeting end:
  - Ended meeting: Brijay - Duration: 372s, Segments: 101 (processing runtime, not audio duration)

## What’s Implemented (current branch)

- Decode & resample
  - Symphonia first for m4a/mp4/aac; fall back to rodio if probe/decoder fails
  - Rodio cap raised (now 300M interleaved samples) with decode logging
  - Mono downmix + 16k resample
- Segmentation
  - Imports use VAD-based segmentation (Silero + smoothing) to cut at natural pauses
  - Fallback to fixed windows (20–60s) with 0.8s overlap
- Model
  - Parakeet/Whisper both supported; model auto-load waits up to 30s
- Updates & outputs
  - .meeting-updates.jsonl updates with source labels (import:file|import:youtube)
  - transcript.json, transcript.md
  - summary.md (Key Points, Decisions, Questions)
- UI
  - Native file picker (Rust plugin) and YouTube input
  - Progress events (import-progress)
  - On complete: Saved-to row with Open Folder / Open Transcript / Open Summary buttons

## Hypotheses for Remaining Gaps

1) VAD segmentation settings
- Threshold or smoothing too aggressive → dropping quiet speech or trimming tails
- Large blocks split but boundaries still mid-word in crosstalk

2) Engine differences
- Parakeet is fast; Whisper-medium/large may recover more words in noisy/crosstalk conditions

3) Channel/format edge case
- Symphonia reported unknown channels for this file; rodio decoded PCM path is fine, but we may miss codec/channel metadata handling that could help (e.g., defaulting channel layout)

4) Overlap/dedupe
- Overlap helps catch boundary words, but no downstream dedupe can produce duplicates or omit borderline words in rare cases

## Instrumentation Added / Needed

Added
- Decode path logging: which decoder, sample counts
- Mono sample count + approx minutes after decode
- Segmentation coverage: per-segment start/end/duration (debug), summary count/mean/median + last_end vs total
- VAD summary (count, mean, median) and slightly more permissive threshold (0.4) with longer hangover
- Simple overlap-aware dedupe at segment joins (trim duplicate prefix in next segment)

Needed (optional)
- Optional: log first/last 30s text to confirm tails are present

## Action Plan (next agent)

1) Add coverage instrumentation
- import.rs (after decode):
  - log!("mono_samples={}, approx_minutes={}", mono.len(), mono.len() as f64 / 16000.0 / 60.0);
- After building segment list:
  - log start/end seconds per segment; sum coverage. Ensure last end ~ total_seconds

2) Tune VAD parameters
- Threshold: try 0.35–0.45 (currently 0.5)
- Smoothing: adjust prefill=8–12, hangover=12–16, onset=3–4
- Cap segments at ~45–60s; avoid huge segments that slow inference

3) Add overlap-aware dedupe
- On merging back transcripts, trim leading/trailing repeated short substrings (e.g., 5–10 tokens) between adjacent segments

4) Offer Whisper for imports
- Implemented: setting “Prefer Whisper for imports” (Advanced Settings)
- Behavior: on import, load the best downloaded Whisper model by accuracy; otherwise fall back to your selected model

5) Symphonia channel handling
- For `Unknown channels`, derive channel count from track metadata or default to 2 if absent, then continue Symphonia path instead of fallback
- Alternative: update Symphonia features or try `symphonia-bundle-*` crates if needed

6) Double-check fixed segmentation fallback
- If VAD yields too many tiny segments or none, ensure fixed window fallback is invoked and logs indicate that path

## Acceptance Checks

- Decode duration ~ 35 min (mono_samples ≈ 33.6M). For sample file at `/Users/damonbodine/speechtotext/Handy/audio1466401210.m4a`, confirm logs show `Import decode: mono_samples=..., approx_minutes=...` and either `Decoded with symphonia` or `Decoded with rodio` with expected sample magnitude.
- Segments cover almost full duration (last end ≥ total_seconds - 1)
- Subjective review: boundary words preserved; summary.md reflects correct themes
- UI buttons open the folder/transcripts successfully

## File Pointers

- Decode + resample: src-tauri/src/audio_toolkit/audio/loader.rs
- Import flow: src-tauri/src/commands/import.rs
- VAD: src-tauri/src/audio_toolkit/vad/*
- Meeting manager (save outputs): src-tauri/src/managers/meeting.rs
- Frontend Import UI: src/components/meeting/ImportAudio.tsx

## Known Constraints

- Large files require memory to hold decoded PCM; no streaming decoder yet
- Parakeet vs Whisper accuracy tradeoffs; Whisper may be slower but better in crosstalk
- Symphonia channel metadata edge-case seen on this Zoom file

## Suggested Next Commits (pseudo)

- Add instrumentation:
  - log total mono samples + computed minutes after decode
  - log segment ranges and total coverage
- Tweak VAD: threshold 0.4, hangover 12, onset 3
- Add simple dedupe at segment joins
- Optional toggle: prefer Whisper for imports

## Appendix: Example Logs of a “Good” Run (desired)

- INFO loader: Decoded with symphonia: … samples=67,200,000 (mono_samples=33,600,000 ≈ 35.0 min)
- INFO import: Using VAD segmentation: segments=128, mean_len=16.4s
- INFO import: coverage=2099.7s (target=2100.2s), last_end=2099.9s
- INFO meeting: transcript saved: …/2025-11-06_title
- INFO meeting: summary.md written
