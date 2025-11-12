# Phase 3: Real-Time Automation & Polish

## Overview

Phase 3 transforms MeetingCoder from a manual-trigger tool into a fully automated live coding assistant. AppleScript automation triggers Claude Code updates during the meeting, and code generation happens in real-time with minimal user intervention.

**Timeline**: 2-3 weeks
**Priority**: P1 (Differentiation feature)
**Dependencies**: Phase 2 complete (summarization agent + `/meeting` command working)

## Status Update (2025-11-05)

- Pre-reqs: Phase 2 not yet implemented (streaming to LLM gated)
- Partial groundwork:
  - Continuous transcription now emits segments every 10s
  - Stable capture/processing path on macOS
- Not implemented:
  - Streaming segments to LLM / incremental code updates
  - Live preview, automation triggers, and auto-accept
  - Performance hardening for <2min end-to-end

Near-term enablers (after Phase 2):
- Switch segment feed cadence to 10–20s (already aligned)
- Add orchestrator to watch `.meeting-updates.jsonl` and trigger `/meeting`
- Instrument end-to-end latency metrics (speech → code)

## Key Changes from Original PRD

**What Changed**:
- ❌ No custom streaming orchestrator (Claude Code handles this)
- ❌ No custom incremental update logic (Claude Code manages context)
- ✅ AppleScript automation for triggering `/meeting` command
- ✅ Auto-accept mechanism for Claude Code tool calls
- ✅ Notification system for user awareness
- ✅ Performance optimization (<2min lag from speech to code)

## Goals

1. Stream transcription segments to LLM as they arrive (not waiting for meeting end)
2. Generate code incrementally during the meeting
3. Show live code preview to user during call
4. Enable "live demo" mode where generated UI is immediately viewable
5. Add quality-of-life features based on beta feedback
6. Optimize performance for <2 minute lag from speech to code

## Success Criteria

- [ ] Code generation begins within 2 minutes of first requirements mentioned
- [ ] Incremental updates arrive every 30-60 seconds during active discussion
- [ ] Live preview works for web applications (React, HTML)
- [ ] User can share screen of generated app during same meeting
- [ ] Memory usage stays under 2GB during 2-hour meeting
- [ ] 90%+ uptime (no crashes during meetings)
- [ ] Beta tester satisfaction: 4+ stars

## Features & Requirements

### 3.1 Real-Time Transcription Streaming

**User Story**: As a user, I want transcription segments sent to the AI as soon as they're ready so code generation can start before the meeting ends.

**Requirements**:

#### Streaming Architecture
- Transcript segments sent to LLM every 30-60 seconds
- Accumulate context from previous segments
- Trigger generation when enough context exists (first requirements mentioned)
- Continue updating as more details emerge

**Technical Design**:
```rust
pub struct StreamingOrchestrator {
    transcript_buffer: Vec<TranscriptSegment>,
    last_sent_index: usize,
    generation_state: GenerationState,
}

enum GenerationState {
    Idle,                          // No requirements yet
    RequirementsGathering,         // Collecting initial context
    GeneratingInitial,             // First code gen in progress
    IterativeUpdating,             // Applying incremental changes
}

impl StreamingOrchestrator {
    pub async fn on_new_segment(&mut self, segment: TranscriptSegment) {
        self.transcript_buffer.push(segment);

        match self.generation_state {
            GenerationState::Idle => {
                if self.has_enough_context() {
                    self.start_initial_generation().await;
                }
            },
            GenerationState::IterativeUpdating => {
                if self.should_trigger_update() {
                    self.apply_incremental_update().await;
                }
            },
            _ => {} // Wait for current operation
        }
    }

    fn has_enough_context(&self) -> bool {
        // Heuristic: Do we have enough to start?
        // - Project type mentioned
        // - At least 2-3 features discussed
        // - 2+ minutes of conversation
        self.transcript_buffer.len() > 4 &&
        self.contains_requirement_keywords()
    }
}
```

**Acceptance Criteria**:
- [ ] Segments streamed to LLM every 30-60 seconds
- [ ] Generation starts when sufficient context exists (auto-detection)
- [ ] No duplicate generations (idempotent)
- [ ] Handles interruptions (speaker changes, tangents)
- [ ] User notified when generation starts

### 3.2 Incremental Code Updates

**User Story**: As a user, I want the code to update incrementally as I discuss more features, not regenerate everything from scratch each time.

**Requirements**:

#### Smart Diffing
- Determine what changed in requirements since last generation
- Generate only affected files
- Preserve user edits where possible
- Merge updates intelligently (not full overwrites)

**Update Triggers**:
1. **Additive**: New feature mentioned → Generate new component/module
2. **Modification**: Existing feature refined → Update specific file
3. **Clarification**: Ambiguity resolved → Regenerate with better context

**Prompt Strategy** (incremental):
```
You are updating an existing codebase based on new requirements from a meeting.

PREVIOUS REQUIREMENTS:
{previous_requirements_json}

PREVIOUS CODE FILES:
{file_manifest}

NEW TRANSCRIPT SEGMENTS:
{new_segments}

INSTRUCTIONS:
1. Identify what has CHANGED in requirements
2. Generate ONLY the files that need updates
3. For existing files, provide minimal diffs (not full rewrites)
4. For new features, generate new files

OUTPUT:
{
  "changes": [
    {
      "type": "update",
      "path": "src/components/LoginForm.tsx",
      "reason": "Added password strength requirements",
      "diff": "..."
    },
    {
      "type": "create",
      "path": "src/components/PasswordStrength.tsx",
      "reason": "New component for password validation",
      "content": "..."
    }
  ]
}
```

**Acceptance Criteria**:
- [ ] Updates apply in <30 seconds
- [ ] Only changed files are regenerated
- [ ] User edits preserved unless conflicting
- [ ] Clear changelog of what was updated and why
- [ ] Rollback capability if update breaks something

### 3.3 Live Code Preview

**User Story**: As a user, I want to see the generated UI running live during the meeting so I can show it to stakeholders immediately.

**Requirements**:

#### Web Preview Server
- Embedded dev server (Vite, webpack-dev-server)
- Auto-reloads when files change
- Accessible via localhost URL
- Hot module replacement (HMR) for instant updates

**Preview Modes**:
1. **Code View**: Syntax-highlighted file browser
2. **UI Preview**: Live rendering of web apps
3. **Split View**: Code + Preview side-by-side

**Technical Implementation**:
```rust
pub struct PreviewServer {
    port: u16,
    project_path: PathBuf,
    process: Option<Child>,
}

impl PreviewServer {
    pub async fn start(&mut self) -> Result<String> {
        // Detect project type
        let project_type = detect_project_type(&self.project_path)?;

        // Start appropriate dev server
        let process = match project_type {
            ProjectType::React => {
                Command::new("npm")
                    .args(&["run", "dev"])
                    .current_dir(&self.project_path)
                    .spawn()?
            }
            ProjectType::NodeAPI => {
                Command::new("node")
                    .args(&["--watch", "src/index.js"])
                    .current_dir(&self.project_path)
                    .spawn()?
            }
            _ => return Err("Preview not supported for this project type"),
        };

        self.process = Some(process);
        Ok(format!("http://localhost:{}", self.port))
    }
}
```

**Preview UI**:
```
┌────────────────────────────────────────────────┐
│ Live Preview: Customer Feedback Dashboard  [X]│
├────────────────────────────────────────────────┤
│ [Code] [Preview] [Split]     ⟳ Refresh   ⚙    │
├──────────────┬─────────────────────────────────┤
│ 📁 src/      │                                 │
│ ├─ App.tsx   │   ┌───────────────────────┐   │
│ ├─ main.tsx  │   │ Customer Feedback     │   │
│ └─ components│   ├───────────────────────┤   │
│    ├─ Login  │   │ Email: [__________]   │   │
│    └─ Feed.. │   │ Password: [________]  │   │
│              │   │ [Login]               │   │
│              │   └───────────────────────┘   │
│              │                                 │
│              │   ✓ Auto-reload on file change │
└──────────────┴─────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] Preview server starts automatically
- [ ] Updates reflected within 2-3 seconds of file change
- [ ] Works for React, HTML, and simple Node.js APIs
- [ ] Clear error messages if preview fails
- [ ] Can share preview URL over screen share

### 3.4 Meeting Insights & Suggestions

**User Story**: As a user, I want the AI to proactively suggest improvements or flag potential issues during the meeting.

**Requirements**:

#### Real-Time Analysis
- Detect missing requirements (mentioned "users" but no auth discussed)
- Suggest technical approaches ("Consider using WebSockets for real-time features")
- Flag scope creep ("This seems complex - consider MVP version first")
- Identify contradictions ("Stakeholder said X earlier, but now saying Y")

**Insight Types**:
1. **Missing Requirements**: "⚠️ No database mentioned - where will data be stored?"
2. **Technical Suggestions**: "💡 For file uploads, consider using S3 or similar"
3. **Scope Warnings**: "⚠️ This feature is complex - may delay MVP"
4. **Clarification Needed**: "❓ Unclear if authentication should be social or email/password"

**UI Integration**:
```
┌─────────────────────────────────────┐
│ Live Meeting Insights           [x] │
├─────────────────────────────────────┤
│ 💡 SUGGESTION (00:15:23)            │
│ "Consider using Firebase for        │
│ authentication - faster than        │
│ building from scratch"              │
│ [Apply] [Dismiss]                   │
│                                      │
│ ⚠️ MISSING INFO (00:18:45)          │
│ "No database mentioned yet. Ask:    │
│ What data needs to persist?"        │
│ [Ask in Meeting] [Dismiss]          │
│                                      │
│ ❓ CLARIFICATION (00:22:10)         │
│ "Two conflicting requirements:      │
│ - 'Keep it simple' (00:05:30)       │
│ - 'Real-time updates' (00:22:10)    │
│ Which takes priority?"              │
│ [Clarify] [Dismiss]                 │
└─────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] At least 2-3 useful insights per 30-minute meeting
- [ ] <10% false positives (irrelevant suggestions)
- [ ] Insights arrive within 30 seconds of relevant discussion
- [ ] User can dismiss or act on insights
- [ ] Applied suggestions reflected in generated code

### 3.5 Meeting Summary & Handoff

**User Story**: As a user, I want a comprehensive summary at the meeting end that includes requirements, generated code, and next steps.

**Requirements**:

#### Auto-Generated Summary
- Executive summary (2-3 sentences)
- Feature list with priorities
- Technical decisions made
- Open questions / action items
- Links to generated code
- Setup instructions for stakeholder demo

**Summary Formats**:
1. **Markdown** (for README)
2. **PDF** (for sharing with stakeholders)
3. **JSON** (for programmatic access)

**Example Summary**:
```markdown
# Meeting Summary: Customer Feedback Dashboard
**Date**: January 15, 2025
**Duration**: 45 minutes
**Participants**: You, Sarah (Product Manager)

## Executive Summary
Discussed building a web-based customer feedback dashboard for collecting
and analyzing user feedback. Stakeholder prioritized quick MVP with core
submission and admin review features.

## Features Discussed

### High Priority
- ✅ User authentication (email/password)
- ✅ Feedback submission form (text + category)
- ✅ Admin dashboard to view submissions

### Medium Priority
- 🔄 Export feedback to CSV
- 🔄 Email notifications on new feedback

### Deferred
- ⏸️ Analytics and trending (v2 feature)
- ⏸️ Mobile app (web-first approach)

## Technical Decisions
- **Frontend**: React + TypeScript + Vite
- **Backend**: Node.js + Express
- **Database**: PostgreSQL
- **Hosting**: Vercel (frontend) + Heroku (backend)

## Generated Code
📁 Project location: `~/MeetingCoder/projects/customer-feedback-dashboard/`
🌐 Live preview: http://localhost:5173

**Files generated**: 12 files, 847 lines of code

## Open Questions
1. Should we support anonymous feedback submission?
2. What's the expected user volume (for database sizing)?
3. Do we need role-based access (admin vs. viewer)?

## Next Steps
1. Review generated code in project folder
2. Run `npm install && npm run dev` to test locally
3. Address open questions with stakeholder
4. Deploy to staging for stakeholder review
5. Schedule follow-up in 1 week

## Demo Instructions
To show stakeholder the generated MVP:
1. Navigate to project: `cd ~/MeetingCoder/projects/customer-feedback-dashboard`
2. Install: `npm install`
3. Start: `npm run dev`
4. Open: http://localhost:5173
5. Test accounts: admin@test.com / password123

---
Generated by MeetingCoder on Jan 15, 2025 at 3:45 PM
```

**Acceptance Criteria**:
- [ ] Summary generated automatically at meeting end
- [ ] All key decisions captured
- [ ] Open questions clearly listed
- [ ] Next steps actionable
- [ ] PDF export works
- [ ] Can be shared via email directly from app

### 3.6 Performance Optimization

**User Story**: As a user, I want the app to be fast and responsive even during long meetings with complex code generation.

**Requirements**:

#### Optimization Targets
- **Latency**: Speech → Code in <2 minutes
- **Memory**: <2GB RAM during 2-hour meeting
- **CPU**: <60% average during generation
- **Responsiveness**: UI never freezes (async operations)

**Optimization Strategies**:
1. **Parallel Processing**
   - Transcribe and generate concurrently
   - Multi-threaded file writing
   - Async LLM calls with queue

2. **Caching**
   - Cache LLM responses for similar prompts
   - Reuse project templates
   - Cache validation results

3. **Resource Management**
   - Unload old transcript segments from memory
   - Stream large files instead of loading entirely
   - Garbage collect completed tasks

4. **Background Tasks**
   - Run validation in background
   - Pre-compile preview server
   - Index generated code for search

**Performance Monitoring**:
```rust
pub struct PerformanceMonitor {
    metrics: Arc<Mutex<Metrics>>,
}

struct Metrics {
    transcription_latency: Vec<Duration>,
    generation_latency: Vec<Duration>,
    memory_usage: Vec<usize>,
    cpu_usage: Vec<f32>,
}

impl PerformanceMonitor {
    pub fn record_event(&self, event: PerformanceEvent) {
        // Track metrics for debugging
    }

    pub fn get_report(&self) -> PerformanceReport {
        // Generate performance summary
    }
}
```

**Acceptance Criteria**:
- [ ] All latency targets met 90%+ of the time
- [ ] No memory leaks (constant usage after warmup)
- [ ] UI remains responsive during heavy operations
- [ ] Graceful degradation if resources constrained
- [ ] Performance metrics logged for debugging

## Technical Architecture Updates

### Streaming Pipeline

```
Audio Stream
    ↓
[VAD Filter]
    ↓
[Transcription Queue] ← 30s chunks
    ↓
[Transcript Buffer]
    ↓
[Streaming Orchestrator]
    ├─→ Has enough context? → [Initial Generation]
    └─→ New segments? → [Incremental Update]
    ↓
[Code Generator]
    ↓
[File Writer]
    ↓
[Preview Server] ← File watcher
    ↓
Live Preview UI
```

### New Modules

```
src-tauri/src/
  streaming/
    mod.rs              # Streaming orchestrator
    buffer.rs           # Transcript buffering
    triggers.rs         # Generation trigger logic
  preview/
    mod.rs              # Preview server management
    server.rs           # Embedded dev server
    watcher.rs          # File system watcher
  insights/
    mod.rs              # Real-time insights
    analyzer.rs         # Requirement analysis
    suggestions.rs      # Suggestion generation
  performance/
    mod.rs              # Performance monitoring
    metrics.rs          # Metrics collection
```

## User Experience Improvements

### 3.7 Onboarding & Setup Wizard

**First-Time User Experience**:
1. Welcome screen with video walkthrough
2. LLM provider setup (API key entry)
3. System audio configuration guide
4. Test meeting with sample transcript
5. Success confirmation

### 3.8 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + Shift + M` | Start/stop meeting |
| `Cmd/Ctrl + Shift + P` | Toggle preview |
| `Cmd/Ctrl + Shift + R` | Regenerate code |
| `Cmd/Ctrl + Shift + I` | Show insights |
| `Cmd/Ctrl + Shift + S` | Save manual edits |

### 3.9 Error Recovery

- **Network Failure**: Queue requests, retry with backoff
- **LLM Rate Limit**: Switch to fallback provider or wait
- **Invalid Code Generated**: Rollback to last good version
- **Transcription Error**: Mark segment as uncertain, continue
- **App Crash**: Auto-save state, recover on restart

## Testing Strategy

### Load Testing
- 2-hour continuous meeting simulation
- Memory leak detection (Valgrind, instruments)
- CPU profiling under load
- Concurrent meeting handling

### End-to-End Testing
- Real meetings with beta testers (10+ users)
- Different project types and complexities
- Various LLM providers
- Network condition variations

### Usability Testing
- Task completion rates
- Time-to-first-code measurement
- User satisfaction surveys
- Feature discovery metrics

## Beta Testing Program

### Recruitment
- 20-30 beta testers
- Mix of founders, developers, PMs
- Different platforms (Mac, Windows, Linux)
- Active meeting participants (2+ meetings/week)

### Feedback Collection
- In-app feedback form
- Weekly surveys
- 1-on-1 user interviews
- Usage analytics (opt-in)

### Success Metrics
- **Activation**: 70%+ complete onboarding
- **Retention**: 50%+ use for 3+ meetings
- **Satisfaction**: 4+ star rating
- **Referral**: 30%+ invite others

## Documentation Deliverables

1. **User Guide**: Complete feature documentation
2. **Video Tutorials**: Setup, first meeting, advanced features
3. **API Documentation**: For extending functionality
4. **Troubleshooting Guide**: Common issues and solutions
5. **Best Practices**: How to get best results

## Phase 3 Completion Checklist

- [ ] Real-time streaming working end-to-end
- [ ] Incremental updates reliable
- [ ] Live preview functional for web apps
- [ ] Meeting insights providing value
- [ ] Summary generation complete
- [ ] Performance targets met
- [ ] Beta testing completed (20+ testers)
- [ ] All critical bugs fixed
- [ ] Documentation complete
- [ ] Onboarding flow polished
- [ ] Ready for public release

## Post-Phase 3: Launch Preparation

### Pre-Launch Checklist
- [ ] Website with demo video
- [ ] GitHub repository public
- [ ] Installer builds for all platforms
- [ ] Auto-update system working
- [ ] Analytics integrated (privacy-respecting)
- [ ] Support channels established (Discord, GitHub Issues)
- [ ] Launch blog post written
- [ ] Social media assets prepared

### Launch Targets
- **Platform**: Product Hunt, Hacker News, /r/SideProject
- **Goal**: 500+ installs in first week
- **Metrics**: Track downloads, active users, retention
- **Feedback Loop**: Rapid iteration based on user reports

### Post-Launch Roadmap (Phase 4+)
- Mobile app (iOS/Android)
- Team collaboration features
- Custom AI model fine-tuning
- Integration with IDEs (VS Code extension)
- API for third-party integrations
- Multi-language support (Spanish, French, etc.)
- Video analysis (analyze screenshares)
- Automated testing generation
### Automation Settings (macOS)

- Auto-trigger `/meeting` is controlled by settings:
  - `auto_trigger_meeting_command` (bool, default false)
  - `auto_trigger_min_interval_seconds` (u32, default 75; clamped 30–600)
  - `auto_accept_changes` (bool, default false) → sends "y" + Return after `/meeting`
- Persistence: `.claude/.automation-state.json` stores `last_trigger_update_id` and `last_trigger_time` to avoid duplicate triggers across restarts.
- Logs: Structured messages prefixed with `AUTOMATION` for trigger, auto-accept, fallback, and errors.
- macOS permissions: AppleScript + System Events require Accessibility permissions. If keystrokes fail, guidance should be surfaced.
