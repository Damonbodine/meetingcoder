# Phase 2: Claude Code Integration & Requirement Structuring

## Overview

Phase 2 transforms meeting transcripts into working code by integrating with Claude Code CLI. Instead of implementing custom code generation, we leverage Claude Code's existing capabilities through a persistent session model with structured context updates.

**Timeline**: 3-4 weeks
**Priority**: P0 (Core value proposition)
**Dependencies**: Phase 1 (continuous meeting transcription) working on macOS

## Status Update (2025-11-05)

- Phase 1 dependency: ✅ satisfied on macOS (10s chunks, auto-save transcripts)
- Implemented assets/templates: `/meeting` command template exists in repo ✅
- Implemented in Phase 2 so far:
  - ✅ Minimal summarization agent (deterministic heuristic) extracting features/decisions/questions
  - ✅ File-based context writer for `.meeting-updates.jsonl` with persistent `update_id` and metadata
  - ✅ Runtime trigger appends updates during meetings at configurable interval (default 20s)
  - ✅ Project initialization at meeting start: creates project dir, `.claude` scaffolding, copies `/meeting` command, seeds state, initializes git
  - ✅ Frontend: settings for chunk duration and update interval; basic “Meeting Updates” viewer
- Not implemented yet:
  - ❌ LLM-backed summarization with deduplication vs prior context
  - ❌ `/meeting` automation (AppleScript) and auto-accept flow
  - ❌ First-update project metadata: project_type, tech_stack extraction
  - ❌ Soak instrumentation (metrics over 60+ minutes)

Recommended next steps:
- Upgrade summarizer to consider previous context and richer schema (IDs, priorities)
- Add soak instrumentation (update counts, file size, memory)
- Implement optional automation trigger (Phase 3) gated by setting
- Expand first-update metadata when available

## Goals

1. Extract structured requirements from conversational transcripts via summarization agent
2. Create protocol for feeding context to Claude Code session
3. Build custom `/meeting` slash command for Claude Code
4. Implement automation for triggering Claude Code updates during meetings
5. Support persistent Claude Code session throughout meeting duration

## Success Criteria

- [ ] Summarization agent extracts 80%+ of discussed features from transcripts
- [x] `.meeting-updates.jsonl` protocol successfully feeds context to Claude Code (schema present, append-only)
- [x] `/meeting` slash command reads and processes new requirements (template provided)
- [ ] Claude Code generates syntactically valid code (handled by Claude Code)
- [ ] Automation triggers Claude Code every 60-90 seconds during active discussion (Phase 3)
- [x] Generated project structure initialized for meetings (project scaffolding & git)
- [x] User can review Claude Code's proposed changes before accepting (manual command)

## Architecture Changes from Original PRD

**What Changed**:
- ❌ No custom LLM provider integration (Claude, OpenAI, Ollama)
- ❌ No custom code generation engine
- ❌ No custom validation layer
- ✅ Use Claude Code CLI as code generation engine
- ✅ Summarization agent for transcript → structured requirements
- ✅ File-based protocol for context passing (`.meeting-updates.jsonl`)
- ✅ Custom Claude Code slash command (`/meeting`)
- ✅ AppleScript-based automation for triggering updates

## Features & Requirements

### 2.1 Summarization Agent

**User Story**: As a user, I want the raw meeting transcript automatically converted into structured requirements that Claude Code can understand and act upon.

**Requirements**:

#### Purpose
Transform verbose, conversational transcripts into concise, structured requirements suitable for code generation.

**Input**: Raw transcript segments
```json
[
  {
    "speaker": "Speaker 1",
    "start_time": 125.5,
    "end_time": 132.3,
    "text": "Yeah, so we need users to be able to upload a CSV file, and then we validate the columns..."
  },
  {
    "speaker": "Speaker 2",
    "start_time": 133.0,
    "end_time": 138.5,
    "text": "Right, and if the validation fails, we should show them which columns are wrong."
  }
]
```

**Output**: Structured requirements
```json
{
  "timestamp": "2025-01-30T14:23:45Z",
  "segment_range": [10, 15],
  "new_features": [
    {
      "id": "f3",
      "title": "CSV File Upload",
      "description": "Users can upload CSV files for processing",
      "priority": "high",
      "technical_notes": "Need file validation for CSV format",
      "mentioned_by": "Speaker 1",
      "timestamp": 125.5
    },
    {
      "id": "f4",
      "title": "Column Validation with Error Display",
      "description": "Validate CSV columns and show specific errors for invalid columns",
      "priority": "high",
      "technical_notes": "User-friendly error messages showing which columns failed",
      "mentioned_by": "Speaker 2",
      "timestamp": 133.0
    }
  ],
  "technical_decisions": [
    "CSV file format required",
    "Client-side or server-side validation TBD"
  ],
  "questions": [
    "What is the maximum file size for CSV uploads?",
    "Should validation be client-side or server-side?"
  ]
}
```

**Summarization Prompt Template**:
```
You are a requirements analyst listening to a software development meeting.

NEW TRANSCRIPT SEGMENTS:
{transcript_segments}

PREVIOUS CONTEXT (if any):
{previous_requirements}

TASK:
1. Extract any NEW features or requirements mentioned in these segments
2. Identify technical decisions or constraints discussed
3. Note any questions or uncertainties that need clarification
4. Do NOT repeat features already in the previous context
5. Be concise but capture all important details

OUTPUT FORMAT: JSON matching the structure shown above
```

**Technical Implementation**:
```rust
// src-tauri/src/summarization/agent.rs
pub struct SummarizationAgent {
    llm_provider: Box<dyn LLMProvider>, // Claude API for summarization
}

impl SummarizationAgent {
    pub async fn summarize_segments(
        &self,
        new_segments: &[TranscriptSegment],
        previous_context: Option<&MeetingContext>,
    ) -> Result<StructuredRequirements> {
        let prompt = self.build_prompt(new_segments, previous_context);

        let response = self.llm_provider
            .generate(prompt, max_tokens: 2000)
            .await?;

        // Parse JSON response
        let requirements: StructuredRequirements = serde_json::from_str(&response)?;
        Ok(requirements)
    }
}
```

**Acceptance Criteria**:
- [ ] Extracts discrete features from conversational text
- [ ] Avoids duplicating previously captured requirements
- [ ] Identifies technical constraints and decisions
- [ ] Flags unclear requirements as questions
- [ ] Processes 30-60 seconds of transcript in <10 seconds
- [ ] JSON output is valid and parseable

---

### 2.2 Meeting Context Protocol

**User Story**: As a Claude Code session, I need a standardized way to receive and process new meeting context as it arrives.

**Requirements**:

#### File-Based Protocol: `.meeting-updates.jsonl`

**Location**: `~/MeetingCoder/projects/{project-name}/.meeting-updates.jsonl`

**Format**: JSON Lines (append-only log)

Each line represents one update from the summarization agent:

```jsonl
{"update_id": "u1", "timestamp": "2025-01-30T14:15:30Z", "segment_range": [0, 5], "project_name": "customer-feedback-app", "project_type": "react_web_app", "tech_stack": ["React", "TypeScript", "Node.js"], "features": [...], "technical_decisions": [...], "questions": [...]}
{"update_id": "u2", "timestamp": "2025-01-30T14:17:00Z", "segment_range": [6, 10], "new_features": [...], "technical_decisions": [...], "questions": [...]}
{"update_id": "u3", "timestamp": "2025-01-30T14:18:30Z", "segment_range": [11, 15], "new_features": [...], "clarifications": {"f2": "File size limit confirmed: 10MB"}, "technical_decisions": [...]}
```

**Schema**:
```typescript
interface MeetingUpdate {
  update_id: string;           // Unique update identifier
  timestamp: string;           // ISO 8601
  segment_range: [number, number]; // Which transcript segments this covers

  // First update only:
  project_name?: string;
  project_type?: string;
  tech_stack?: string[];

  // All updates:
  new_features?: Feature[];
  modified_features?: {[feature_id: string]: Partial<Feature>};
  technical_decisions?: string[];
  questions?: string[];
  clarifications?: {[feature_id: string]: string}; // Answers to previous questions
}

interface Feature {
  id: string;
  title: string;
  description: string;
  priority: "high" | "medium" | "low";
  technical_notes?: string;
  mentioned_by: string;
  timestamp: number;
}
```

**Write Pattern** (MeetingCoder App):
```rust
// src-tauri/src/meeting/context_writer.rs
pub struct ContextWriter {
    project_path: PathBuf,
    update_counter: AtomicU32,
}

impl ContextWriter {
    pub fn append_update(&self, requirements: StructuredRequirements) -> Result<()> {
        let update_file = self.project_path.join(".meeting-updates.jsonl");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(update_file)?;

        let update = MeetingUpdate {
            update_id: format!("u{}", self.update_counter.fetch_add(1, Ordering::SeqCst)),
            timestamp: Utc::now().to_rfc3339(),
            segment_range: requirements.segment_range,
            new_features: requirements.new_features,
            // ...
        };

        writeln!(file, "{}", serde_json::to_string(&update)?)?;
        Ok(())
    }
}
```

**Acceptance Criteria**:
- [ ] JSONL format is append-only (never modified)
- [ ] Each update is atomic (one line)
- [ ] File is readable by Claude Code session
- [ ] Schema is well-documented and versioned
- [ ] Handles concurrent writes safely

---

### 2.3 Custom Claude Code Slash Command: `/meeting`

**User Story**: As a Claude Code session, I need a command to read and process new meeting updates from the `.meeting-updates.jsonl` file.

**Requirements**:

#### Slash Command Specification

**Command**: `/meeting`
**Aliases**: `/m`, `/update-from-meeting`
**Location**: `.claude/commands/meeting.md`

**Functionality**:
1. Read `.meeting-updates.jsonl` from current working directory
2. Track which updates have been processed (state file: `.claude/.meeting-state.json`)
3. Process only new updates since last invocation
4. Generate/update code based on new requirements
5. Provide summary of what was added/changed

**Implementation** (`.claude/commands/meeting.md`):
```markdown
Read new updates from .meeting-updates.jsonl and update the codebase accordingly.

Steps:
1. Check if .meeting-updates.jsonl exists in the current directory
2. Read .claude/.meeting-state.json to see which updates have been processed (last_processed_update_id)
3. Read .meeting-updates.jsonl and extract only lines after last_processed_update_id
4. Parse the new updates
5. For the FIRST update (if this is a new project):
   - Create initial project structure based on project_type and tech_stack
   - Implement high-priority features from the first batch
6. For SUBSEQUENT updates:
   - Implement new_features as new components/modules
   - Apply modified_features by updating existing code
   - Address clarifications by refining existing implementations
7. Update .claude/.meeting-state.json with the latest update_id processed
8. Summarize changes made

Context to consider:
- Maintain consistency with existing code style
- Add comments explaining key logic
- Include error handling
- Follow best practices for the tech stack
- Don't over-engineer - implement MVP functionality

If there are no new updates, respond: "No new meeting updates to process."
```

**State Tracking** (`.claude/.meeting-state.json`):
```json
{
  "last_processed_update_id": "u5",
  "last_processed_timestamp": "2025-01-30T14:23:45Z",
  "total_updates_processed": 5
}
```

**Example Usage Flow**:
```
User in Terminal:
$ cd ~/MeetingCoder/projects/customer-feedback-app
$ claude
> /meeting

Claude Code:
Reading .meeting-updates.jsonl...
Found 2 new updates since last check (u4, u5)

Update u4: Added CSV upload feature
- Created components/FileUpload.tsx
- Added file validation logic
- Updated App.tsx to include FileUpload

Update u5: Added column validation with error display
- Enhanced FileUpload component with validation
- Created components/ValidationErrors.tsx
- Added error state management

Summary: Implemented CSV file upload with column validation. Users can now upload files and see specific validation errors.

Proceed with these changes? (y/n)
```

**Acceptance Criteria**:
- [ ] `/meeting` command reads `.meeting-updates.jsonl` correctly
- [ ] Processes only new updates (tracks state)
- [ ] Generates appropriate code for new features
- [ ] Updates existing code for modifications
- [ ] Provides clear summary of changes made
- [ ] Handles empty/no-new-updates case gracefully

---

### 2.4 Claude Code Session Management

**User Story**: As a user, I want Claude Code to remain active during my meeting so code generation happens continuously without manual intervention.

**Requirements**:

#### Persistent Session Model

**Setup** (before meeting):
1. User creates project directory: `mkdir ~/MeetingCoder/projects/my-project`
2. User starts Claude Code session: `cd ~/MeetingCoder/projects/my-project && claude`
3. Claude Code session remains open in Terminal during meeting
4. MeetingCoder app writes updates to `.meeting-updates.jsonl` every 60-90s

**During Meeting**:
- MeetingCoder app triggers `/meeting` command periodically
- Two approaches:

**Approach A: Manual Trigger** (Phase 2)
- User keeps Claude Code Terminal visible during meeting
- When notification appears: "New meeting context available"
- User types `/meeting` and reviews/accepts changes
- Process takes ~10-20 seconds per update

**Approach B: Automated Trigger** (Phase 3)
- MeetingCoder app uses AppleScript to send `/meeting` command to Terminal
- User pre-configures auto-accept mode
- Code updates happen automatically

#### Automation Implementation (Phase 3 Preview)

**AppleScript Automation**:
```rust
// src-tauri/src/automation/claude_trigger.rs
pub struct ClaudeTrigger {
    terminal_window: String, // Terminal window ID running Claude Code
}

impl ClaudeTrigger {
    pub async fn trigger_meeting_update(&self) -> Result<()> {
        // Send /meeting command to Terminal
        Command::new("osascript")
            .arg("-e")
            .arg(format!(r#"
                tell application "Terminal"
                    tell window 1
                        do script "/meeting" in selected tab
                    end tell
                end tell
            "#))
            .spawn()?
            .wait()
            .await?;

        // Wait for Claude Code to present changes
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Auto-accept (send "y" + Enter)
        Command::new("osascript")
            .arg("-e")
            .arg(r#"
                tell application "System Events"
                    keystroke "y"
                    keystroke return
                end tell
            "#)
            .spawn()?;

        Ok(())
    }
}
```

**Acceptance Criteria**:
- [ ] Claude Code session remains stable for 2+ hour meetings
- [ ] `/meeting` can be triggered externally via automation
- [ ] Auto-accept mechanism works reliably (Phase 3)
- [ ] User can fallback to manual trigger if automation fails
- [ ] Terminal window management is robust

---

### 2.5 Project Initialization

**User Story**: As a user starting a new meeting, I want MeetingCoder to create a project folder and initialize it for Claude Code.

**Requirements**:

#### Automatic Project Setup

**When**: Meeting starts and project type is detected (first 2-3 minutes)

**Actions**:
1. Create project directory: `~/MeetingCoder/projects/{meeting-name}/`
2. Write initial `.meeting-updates.jsonl` with project metadata
3. Create `.claude/` directory structure
4. Copy `/meeting` slash command definition
5. Initialize git repository
6. Create placeholder README.md

**Implementation**:
```rust
// src-tauri/src/project/initializer.rs
pub struct ProjectInitializer {
    projects_root: PathBuf,
}

impl ProjectInitializer {
    pub fn create_project(&self, meeting_name: &str) -> Result<PathBuf> {
        let project_path = self.projects_root.join(meeting_name);
        fs::create_dir_all(&project_path)?;

        // Create .claude directory
        let claude_dir = project_path.join(".claude");
        fs::create_dir_all(&claude_dir)?;

        // Create .claude/commands directory
        let commands_dir = claude_dir.join("commands");
        fs::create_dir_all(&commands_dir)?;

        // Write /meeting command
        let meeting_command = include_str!("../../templates/meeting_command.md");
        fs::write(commands_dir.join("meeting.md"), meeting_command)?;

        // Initialize .meeting-updates.jsonl (empty)
        fs::write(project_path.join(".meeting-updates.jsonl"), "")?;

        // Initialize .claude/.meeting-state.json
        let initial_state = json!({
            "last_processed_update_id": null,
            "last_processed_timestamp": null,
            "total_updates_processed": 0
        });
        fs::write(
            claude_dir.join(".meeting-state.json"),
            serde_json::to_string_pretty(&initial_state)?
        )?;

        // Initialize git
        Command::new("git")
            .current_dir(&project_path)
            .args(&["init"])
            .output()?;

        // Create README placeholder
        let readme = format!("# {}\n\nGenerated during meeting by MeetingCoder\n", meeting_name);
        fs::write(project_path.join("README.md"), readme)?;

        Ok(project_path)
    }
}
```

**User Flow**:
1. User starts meeting in MeetingCoder app
2. After 2-3 minutes (sufficient context), app shows: "Ready to create project?"
3. User confirms project name
4. App creates project folder
5. App opens Terminal and prompts: `cd ~/MeetingCoder/projects/{name} && claude`
6. User starts Claude Code session
7. MeetingCoder begins writing updates

**Acceptance Criteria**:
- [ ] Project directory created with correct structure
- [ ] `.claude/commands/meeting.md` is present and functional
- [ ] Git repository initialized
- [ ] User notified with terminal command to start Claude Code
- [ ] README includes meeting metadata

---

### 2.6 Summarization Trigger Logic

**User Story**: As a user, I want MeetingCoder to intelligently decide when to run the summarization agent based on meeting activity.

**Requirements**:

#### Smart Triggering Heuristics

**Trigger Conditions** (any of):
1. **Time-based**: Every 60-90 seconds of active discussion
2. **Content-based**: New feature keywords detected (e.g., "we need", "users should", "implement", "build")
3. **Speaker-based**: Both/multiple speakers have contributed recent segments
4. **Manual**: User clicks "Update Code Now" button

**Implementation**:
```rust
// src-tauri/src/meeting/trigger.rs
pub struct SummarizationTrigger {
    last_summarization: Instant,
    pending_segments: Vec<TranscriptSegment>,
    min_interval: Duration, // 60 seconds
    max_interval: Duration, // 90 seconds
}

impl SummarizationTrigger {
    pub fn should_trigger(&self) -> bool {
        let elapsed = self.last_summarization.elapsed();

        // Must wait at least min_interval
        if elapsed < self.min_interval {
            return false;
        }

        // Force trigger after max_interval
        if elapsed >= self.max_interval {
            return true;
        }

        // Check for requirement keywords
        let has_requirements = self.pending_segments.iter().any(|seg| {
            let text = seg.text.to_lowercase();
            text.contains("need") || text.contains("should") ||
            text.contains("want") || text.contains("implement") ||
            text.contains("build") || text.contains("create")
        });

        // Check for multi-speaker activity
        let speakers: HashSet<_> = self.pending_segments
            .iter()
            .map(|s| &s.speaker)
            .collect();
        let multi_speaker = speakers.len() >= 2;

        has_requirements && multi_speaker && elapsed > Duration::from_secs(45)
    }
}
```

**Acceptance Criteria**:
- [ ] Triggers every 60-90s during active requirements discussion
- [ ] Doesn't trigger during silence or off-topic conversation
- [ ] Respects minimum interval to avoid spam
- [ ] Forces trigger at maximum interval to maintain momentum
- [ ] User can manually trigger anytime

---

## Technical Architecture

### New Modules

```
src-tauri/src/
  summarization/
    mod.rs              # Summarization agent
    agent.rs            # Core summarization logic
    prompts.rs          # Prompt templates

  meeting/
    mod.rs              # Meeting orchestration
    context_writer.rs   # Write .meeting-updates.jsonl
    trigger.rs          # Decide when to summarize

  automation/          # Phase 3
    mod.rs
    claude_trigger.rs   # AppleScript automation

  templates/
    meeting_command.md  # /meeting slash command template
```

### Data Flow

```
Meeting Transcript (Phase 1)
    ↓
[Trigger Logic] → Should we summarize now?
    ↓ (every 60-90s)
[Summarization Agent] → Extract requirements
    ↓
StructuredRequirements JSON
    ↓
[Context Writer] → Append to .meeting-updates.jsonl
    ↓
[Notification to User] → "New context available"
    ↓
User (or Automation) → Types "/meeting" in Claude Code
    ↓
Claude Code → Reads .meeting-updates.jsonl
    ↓
Claude Code → Generates/updates code
    ↓
User → Reviews and accepts changes
    ↓
Generated/Updated Code in Project
```

## Dependencies

**New Rust Crates**:
- `reqwest` - HTTP client for Claude API (summarization agent)
- `serde_json` - JSON parsing/serialization
- `tokio` - Async runtime
- `chrono` - Timestamp handling

**No Custom LLM Integration**:
- Code generation handled by Claude Code CLI
- Only need Claude API for summarization agent

## Testing Strategy

### Unit Tests
- Summarization agent prompt construction
- Trigger logic heuristics
- `.meeting-updates.jsonl` writer
- State tracking for `/meeting` command

### Integration Tests
- End-to-end: transcript → summarization → file write
- `/meeting` command reads and processes updates correctly
- Project initialization creates correct structure

### Manual Testing
- Real meeting: ensure summarization captures key features
- Claude Code session: verify `/meeting` works as expected
- Automation: test AppleScript triggering (Phase 3)

## Documentation Deliverables

1. **Summarization Agent Guide**: How the agent structures requirements
2. **`.meeting-updates.jsonl` Protocol Spec**: Format and schema
3. **`/meeting` Command Usage**: How to use the slash command
4. **Automation Setup Guide**: Configure AppleScript triggers (Phase 3)

## Phase 2 Completion Checklist

- [ ] Summarization agent extracts requirements accurately
- [ ] `.meeting-updates.jsonl` protocol implemented
- [ ] `/meeting` slash command functional
- [ ] Project initialization works
- [ ] Trigger logic behaves correctly
- [ ] End-to-end flow tested with real meeting
- [ ] Documentation complete
- [ ] Ready for Phase 3 (automation + real-time)

## Handoff to Phase 3

Phase 2 deliverables required for Phase 3:
1. Working summarization agent
2. Stable `.meeting-updates.jsonl` protocol
3. Functional `/meeting` slash command
4. Project initialization system

Phase 3 will add:
- AppleScript automation for triggering `/meeting`
- Auto-accept mechanism for Claude Code tool calls
- Real-time code preview during meeting
- Performance optimization for <2min lag
