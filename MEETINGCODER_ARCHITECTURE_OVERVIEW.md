# MeetingCoder Application - Comprehensive Architecture Overview

## Executive Summary

MeetingCoder is a sophisticated Tauri-based desktop application that transforms stakeholder meetings into working code through continuous transcription, AI-powered requirement extraction, and automated code generation via Claude Code CLI. The application is built on top of the Handy speech-to-text foundation and adds meeting management, LLM integration, GitHub automation, and agentic processing capabilities.

**Current Version**: 0.5.4
**Status**: Phase 7 (Intelligent Real-Time Code Generation) - 39% Complete
**Architecture**: Rust backend + React/TypeScript frontend with Tauri framework

---

## Part 1: Current Architecture & Main Components

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│          Frontend (React/TypeScript)                │
│  Settings │ Meeting │ Meetings │ History │ Import   │
└─────────────────────┬───────────────────────────────┘
                      │ Tauri Commands/Events
┌─────────────────────┴───────────────────────────────┐
│          Backend (Rust - src-tauri/src)             │
│                                                     │
│  ┌────────────────────────────────────────────┐   │
│  │         Manager Layer (Coordination)        │   │
│  │  Audio │ Transcription │ Meeting │ Model   │   │
│  └────────────────────────────────────────────┘   │
│                                                     │
│  ┌────────────────────────────────────────────┐   │
│  │     Processing Services                    │   │
│  │  VAD │ Whisper/Parakeet │ Audio Queue      │   │
│  │  ASR Worker │ Summarization Agent          │   │
│  │  GitHub Integration │ Claude API           │   │
│  └────────────────────────────────────────────┘   │
│                                                     │
│  ┌────────────────────────────────────────────┐   │
│  │     Integrations                            │   │
│  │  GitHub API │ Claude API │ Automation      │   │
│  │  Codebase Analysis │ File Isolation        │   │
│  └────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
              │               │               │
        GitHub API      Claude API      System Audio
        (GitHub.com)    (Anthropic)       (Zoom/Meet)
```

### 1.2 Core Components Breakdown

#### Backend Managers (Rust)

**Location**: `/src-tauri/src/managers/`

1. **AudioRecordingManager** (`audio.rs`) - 25KB
   - Continuous system audio capture (platform-specific)
   - Supports microphone and system audio sources
   - Voice Activity Detection (VAD) using Silero VAD
   - Audio device enumeration and switching
   - Ring buffer for audio streaming
   - Resampling to 16kHz for transcription

2. **TranscriptionManager** (`transcription.rs`) - 16KB
   - Manages Whisper/Parakeet model lifecycle
   - Model loading/unloading with idle timeout
   - Transcription with confidence scores
   - Automatic model unloading on inactivity
   - Supports both Whisper (GPU-accelerated) and Parakeet (CPU-optimized)

3. **MeetingManager** (`meeting.rs`) - 71KB (LARGEST)
   - Orchestrates meeting lifecycle (start, pause, resume, end)
   - Maintains active meetings with transcript accumulation
   - Manages speaker identification and diarization
   - Coordinates audio capture and transcription workers
   - Triggers summarization and GitHub integration
   - Auto-saves transcripts to `.meeting-updates.jsonl`
   - Handles project initialization and context

4. **ModelManager** (`model.rs`) - 25KB
   - Downloads and caches Whisper models
   - Manages model directories and versions
   - Tracks available models and active selection
   - Handles model dependency resolution

5. **HistoryManager** (`history.rs`) - 11KB
   - SQLite-based transcription history persistence
   - Tracks all transcriptions with metadata
   - Implements retention policies

#### Core Processing Pipeline

**Location**: `/src-tauri/src/`

1. **Queue System** (`queue.rs`)
   - SQLite-backed audio queue for durable processing
   - FIFO ordering for chunks
   - Status tracking (queued → processing → done)
   - Automatic retry on failure
   - WAL mode for concurrent access

2. **ASR Worker** (`workers/asr_worker.rs`)
   - Worker pool spawned on startup (configurable count: 1-8)
   - Fetches items from queue continuously
   - Loads WAV audio chunks
   - Performs transcription with Whisper/Parakeet
   - Applies speaker diarization (simple toggle logic)
   - Emits segment-added events to frontend
   - Non-blocking processing

3. **Audio Processing** (`audio_toolkit/`)
   - Device detection and enumeration
   - Audio recording via CPAL (cross-platform)
   - Audio file loading and resampling
   - VAD with Silero ONNX model
   - Visualization for audio levels
   - System audio capture (platform-specific):
     - macOS: Core Audio
     - Windows: WASAPI
     - Linux: PulseAudio

#### Agentic & Summarization

**Location**: `/src-tauri/src/`

1. **Summarization Agent** (`summarization/agent.rs`)
   - Heuristic-based feature extraction from transcripts
   - Detects: features, technical decisions, questions
   - Priority inference (High/Medium/Low)
   - Deduplication against prior updates
   - Structured output: `SummarizationOutput` with rich metadata
   - Input: array of `TranscriptSegment`
   - Output: JSON appended to `.meeting-updates.jsonl`

2. **LLM Integration** (`summarization/llm.rs`)
   - Claude API integration for advanced summarization
   - Secure API key storage (keyring + fallback)
   - Structured JSON extraction from Claude
   - Intelligent fallback to heuristic agent
   - Settings: `use_llm_summarization`, `llm_model`
   - Commands: `store_claude_api_key`, `has_claude_api_key`, `delete_claude_api_key`

#### Automation & Integration

**Location**: `/src-tauri/src/`

1. **Claude Trigger** (`automation/claude_trigger.rs`)
   - Executes `/meeting` command in Claude Code
   - AppleScript-based automation (macOS)
   - Debounce protection (minimum 30-600 seconds between triggers)
   - Automation state tracking (`.claude/.automation-state.json`)
   - Command injection protection

2. **GitHub Integration** (`integrations/github.rs`)
   - OAuth Device Flow for authentication
   - Repository listing and selection
   - Branch creation and management
   - Commit and push operations
   - Pull request creation and updates
   - PR comment posting with summaries
   - State management (`.claude/.github-state.json`)

3. **Codebase Analysis** (`codebase/analyzer.rs` & `codebase/isolation.rs`)
   - Framework detection (Next.js, React, Django, Rails, Tauri, FastAPI, Flask)
   - Language detection (TypeScript, Python, Rust, Go, Java, Ruby, PHP, Swift)
   - Dependency extraction (package.json, requirements.txt, Cargo.toml)
   - Directory mapping and entry point discovery
   - File isolation with `.claudeignore` generation
   - Safe path validation for Claude Code integration
   - Experiments folder isolation for experimental code

#### Storage & Persistence

**Location**: `/src-tauri/src/storage/`

1. **TranscriptStorage** (`transcript/`)
   - Saves transcripts with metadata
   - JSONL format for incremental updates
   - Metadata tracking (duration, participants, timestamps)
   - Context writer for append-only protocol

---

## Part 2: Live Transcription Features

### 2.1 Meeting Transcription Flow

```
Meeting Start
    ↓
System Audio Capture (CPAL)
    ↓
30-60s Audio Chunks
    ↓
Voice Activity Detection (Silero VAD)
    ↓
Queue: SQLite Audio Queue
    ↓
ASR Worker Pool (1-8 workers)
    ↓
Transcription (Whisper/Parakeet)
    ↓
Speaker Diarization (Toggle-based)
    ↓
TranscriptSegment (with confidence, timestamps)
    ↓
MeetingManager accumulation
    ↓
Frontend Event: transcript-segment-added
    ↓
UI Update + Summarization trigger
```

### 2.2 Continuous Transcription Support

**Key Features**:
- ✅ Real-time transcript display with speaker labels
- ✅ Configurable chunk duration (2-60 seconds, default 10s)
- ✅ Update intervals for summarization (default 20s)
- ✅ Live speaker diarization (2-speaker toggle model)
- ✅ Confidence scores per segment
- ✅ Timestamp tracking (start_time, end_time in seconds from meeting start)
- ✅ Speaker relabeling UI (rename Speaker 1 → "Alice")
- ✅ Pause/Resume capability
- ✅ Auto-save to `.meeting-updates.jsonl`

### 2.3 Transcription Settings

**Frontend Controls** (`src/components/settings/`):
- `ChunkDuration.tsx`: 2-60 seconds (default 10)
- `UpdateInterval.tsx`: 5-300 seconds (default 20)
- `SystemAudioTest.tsx`: Test system audio capture
- `PreferWhisperForImports.tsx`: Choose Whisper vs Parakeet for audio imports
- `FastImportMode.tsx`: Speed vs quality tradeoff
- `MinSegmentDuration.tsx`: Filter very short segments
- `UseFixedWindowsForImports.tsx`: Fixed vs sliding window chunking

---

## Part 3: The `/meeting` Command Implementation

### 3.1 What `/meeting` Does

The `/meeting` command is a custom Claude Code slash command that:

**Purpose**: Ingest real-time meeting updates and generate/update code incrementally

**Located**: `Handy/src-tauri/templates/meeting_command.md`

**Flow**:
1. Claude Code session reads `.meeting-updates.jsonl`
2. Parses new features, technical decisions, questions
3. Updates codebase with new implementations
4. Generates code for discussed features
5. Returns changes for user review/acceptance

### 3.2 Command Template Structure

**Template Location**: `src-tauri/templates/meeting_command.md`

**Key Sections**:
1. **Mode Detection** - Reads `.claude/.meeting-state.json` to determine:
   - Developer Mode: Working on existing repository
   - Starter Kit Mode: Creating new project from scratch

2. **Developer Mode** (For existing codebases):
   - Analyzes existing codebase structure
   - Detects framework and tech stack
   - Routes changes to `experiments/{meeting_id}/` folder
   - Applies `.claudeignore` safety rules
   - Generates working code without breaking existing code

3. **Starter Kit Mode** (For new projects):
   - Scaffolds complete project (Vercel + Supabase + Next.js)
   - Generates all necessary files
   - Sets up development environment
   - Creates deployment configuration

4. **Context Feeding**:
   - Reads incremental feature updates from `.meeting-updates.jsonl`
   - Deduplicates features across updates
   - Tracks update IDs to avoid re-processing

5. **Build Validation**:
   - Runs TypeScript checks
   - Executes npm build
   - Validates code syntax
   - Auto-retries on failure

### 3.3 Data Format for `/meeting` Command

**Input File**: `.meeting-updates.jsonl` (append-only JSONL)

**Per-Update Schema**:
```json
{
  "timestamp": "2025-11-11T12:00:00Z",
  "update_id": 1,
  "segment_range": [0, 25],
  "meeting_id": "uuid",
  "meeting_name": "Feature Discussion",
  "project_type": "next.js-web-app",
  "tech_stack": ["Next.js", "React", "TypeScript", "Supabase"],
  
  "new_features": ["Add user authentication", "Create dashboard"],
  "new_features_structured": [
    {
      "id": "f1234567890abc",
      "title": "User Authentication",
      "description": "OAuth with GitHub",
      "priority": "high",
      "technical_notes": ["Use NextAuth.js", "Store in Supabase"]
    }
  ],
  "modified_features": { "f_old_id": {"priority": "medium"} },
  "clarifications": { "f_xyz": "User mentioned it needs email too" },
  "target_files": ["app/auth/route.ts", "components/LoginForm.tsx"],
  
  "technical_decisions": ["Use Supabase", "Next.js API routes"],
  "questions": ["Should auth support SAML?"],
  
  "segment_count": 15
}
```

---

## Part 4: Current AI Integrations

### 4.1 LLM Providers

**Integrated**:
- ✅ **Claude API** (Anthropic)
  - Used for: Advanced summarization, feature extraction
  - Settings: API key storage (keyring + fallback)
  - Commands: `store_claude_api_key`, `has_claude_api_key`, `delete_claude_api_key`
  - Model: Claude 3.5 Sonnet (configurable)

**Planned**:
- ❌ OpenAI API (Phase 2 design mentions, not yet implemented)
- ❌ Ollama (local LLM support, not yet implemented)

### 4.2 Transcription Models

**Implemented**:
1. **Whisper Models** (OpenAI)
   - Variants: Small, Medium, Turbo, Large
   - GPU acceleration (Metal on macOS, Vulkan on Windows/Linux)
   - High accuracy, larger models
   - Model: `transcribe-rs` (Rust binding)

2. **Parakeet V3** (NVIDIA)
   - CPU-only operation
   - Automatic language detection
   - ~5x real-time speed on mid-range hardware
   - Lightweight, good for low-resource machines

### 4.3 Current AI Processing Features

**Summarization Agent** (Heuristic-based):
- Keyword matching for feature extraction
- Priority inference from language ("must", "should", "nice to have")
- Question detection (sentences ending with "?")
- Technical decision detection
- Deduplication via hash-based IDs

**LLM-Based Summarization** (Optional, fallback):
- Calls Claude API if enabled
- Structured JSON extraction
- Priority scoring with confidence
- Falls back to heuristic on failure
- Integrated into meeting manager

---

## Part 5: Meeting Notes & Transcript Storage

### 5.1 Storage Structure

**Base Directory**: `~/.handy/meetings/{meeting_id}/`

**Files Created**:
```
meetings/{meeting_id}/
├── transcript.jsonl          # Raw transcript segments (JSONL)
├── metadata.json             # Meeting metadata
├── .meeting-updates.jsonl    # Incremental updates (append-only)
├── .claude/
│   ├── meeting-state.json    # Mode, project type, stack
│   ├── .automation-state.json # Last trigger time/update_id
│   ├── .github-state.json     # GitHub branch, PR info
│   ├── .claudeignore          # File safety rules (if Developer Mode)
│   └── /meeting               # Slash command template (copy from template)
├── experiments/
│   └── {meeting_id}/          # Developer Mode: safe workspace
│       ├── src/
│       ├── tests/
│       └── docs/
└── project/ (if GitHub enabled)
    └── files...               # Actual code project
```

### 5.2 Transcript Format

**Raw Transcript** (`transcript.jsonl`):
```json
{"speaker":"Speaker 1","start_time":0.5,"end_time":5.2,"text":"We need user auth","confidence":0.95,"timestamp":"2025-11-11T12:00:00Z"}
{"speaker":"Speaker 2","start_time":5.3,"end_time":8.1,"text":"Yes, with OAuth","confidence":0.93,"timestamp":"2025-11-11T12:00:08Z"}
```

**Metadata** (`metadata.json`):
```json
{
  "id": "meeting-uuid",
  "name": "Feature Planning Session",
  "start_time": "2025-11-11T12:00:00Z",
  "end_time": "2025-11-11T12:45:00Z",
  "duration_seconds": 2700,
  "participants": ["Speaker 1", "Speaker 2"],
  "total_segments": 47,
  "transcript_file": "transcript.jsonl"
}
```

### 5.3 Updates Protocol

**Purpose**: Incremental updates fed to Claude Code

**Format**: Append-only JSONL (`.meeting-updates.jsonl`)

**Trigger**: Every 20 seconds (configurable via `meeting_update_interval_seconds`)

**Deduplication**:
- Hash-based feature IDs to prevent duplicates
- Tracks `update_id` to avoid re-processing
- Compares against prior 1000 lines

**Update Contents**:
- New features with IDs and priorities
- Modified features (partial updates)
- Clarifications from participants
- Target files mentioned in conversation
- Technical decisions and questions

---

## Part 6: Agentic & Autonomous Processing Features

### 6.1 Existing Agentic Features

**Level 1: Continuous Summarization Agent** ✅
- Runs automatically every 20 seconds during active meetings
- Extracts features, decisions, questions from new transcript segments
- Outputs structured JSON to `.meeting-updates.jsonl`
- Updates are deduplicated and enriched with metadata

**Level 2: Automated `/meeting` Command Triggering** ⚠️ Partial
- **Implemented**:
  - `trigger_meeting_update()` function in `automation/claude_trigger.rs`
  - Debounce protection (30-600 second minimum intervals)
  - Automation state tracking across restarts
  - AppleScript execution (macOS only)

- **Not Fully Integrated**:
  - ❌ Real-time automation loop (Phase 3 feature)
  - ❌ Auto-accept mechanism for changes
  - ❌ Full error recovery and retry logic

**Level 3: GitHub Automation** ⚠️ Partial
- **Implemented**:
  - OAuth Device Flow for authentication
  - Repository selection and branch creation
  - Commit and push to GitHub
  - PR creation with meeting summaries
  - PR comment updates with incremental changes

- **Not Fully Integrated**:
  - ❌ Automatic PR updates on each meeting update
  - ❌ Workflow triggering
  - ❌ Review assignment

**Level 4: Codebase Analysis & Isolation** ✅
- Framework detection (Next.js, Django, Rails, etc.)
- Language detection
- Dependency extraction
- `.claudeignore` generation for safe file routing
- `experiments/{meeting_id}/` isolation for experimental code
- Safe path validation to prevent file overwrites

### 6.2 Autonomous Processing Pipeline (Phase 7)

**Target Architecture** (In Progress):

```
Meeting Start
    ↓
Codebase Analysis (auto) → .meeting-state.json
    ↓
Generate .claudeignore (auto) → File isolation
    ↓
Initialize /meeting template (auto)
    ↓
LOOP every 20 seconds:
    ├─ Transcription: audio → segments
    ├─ Summarization: segments → .meeting-updates.jsonl
    ├─ Check trigger condition (interval + debounce)
    ├─ If trigger:
    │   ├─ Open Claude Code
    │   ├─ Execute /meeting command
    │   ├─ Wait for changes
    │   ├─ Auto-accept (AppleScript)
    │   ├─ Push to GitHub (if enabled)
    │   └─ Post PR comment with update
    └─ End loop
    ↓
Meeting End
    ↓
Final code review + merge
```

### 6.3 Current Automation Settings

**Frontend Controls** (`src/components/settings/`):
- `AutoTriggerToggle.tsx`: Enable/disable auto-triggering
- `AutoAcceptChanges.tsx`: Auto-send 'y' to accept changes
- `AutomationDebounce.tsx`: Minimum interval between triggers (30-600s, default 60s)
- `UpdateInterval.tsx`: Summarization interval (5-300s, default 20s)

**Backend Settings** (`settings.rs`):
```rust
pub struct Settings {
    pub auto_trigger_meeting_command: bool,        // Enable automation
    pub auto_accept_changes: bool,                 // Auto-accept changes
    pub auto_trigger_min_interval_seconds: u64,    // Debounce (30-600s)
    pub meeting_update_interval_seconds: u64,      // Summarization (5-300s)
}
```

### 6.4 Areas for Enhanced Agentic Processing

**Gap 1: Real-Time Streaming Orchestration**
- Current: Updates accumulate until manual trigger
- Needed: Continuous autonomous triggering with proper debounce
- Status: Partially implemented in `claude_trigger.rs`

**Gap 2: Context Window Management**
- Current: All segments in `.meeting-updates.jsonl`
- Needed: Sliding window to stay within Claude's context limits
- Opportunity: Implement summarization of older updates

**Gap 3: Error Recovery**
- Current: No automatic retry on failed triggers
- Needed: Exponential backoff, fallback to manual trigger
- Opportunity: Track failure states in automation-state.json

**Gap 4: Multi-Stage Code Review**
- Current: User must manually review Claude Code changes
- Needed: Automated linting, testing, validation before acceptance
- Opportunity: Add pre-accept validation stage

**Gap 5: Collaborative Participant Detection**
- Current: Simple 2-speaker toggle
- Needed: True speaker diarization with participant names
- Opportunity: Use pyannote.audio or other models

**Gap 6: Feature Tracking & Coverage**
- Current: Each update is independent
- Needed: Track which features are covered, what's missing
- Opportunity: Build coverage matrix against requirements

**Gap 7: Context Injection from Codebase**
- Current: Only high-level analysis (framework, languages)
- Needed: Inject relevant code snippets for contextual generation
- Opportunity: Semantic search + embeddings for relevant code selection

---

## Part 7: Overall Flow from Meeting Audio to Code/Documentation

### 7.1 Complete End-to-End Flow

```
┌─────────────────────────────────────────────────────────┐
│ Step 1: MEETING SETUP (User Action)                     │
│ - Click "Start Meeting" in UI                           │
│ - Enter meeting name                                    │
│ - Select audio source (system audio or microphone)      │
└─────────────────────┬───────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────┐
│ Step 2: INITIALIZATION (Automatic)                      │
│ - MeetingManager.start_meeting()                        │
│ - Create ~/.handy/meetings/{meeting_id}/                │
│ - Codebase analysis (if Developer Mode)                 │
│ - Generate .claudeignore (if Developer Mode)            │
│ - Create .claude/ scaffolding                           │
│ - Initialize git repository                            │
│ - Copy /meeting template                                │
│ - Set meeting state to .claude/meeting-state.json       │
└─────────────────────┬───────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────┐
│ Step 3: CONTINUOUS AUDIO CAPTURE (Background)           │
│ - AudioRecordingManager starts system audio capture     │
│ - CPAL reads audio from selected source                 │
│ - 30-60 second chunks accumulated                       │
│ - VAD (Silero) filters silence                          │
└─────────────────────┬───────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────┐
│ Step 4: CHUNKING & QUEUING (Background)                 │
│ - Audio chunks → Queue (SQLite)                         │
│ - Each chunk: start_ms, end_ms, file_path               │
│ - Queue persists across app restarts                    │
└─────────────────────┬───────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────┐
│ Step 5: TRANSCRIPTION (Worker Pool)                     │
│ - ASR Workers (1-8) fetch queue items                   │
│ - Load audio chunk (16kHz mono WAV)                     │
│ - Whisper or Parakeet model transcription               │
│ - Output: text with confidence score                    │
│ - Emit: transcript-segment-added event                  │
└─────────────────────┬───────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────┐
│ Step 6: SPEAKER DIARIZATION (ASR Worker)                │
│ - Detect turn boundaries via silence threshold          │
│ - Toggle speaker label (Speaker 1 ↔ Speaker 2)         │
│ - Create TranscriptSegment with speaker label           │
└─────────────────────┬───────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────┐
│ Step 7: MEETING ACCUMULATION (ASR Worker)               │
│ - Add segment to MeetingManager                         │
│ - Save to transcript.jsonl                              │
│ - Emit UI event (toast + update)                        │
│ - Update metadata.json                                  │
└─────────────────────┬───────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────┐
│ Step 8: REAL-TIME DISPLAY (Frontend)                    │
│ - Listen to transcript-segment-added events             │
│ - Update LiveTranscript component                       │
│ - Show speaker label + text + timestamp                 │
│ - Auto-scroll to latest                                 │
│ - User can relabel speakers                             │
└─────────────────────┬───────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────┐
│ Step 9: PERIODIC SUMMARIZATION (Every 20s)              │
│ - MeetingManager.summarize_new_segments()               │
│ - Collect segments since last update                    │
│ - Call heuristic agent or LLM summarizer                │
│ - Extract: features, decisions, questions               │
│ - Append to .meeting-updates.jsonl                      │
│ - Update update_id counter                              │
└─────────────────────┬───────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────┐
│ Step 10: AUTOMATION TRIGGER CHECK (Every 20s)           │
│ - If auto_trigger_meeting_command enabled               │
│ - Check debounce interval (30-600s)                     │
│ - Check if new updates exist                            │
│ - If conditions met:                                    │
│   ├─ Open Claude Code session                           │
│   ├─ Execute /meeting slash command                     │
│   ├─ Claude Code reads .meeting-updates.jsonl           │
│   ├─ Claude generates/updates code                      │
│   └─ If auto_accept_changes, send 'y' + Enter          │
└─────────────────────┬───────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────┐
│ Step 11: CODE GENERATION (Claude Code)                  │
│ - /meeting command template processes updates           │
│ - Determines mode (Developer vs Starter Kit)            │
│ - Analyzes codebase for context                         │
│ - Routes changes to safe locations (.claudeignore)      │
│ - Generates code for discussed features                 │
│ - Validates syntax and runs build                       │
│ - Returns diff for review                               │
└─────────────────────┬───────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────┐
│ Step 12: GITHUB INTEGRATION (After Acceptance)          │
│ - If GitHub enabled:                                    │
│   ├─ Create or checkout feature branch                  │
│   ├─ Commit changes with meeting context                │
│   ├─ Push to remote                                     │
│   ├─ Create or update pull request                      │
│   └─ Post PR comment with meeting summary               │
└─────────────────────┬───────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────┐
│ Step 13: MEETING END (User Action)                      │
│ - Click "End Meeting"                                   │
│ - Finalize transcript                                   │
│ - Generate final summary                                │
│ - Save meeting to history                               │
│ - Return MeetingSummary                                 │
└─────────────────────────────────────────────────────────┘
```

### 7.2 Time Distribution in Pipeline

**Typical Meeting (60 minutes)**:
- Audio capture: 60 min (continuous)
- Transcription: ~10-15 min (real-time with workers)
- Summarization: ~2-3 min (incremental every 20s)
- Claude Code execution: ~5-10 min per trigger (if auto-triggered every 2-3 min)
- GitHub integration: ~1 min per PR update

**Total latency (speech → code)**:
- Speech capture: ~10 seconds
- Transcription: ~5-10 seconds
- Summarization: ~1 second
- Claude Code trigger: ~30-60 seconds
- **End-to-end**: ~45-90 seconds (speech spoken → code written)

---

## Part 8: Technology Stack & Dependencies

### 8.1 Core Framework
- **Tauri** 2.9.1 - Desktop app framework (Rust + React)
- **Rust** (latest stable) - Backend
- **React** 18.3.1 - Frontend UI
- **TypeScript** 5.6.3 - Type-safe frontend

### 8.2 Audio & Transcription
- **cpal** 0.16.0 - Cross-platform audio I/O
- **hound** 3.5.1 - WAV file reading/writing
- **rubato** 0.16.2 - Audio resampling
- **symphonia** 0.5 - Audio format decoding (mp3, flac, aac, vorbis)
- **transcribe-rs** 0.1.4 - Whisper/Parakeet binding
- **vad-rs** (git) - Voice Activity Detection
- **rodio** (git fork) - Audio playback

### 8.3 AI & Processing
- **reqwest** 0.11.27 - HTTP client (Claude API calls)
- **futures-util** 0.3 - Async utilities
- **rustfft** 6.4.0 - FFT for audio analysis

### 8.4 System Integration
- **rdev** (git) - Global keyboard shortcuts
- **enigo** 0.6.1 - Keyboard/mouse automation
- **git2** 0.19 - Git operations
- **keyring** 3.2 - Secure credential storage

### 8.5 Storage & Persistence
- **rusqlite** 0.32.1 - SQLite database
- **serde** + **serde_json** - JSON serialization
- **chrono** 0.4 - DateTime handling
- **dirs** 5.0 - OS-specific paths
- **walkdir** 2.5 - Directory traversal
- **toml** 0.8 - Config file parsing

### 8.6 Frontend Libraries
- **zustand** 5.0.8 - State management
- **zod** 3.25.76 - Schema validation
- **tailwindcss** 4.1.16 - CSS framework
- **lucide-react** 0.542.0 - Icon library
- **sonner** 2.0.7 - Toast notifications

### 8.7 Tauri Plugins
- `tauri-plugin-autostart` - App autostart
- `tauri-plugin-clipboard-manager` - Clipboard access
- `tauri-plugin-dialog` - File dialogs
- `tauri-plugin-fs` - File system
- `tauri-plugin-global-shortcut` - Keyboard shortcuts
- `tauri-plugin-opener` - Open files/URLs
- `tauri-plugin-process` - subprocess
- `tauri-plugin-sql` - SQLite integration
- `tauri-plugin-store` - Key-value storage
- `tauri-plugin-updater` - App updates
- `tauri-plugin-macos-permissions` - macOS permissions
- `tauri-plugin-single-instance` - Single app instance

---

## Part 9: Configuration & Settings

### 9.1 Key Settings

**Audio/Transcription**:
- `microphone_mode`: Push-to-talk vs Always-on
- `selected_microphone`: Device selection
- `selected_output_device`: Output device
- `model_unload_timeout`: Idle unload timing
- `chunk_duration`: 2-60 seconds
- `meeting_update_interval_seconds`: 5-300 seconds

**Meeting/Automation**:
- `auto_trigger_meeting_command`: Enable automation
- `auto_accept_changes`: Auto-accept Claude Code changes
- `auto_trigger_min_interval_seconds`: Debounce (30-600s)
- `system_audio_silence_threshold`: VAD threshold
- `system_audio_buffer_seconds`: Ring buffer size

**GitHub Integration**:
- `github_enabled`: Enable GitHub features
- `github_repo_owner`: Repository owner
- `github_repo_name`: Repository name
- `github_default_branch`: Base branch for PRs
- `github_branch_pattern`: Branch naming (e.g., "meeting/{meeting_id}")

**Import/Export**:
- `prefer_whisper_for_imports`: Use Whisper for audio imports
- `fast_import_mode`: Speed vs quality tradeoff
- `use_fixed_windows_for_imports`: Fixed vs sliding window
- `min_segment_duration_for_imports`: Filter short segments

**Summarization**:
- `use_llm_summarization`: Use Claude API vs heuristic
- `llm_model`: Claude model version

---

## Part 10: Key Entry Points & Command Surface

### 10.1 Frontend Components

**Main Meeting UI**:
- `src/App.tsx` - Main app entry
- `src/components/meeting/MeetingView.tsx` - Meeting controls & display
  - `MeetingControls.tsx` - Start/stop/pause/resume
  - `LiveTranscript.tsx` - Real-time transcript display
  - `MeetingUpdates.tsx` - Shows summarized updates
  - `MeetingChecklist.tsx` - Feature tracking
  - `AudioSetupSection.tsx` - Audio device selection

**Settings Pages**:
- `src/components/settings/` - 40+ setting components
- `src/components/settings/IntegrationsSettings.tsx` - GitHub + Claude API
- `src/components/settings/GitHubOAuth.tsx` - OAuth Device Flow UI

**History**:
- `src/components/settings/HistorySettings.tsx` - List saved meetings
- `src/components/transcription/TranscriptionView.tsx` - View meeting transcripts

### 10.2 Tauri Commands (Frontend ↔ Backend RPC)

**Meeting Commands**:
- `start_meeting(name)` → meeting_id
- `end_meeting(meeting_id)` → MeetingSummary
- `pause_meeting(meeting_id)`
- `resume_meeting(meeting_id)`
- `get_live_transcript(meeting_id)` → Vec<TranscriptSegment>
- `get_active_meetings()` → Vec<String>
- `get_meeting_info(meeting_id)` → MeetingInfo

**Import Commands**:
- `import_audio_as_meeting(name, file_path)` → MeetingSummary
- `import_youtube_as_meeting(name, url)` → MeetingSummary
- `pick_audio_file()` → Option<String>

**Automation**:
- `trigger_meeting_command_now(project_path)` → Result

**GitHub Integration**:
- `test_github_connection()` → Result
- `list_github_repos()` → Vec<Repo>
- `set_github_repo(owner, name)`
- `push_meeting_changes(project_path)`
- `create_or_update_pr(project_path, meeting_summary)`
- `post_meeting_update_comment(pr_url, summary)`
- `github_begin_device_auth()` → (user_code, device_code)
- `github_poll_device_token(device_code)` → access_token

**LLM/Claude Integration**:
- `store_claude_api_key(key)`
- `has_claude_api_key()` → bool
- `delete_claude_api_key()`

**Codebase Analysis**:
- `analyze_project_codebase(path)` → CodebaseInfo
- `analyze_and_save_codebase(meeting_id, project_path)` → CodebaseInfo

---

## Part 11: Known Limitations & Future Opportunities

### 11.1 Current Limitations

1. **Speaker Diarization**
   - Simple 2-speaker toggle based on silence
   - No true speaker identification
   - Limited to binary speaker scenarios
   - Need: pyannote.audio or similar

2. **Context Window Management**
   - No sliding window for large meetings (60+ minutes)
   - All updates accumulate in single JSONL file
   - Claude may hit context limits
   - Need: Smart summarization of old updates

3. **Automation Integration**
   - AppleScript-only (macOS)
   - No Windows/Linux automation yet
   - Manual CLI trigger only on non-macOS
   - Need: Cross-platform automation layer

4. **Code Review**
   - All Claude Code changes auto-accepted if enabled
   - No validation before acceptance
   - No linting/testing gates
   - Need: Pre-accept validation stage

5. **Error Recovery**
   - No automatic retry on failed `/meeting` triggers
   - Failed transcriptions lost
   - GitHub auth timeout handling incomplete
   - Need: Exponential backoff + state recovery

6. **Model Support**
   - Whisper-only LLM for transcription
   - No support for other STT models
   - Need: Pluggable model system

7. **Language Support**
   - English-primary design
   - Limited multilingual support
   - Need: Full i18n + language-specific prompts

### 11.2 Enhancement Opportunities for Agentic Processing

**Opportunity 1: Intelligent Context Windowing**
- Detect when context approaching limit
- Summarize older updates into compressed form
- Maintain feature coverage matrix
- Implementation: `summarization/context_compressor.rs`

**Opportunity 2: Multi-Stage Code Validation**
- Run linting before auto-accept
- Run test suite on generated code
- Validate build succeeds
- Semantic validation of requirements coverage
- Implementation: `codebase/validator.rs`

**Opportunity 3: Real Speaker Diarization**
- Integrate pyannote.audio via Python subprocess
- Track speaker embeddings across meeting
- Handle >2 participants
- Implementation: `audio_toolkit/speaker_diarizer.rs`

**Opportunity 4: Intelligent Feature Tracking**
- Build feature-to-code mapping
- Track coverage percentage
- Identify unimplemented features
- Surface gaps to user
- Implementation: `summarization/feature_tracker.rs`

**Opportunity 5: Context Injection from Codebase**
- Extract relevant code snippets via semantic search
- Inject into /meeting command context
- Reduce hallucination in generated code
- Implementation: `codebase/context_injector.rs`

**Opportunity 6: Cross-Platform Automation**
- UNO/Win32 for Windows
- DBus for Linux
- Abstracted automation trait
- Implementation: `automation/platform_automation.rs`

**Opportunity 7: Conflict Resolution**
- Detect overlapping changes
- Smart merge strategy for incremental updates
- User-facing conflict UI
- Implementation: `codebase/conflict_resolver.rs`

**Opportunity 8: Meeting Insights & Analytics**
- Feature velocity tracking
- Participant contribution analysis
- Decision timeline visualization
- Implementation: `analytics/meeting_insights.rs`

---

## Summary Table: Components at a Glance

| Component | Location | LOC | Purpose | Status |
|-----------|----------|-----|---------|--------|
| AudioRecordingManager | managers/audio.rs | 25KB | System audio capture | ✅ Complete |
| TranscriptionManager | managers/transcription.rs | 16KB | Model inference & lifecycle | ✅ Complete |
| MeetingManager | managers/meeting.rs | 71KB | Meeting orchestration | ✅ Complete |
| Summarization Agent | summarization/agent.rs | ~400 | Feature extraction (heuristic) | ✅ Complete |
| LLM Summarization | summarization/llm.rs | ~600 | Claude-powered extraction | ✅ Complete |
| Claude Trigger | automation/claude_trigger.rs | 12KB | /meeting automation | ⚠️ Partial |
| GitHub Integration | integrations/github.rs | 20KB | OAuth, PR, push, comments | ✅ Complete |
| Codebase Analysis | codebase/analyzer.rs | 18KB | Framework/lang detection | ✅ Complete |
| File Isolation | codebase/isolation.rs | 12KB | Safety rules (.claudeignore) | ✅ Complete |
| ASR Worker | workers/asr_worker.rs | 5KB | Transcription processing | ✅ Complete |
| Audio Queue | queue.rs | 10KB | Durable queue (SQLite) | ✅ Complete |
| MeetingView | src/components/meeting/ | ~500 | Meeting UI | ✅ Complete |
| Settings | src/components/settings/ | ~2000 | Configuration UI | ✅ Complete |

---

## Conclusion

MeetingCoder is a sophisticated, production-grade application that demonstrates advanced agentic capabilities:

**Currently Implemented**:
- ✅ Real-time continuous transcription
- ✅ Automated requirement summarization
- ✅ File-based context protocol for Claude Code
- ✅ GitHub integration with OAuth
- ✅ Codebase analysis and safe code routing
- ✅ LLM-powered requirement extraction
- ✅ Configurable automation triggers

**Areas for Autonomous Enhancement**:
- Context windowing for long meetings
- Multi-stage code validation
- True speaker diarization
- Intelligent feature coverage tracking
- Error recovery and automatic retry
- Cross-platform automation

The application is ideally positioned for Phase 7 completion (real-time code generation) and serves as an excellent foundation for exploring advanced agentic patterns in Rust/Tauri applications.

