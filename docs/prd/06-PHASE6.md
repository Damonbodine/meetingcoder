# Phase 6: Import Audio & YouTube as Offline Meetings

> Adds the ability to import pre-recorded audio or a YouTube URL and process them as MeetingCoder sessions (transcript, meeting updates, and optional automation) without live capture.

## Summary

- Users can:
  - Import a local audio file (wav/mp3/m4a/ogg/flac) with a meeting name.
  - Import a YouTube URL with a meeting name (requires `yt-dlp`).
- The app transcribes in chunks with existing models (Whisper/Parakeet), writes transcript files, and appends `.meeting-updates.jsonl` entries so downstream automation (/meeting) remains compatible.
- Optional GitHub integration is honored (Phase 4): updates append in the attached repo.

## Dependencies & Compatibility

- Depends on:
  - Phase 1: Continuous transcription stack (model management, chunk size setting, transcript storage)
  - Phase 2: Summarization agent and `.meeting-updates.jsonl` format
  - Phase 3: Automation trigger (optional)
  - Phase 4: GitHub repo attach/create (optional)
- No conflicts with:
  - Phase 5 (Discovery Mode): This phase complements live paths by enabling offline ingestion. Both can coexist.
  - Existing transcript storage and update formats are reused as-is (no schema changes).

## Objectives

- Import pre-recorded audio to produce transcript and updates identical to live sessions.
- Support YouTube ingestion via audio download for parity with file import.
- Keep the system private/local except for optional YouTube download.

## User Stories

- Select an audio file, name the meeting, and receive a transcript and updates in the Meeting Updates panel.
- Paste a YouTube URL, provide the meeting name, and process it into transcript and updates.
- If a repo is attached (Phase 4), updates append in the project and /meeting can be triggered automatically if enabled (Phase 3).

## Scope

- New UI section in Meetings view: “Import Audio into MeetingCoder”.
  - File picker using system dialog.
  - Input for YouTube URL and Import button.
- Backend commands:
  - `import_audio_as_meeting(meeting_name, file_path) -> MeetingSummary`
  - `import_youtube_as_meeting(meeting_name, url) -> MeetingSummary`
- Audio loader utility:
  - Decode common formats, downmix to mono, resample to 16kHz.
- Offline meeting entrypoint:
  - Create meeting session without starting the live system-audio loop; reuse chunk transcription pipeline and storage.
- Summarization + updates:
  - Append `.meeting-updates.jsonl` with `source = "import:file" | "import:youtube"`.
  - Optionally trigger automation.

## Non-Goals

- Speaker diarization / multi-speaker labeling.
- Streaming progress UI (can be added later as a progress event stream).
- Official YouTube captions import (future enhancement).

## UX Notes

- Minimal friction: one section at top of Meetings view with meeting name, “Choose Audio File…”, and YouTube URL + Import.
- Toast notifications for start/success/failure; progress indicator optional.

## Security & Privacy

- File paths originate from OS picker; no arbitrary path traversal inputs.
- Audio decoding is memory-capped to prevent runaway allocations on malformed files.
- Transcription runs locally; YouTube requires `yt-dlp` and network access.

## Settings Interactions

- `transcription_chunk_seconds` respected (2–60s; default from Phase 1).
- `selected_model`, language/translation flags, and model unload strategy.
- GitHub settings (Phase 4) determine project path and update destination.

## Testing Strategy

- File import:
  - WAV and MP3 samples: verify transcript.json/md saved; updates appear; optional automation triggers.
  - Large file: ensure chunking works; no OOM; transcript saved.
- YouTube import:
  - With `yt-dlp` installed and network on: verify download + transcription + updates.
  - Missing `yt-dlp`/no network: error and recovery messaging.
- Regression:
  - Live meeting flows still work; no interference with system audio capture.

## Acceptance Criteria

- Import file flow completes with a saved transcript and at least one update appended; UI shows success.
- Import YouTube flow works with `yt-dlp` available; otherwise shows an actionable error.
- Meeting Updates panel reflects appended records; /meeting can be triggered if enabled.
- No regressions in Phases 1–5 behaviors.

## Risks & Mitigations

- Format incompatibility: Document supported formats; guide users to convert if needed.
- Large/corrupt audio: Chunked processing and caps; handle errors gracefully.
- `yt-dlp` availability: Provide clear install guidance and fallback to file import.

## Future Enhancements

- Progress events with estimated time remaining; cancelation support.
- Diarization; or pass-through of per-track speaker info if available.
- Native caption ingestion for YouTube as a fast, no-transcribe path.

## Implementation Pointers

- Use existing meeting storage and summarization/automation pipeline to keep behavior consistent with live sessions.
- Reuse settings and managers (Model, Transcription, Meeting, Context Writer) to avoid duplication.
- Maintain `.meeting-updates.jsonl` schema compatibility so downstream tooling remains unchanged.

