# Phase 7: Intelligent Real-Time Code Generation

## Overview

Phase 7 transforms MeetingCoder from a proof-of-concept into a production-ready system with two fully automated operational modes: **Developer Mode** (collaborative work on existing repositories) and **Starter Kit Mode** (rapid prototyping from scratch). Both modes deliver real-time transcription → requirement extraction → automated code generation via Claude Code with zero manual intervention.

**Timeline**: 6 weeks
**Priority**: P0 (Core value proposition realization)
**Dependencies**:
- Phase 1 (continuous transcription) - ✅ Complete
- Phase 2 (Claude Code integration) - ⚠️ Partial (50%)
- Phase 4 (GitHub integration) - ⚠️ Partial (40%)

## Status Update (2025-11-07 - Week 3 In Progress)

### ✅ Completed (Weeks 1-3: Foundations + GitHub + Developer Mode)

**7.1.1 Enhanced `/meeting` Command Template** - ✅ Complete
- Location: `Handy/src-tauri/templates/meeting_command.md`
- Dual-mode support (Developer vs Starter Kit) with mode detection from `.claude/.meeting-state.json`
- Developer Mode: Codebase analysis, experiments folder isolation, safe file routing
- Starter Kit Mode: Vercel + Supabase + Next.js scaffolding instructions
- Build validation and error recovery (TypeScript checks, npm build, auto-retry)
- Comprehensive example workflows for both modes
- Dev server integration instructions

**7.1.2 LLM-Based Summarization** - ✅ Complete
- Location: `Handy/src-tauri/src/summarization/llm.rs`
- Secure Claude API key storage (keyring + file fallback)
- Full Claude API integration with structured JSON extraction
- Intelligent prompts for feature extraction with confidence scores
- Project type detection from first meeting segment
- Automatic fallback to heuristic agent on failure
- New settings: `use_llm_summarization`, `llm_model`
- New Tauri commands: `store_claude_api_key`, `has_claude_api_key`, `delete_claude_api_key`
- Integrated into meeting manager with smart fallback

**Implementation Files Modified**:
- ✅ `Handy/src-tauri/templates/meeting_command.md` - Enhanced with dual-mode logic
- ✅ `Handy/src-tauri/src/summarization/llm.rs` - New LLM module
- ✅ `Handy/src-tauri/src/summarization/mod.rs` - Added llm module
- ✅ `Handy/src-tauri/src/commands/llm.rs` - New API key commands
- ✅ `Handy/src-tauri/src/commands/mod.rs` - Registered llm commands
- ✅ `Handy/src-tauri/src/settings.rs` - Added LLM settings
- ✅ `Handy/src-tauri/src/managers/meeting.rs` - Integrated LLM summarization with fallback
- ✅ `Handy/src-tauri/src/lib.rs` - Registered LLM commands in invoke_handler

**7.1.3 GitHub OAuth Device Flow UI** - ✅ Complete
- Location: `Handy/src/components/settings/GitHubOAuth.tsx`
- OAuth Device Flow implementation with user code display
- Automatic token polling every 5 seconds
- Visual feedback (loading spinner, success/error states)
- Copy-to-clipboard for user code
- Auto-opens verification URL in browser
- Integrated into IntegrationsSettings with toggle for manual token entry
- Backend: `github_begin_device_auth` and `github_poll_device_token` commands
- New backend functions in `integrations/github.rs`: `begin_device_auth()`, `poll_device_token()`

**7.1.4 GitHub Branch & PR Management** - ✅ Complete (backend already existed)
- Branch creation: `create_branch()` in `integrations/github.rs`
- Branch checkout and push: `push_to_remote()`
- PR creation: `create_pull_request()` with draft support
- PR updates: `update_pull_request()` for incremental meeting updates
- PR comments: `post_pr_comment()` for meeting summaries
- Branch name generation from meeting ID
- State tracking in `.claude/.github-state.json`
- Tauri commands: `push_meeting_changes`, `create_or_update_pr`, `post_meeting_update_comment`

**7.1.5 GitHub Repo Picker** - ✅ Complete (already existed)
- Component: `GitHubRepoPicker.tsx`
- Lists user repositories via `list_github_repos` command
- Search and filter functionality
- Already integrated in IntegrationsSettings

**Implementation Files Modified (Week 2)**:
- ✅ `Handy/src-tauri/src/integrations/github.rs` - Added OAuth Device Flow functions (+85 lines)
- ✅ `Handy/src-tauri/src/commands/github.rs` - Added OAuth commands (+13 lines)
- ✅ `Handy/src-tauri/src/lib.rs` - Registered OAuth commands
- ✅ `Handy/src/components/settings/GitHubOAuth.tsx` - New OAuth UI component (150 lines)
- ✅ `Handy/src/components/settings/IntegrationsSettings.tsx` - Enhanced with OAuth + manual toggle

**7.2.1 Codebase Context Ingestion** - ✅ Complete
- Location: `Handy/src-tauri/src/codebase/analyzer.rs`
- Comprehensive repo analysis: framework detection (Next.js, React, Django, Rails, Tauri, FastAPI, Flask)
- Language detection: TypeScript, JavaScript, Python, Rust, Go, Java, Ruby, PHP, Swift
- Entry point discovery: app entry files, main components
- Directory mapping: src/, components/, pages/, lib/, etc.
- Dependency extraction: package.json, requirements.txt, Cargo.toml
- Source file counting (excludes node_modules, target, build dirs)
- Manifest saved to `.claude/.meeting-state.json` with structured data
- Auto-triggers on meeting start (both live and offline meetings)
- Background execution (non-blocking)
- New Tauri commands: `analyze_project_codebase`, `analyze_and_save_codebase`

**7.2.2 Intelligent File Isolation** - ✅ Complete
- Location: `Handy/src-tauri/src/codebase/isolation.rs`
- Generates `.claudeignore` file with framework-specific protection patterns
- Protects core files: src/, app/, components/, lib/, package.json, config files
- Protects secrets: .env*, *.key, credentials.json
- Allows safe paths: experiments/**, .claude/**, tests/**
- Creates `experiments/{meeting_id}/` directory structure (src/, tests/, docs/)
- Generates README.md with workflow instructions
- Path safety validation: `is_safe_path()`, `validate_file_operation()`
- Integrated into meeting lifecycle (auto-runs on start)
- Updated `/meeting` template with Step 7: File isolation safety
- Clear enforcement rules for Developer Mode

**Implementation Files Modified (Week 3)**:
- ✅ `Handy/src-tauri/src/codebase/analyzer.rs` - New codebase analysis module (554 lines)
- ✅ `Handy/src-tauri/src/codebase/isolation.rs` - New file isolation module (380 lines)
- ✅ `Handy/src-tauri/src/codebase/mod.rs` - Module exports (11 lines)
- ✅ `Handy/src-tauri/src/commands/codebase.rs` - New Tauri commands (26 lines)
- ✅ `Handy/src-tauri/src/commands/mod.rs` - Registered codebase module
- ✅ `Handy/src-tauri/src/managers/meeting.rs` - Integrated codebase analysis + isolation (+68 lines)
- ✅ `Handy/src-tauri/src/lib.rs` - Registered codebase commands
- ✅ `Handy/src-tauri/Cargo.toml` - Added walkdir, toml dependencies
- ✅ `Handy/src-tauri/templates/meeting_command.md` - Added file isolation step (+16 lines)

### ⏳ Pending (Weeks 3-6)

**Developer Mode Features** (Week 3 - Remaining)
- 7.2.3 Feature Branch Workflow Integration
- 7.2.4 Code-Aware Transcript Analysis

**Starter Kit Mode Features** (Week 4)
- 7.3.1 Starter Pack System (Vercel + Supabase + Next.js)
- 7.3.2 Project Type Detection
- 7.3.3 Live Dev Server Integration
- 7.3.4 Real-Time Scaffolding

**Full Automation** (Week 5)
- 7.4.1 AppleScript Terminal Automation
- 7.4.2 Auto-Accept for Claude Code Changes
- 7.4.3 Real-Time Update Streaming
- 7.4.4 Error Recovery & Validation

**UI/UX Polish** (Week 6)
- 7.5.1 Mode Selection Interface
- 7.5.2 Live Preview Window
- 7.5.3 Code Diff Viewer
- 7.5.4 Meeting Insights Panel

**Progress**: 9/23 tasks complete (39%)

### 🎯 Next Steps (Week 4 - Starter Kit Mode)

**7.3.1 Starter Pack System**
- **Goal**: Scaffold complete Vercel + Supabase + Next.js apps from scratch
- **Requirements**:
  - Create template directory structure with manifest.json
  - Include: package.json, next.config.js, tailwind.config.js, app structure
  - Supabase client setup with cookie-based auth
  - Auth pages: login, signup
  - Protected route middleware
  - Environment template (.env.local.example)
- **Acceptance Criteria**:
  - User says "I want a web app with login" → full scaffolding appears
  - Dependencies install automatically
  - App runs with `npm run dev`
- **Files to Create**:
  - `resources/templates/packs/vercel-supabase-nextjs/`
  - `project/initializer.rs` - Pack initialization logic

**7.3.2 Project Type Detection Integration**
- **Goal**: Auto-detect project type and select appropriate starter pack
- **Requirements**:
  - LLM already extracts `project_type` in first update ✅
  - Integrate with project initialization
  - Prompt user to confirm: "Detected web app with auth. Use Vercel + Supabase starter?"
- **Implementation**:
  - Hook project_type detection into meeting start
  - Auto-initialize pack based on detected type

**7.3.3 Live Dev Server Integration**
- **Goal**: Start dev servers automatically and show preview URLs
- **Requirements**:
  - Spawn dev server process after pack initialization
  - Parse stdout for "Local: http://localhost:3000"
  - Emit Tauri event: `dev-server-ready` with URL
  - Track PID in `.claude/.meeting-state.json`
  - Frontend displays "🌐 Preview available at http://localhost:3000"

**7.3.4 Real-Time Scaffolding**
- **Goal**: Generate complete components from natural language
- **Requirements**:
  - "I need a contact form" → `src/components/ContactForm.tsx`
  - "Add API endpoint for users" → `src/app/api/users/route.ts`
  - Follow Next.js App Router conventions
  - Use Tailwind CSS for styling
  - Use Supabase client for data/auth

**Why Week 4 Matters**:
- Starter Kit Mode enables **rapid prototyping** from zero to working app in minutes
- Live dev server + preview makes the experience **visual and immediate**
- Completes the "blow their mind" demo experience

---

## Goals

1. **Complete Foundational Blockers** (Weeks 1-2)
   - Implement production-ready `/meeting` command with dual-mode support
   - Upgrade to LLM-based requirement extraction (Claude API)
   - Complete GitHub OAuth UI and PR management

2. **Developer-to-Developer Mode** (Week 3)
   - Enable collaborative coding on existing repositories
   - Isolate new code in experiments folders by default
   - Create feature branches and draft PRs automatically
   - Correlate transcript mentions with codebase files

3. **Starter Kit Mode** (Week 4)
   - Scaffold complete Vercel + Supabase + Next.js apps from scratch
   - Auto-detect project type from conversation
   - Start dev servers and provide live previews
   - Generate production-ready components from natural language

4. **Full Automation Pipeline** (Week 5)
   - Zero manual intervention from meeting start to working code
   - Automated Claude Code triggering via AppleScript
   - Auto-accept changes with safety guardrails
   - Stream code generation progress to frontend
   - Automatic error detection and recovery

5. **Production-Ready UX** (Week 6)
   - Clear mode selection at meeting start
   - Split-screen live preview
   - Code diff viewer with syntax highlighting
   - Proactive insights panel (suggestions, warnings, questions)

---

## Success Criteria

### Developer Mode
- ✅ User starts meeting with existing repo attached
- ✅ Code appears in `experiments/{meeting_id}/` folder without touching core files
- ✅ Feature branch `discovery/{meeting_id}` created automatically
- ✅ Draft PR opened with meeting context and updated incrementally
- ✅ Build succeeds without errors
- ✅ Integration instructions provided in experiments README

### Starter Kit Mode
- ✅ User starts meeting, says "I want a web app with login"
- ✅ Vercel + Supabase + Next.js app scaffolds automatically
- ✅ Dependencies install without manual intervention
- ✅ Dev server starts and URL appears in UI
- ✅ Login page appears live at `http://localhost:3000/login`
- ✅ User can see code being generated in real-time
- ✅ App builds successfully and is deployment-ready

### Automation Quality
- ✅ Zero manual steps from meeting start to working code (except mode selection)
- ✅ Updates appear within 30-60 seconds of discussion
- ✅ Generated code passes linting and type checking
- ✅ Build errors automatically detected and fixed (max 3 retries)
- ✅ Fallback to heuristic summarization if LLM fails

### Performance Metrics
- Time from discussion to code appearance: < 90 seconds
- LLM summarization accuracy: > 80% of discussed features extracted
- Build success rate: > 90% (first attempt or after auto-retry)
- Developer Mode safety: 0 modifications to core files without explicit mention

---

## Architecture

### Dual-Mode Operation

```
┌─────────────────────────────────────────────────────────────────┐
│                      MeetingCoder Phase 7                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ Mode Selection   │
                    │ (at meeting start)│
                    └──────────────────┘
                              │
                 ┌────────────┴────────────┐
                 ▼                         ▼
        ┌─────────────────┐      ┌─────────────────────┐
        │ Developer Mode  │      │ Starter Kit Mode    │
        └─────────────────┘      └─────────────────────┘
                 │                         │
                 ▼                         ▼
    ┌─────────────────────────┐  ┌──────────────────────────┐
    │ Existing Repo Context   │  │ Scaffold New Project     │
    │ - Analyze structure     │  │ - Vercel + Supabase      │
    │ - Detect framework      │  │ - Next.js 14 App Router  │
    │ - Map file paths        │  │ - Tailwind CSS + TS      │
    └─────────────────────────┘  └──────────────────────────┘
                 │                         │
                 ▼                         ▼
    ┌─────────────────────────┐  ┌──────────────────────────┐
    │ experiments/{id}/       │  │ src/components/          │
    │ - New features isolated │  │ src/app/                 │
    │ - Safe integration      │  │ src/lib/                 │
    └─────────────────────────┘  └──────────────────────────┘
                 │                         │
                 └────────────┬────────────┘
                              ▼
                  ┌────────────────────────┐
                  │ Common Pipeline        │
                  │ - LLM Summarization    │
                  │ - Claude Code /meeting │
                  │ - Validation & Build   │
                  │ - Error Recovery       │
                  │ - GitHub PR Updates    │
                  └────────────────────────┘
```

### LLM Summarization Pipeline

```
Transcript Segments
       │
       ▼
┌──────────────────────┐
│ LLM Summarization    │  ← Claude API with structured prompts
│ (use_llm_summarization = true)
└──────────────────────┘
       │
       ├─ Success → Structured Output (features, decisions, questions)
       │
       └─ Failure → ┌──────────────────────┐
                    │ Heuristic Fallback   │  ← Keyword-based agent
                    └──────────────────────┘
                              │
                              ▼
                  ┌────────────────────────┐
                  │ .meeting-updates.jsonl │
                  └────────────────────────┘
                              │
                              ▼
                  ┌────────────────────────┐
                  │ /meeting command       │
                  │ (Claude Code)          │
                  └────────────────────────┘
```

### State Management

```
.claude/
├── .meeting-state.json
│   ├── mode: "developer" | "starter_kit"
│   ├── last_processed_update_id
│   ├── project_metadata
│   │   ├── framework
│   │   ├── languages
│   │   ├── entry_points
│   │   └── safe_paths
│   ├── failed_features
│   └── build_status
│
├── .github-state.json
│   ├── branch: "discovery/{meeting_id}"
│   ├── pr_number
│   └── pr_url
│
└── commands/
    └── meeting.md  ← Enhanced template with dual-mode logic
```

---

## Features & Requirements

### 7.1 Foundational Blockers (Weeks 1-2)

#### 7.1.1 Enhanced `/meeting` Command Template ✅

**Status**: ✅ Complete (2025-11-07)

**Implementation**: `Handy/src-tauri/templates/meeting_command.md`

**What Was Implemented**:
1. **Dual-Mode Detection**
   - Reads `mode` from `.claude/.meeting-state.json`
   - Defaults to `"starter_kit"` for new projects
   - Behavior adapts based on mode setting

2. **Developer Mode Features**
   - Step 3b: Analyze existing codebase structure
   - Detect framework from package.json, tsconfig.json
   - Extract entry points and key directories
   - Write `project_metadata` to state
   - Create `experiments/{meeting_id}/` for new features
   - Only modify existing files when explicitly mentioned
   - Preserve existing patterns and styles

3. **Starter Kit Mode Features**
   - Vercel + Supabase + Next.js scaffolding instructions
   - `npx create-next-app@latest --typescript --tailwind --app`
   - Supabase client setup with cookie-based auth
   - Auth pages: login, signup
   - Protected route middleware
   - Environment template (.env.local.example)

4. **Code Generation Guidelines**
   - TypeScript strict mode (no `any` types)
   - Error boundaries for async operations
   - Loading states for UI
   - Accessibility (semantic HTML, ARIA)
   - Performance (React.memo where appropriate)

5. **Validation & Error Recovery**
   - TypeScript compiler checks (`tsc --noEmit`)
   - Import verification
   - Missing dependency detection
   - Build validation (Next.js/Vite)
   - Auto-fix common errors (install deps, fix imports)
   - Retry logic (max 3 attempts for validation, 2 for builds)
   - Track `failed_features` in state

6. **Enhanced State Tracking**
   - `mode`, `project_metadata`, `failed_features`, `build_status`
   - Detailed summary format with status indicators (✅/⚠️/❌)
   - File counts and line diffs

7. **Example Workflows**
   - Starter Kit: First invocation + subsequent updates
   - Developer Mode: First invocation + subsequent updates
   - Shows realistic output with specific file names and line counts

**Testing Status**: Manual testing required

---

#### 7.1.2 LLM-Based Summarization ✅

**Status**: ✅ Complete (2025-11-07)

**Implementation**:
- `Handy/src-tauri/src/summarization/llm.rs` (new, 390 lines)
- `Handy/src-tauri/src/commands/llm.rs` (new, 14 lines)
- `Handy/src-tauri/src/settings.rs` (modified, +2 settings)
- `Handy/src-tauri/src/managers/meeting.rs` (modified, integration)

**What Was Implemented**:

1. **Secure API Key Management**
   ```rust
   pub fn store_api_key(api_key: &str) -> Result<()>
   pub fn get_api_key() -> Result<String>
   pub fn delete_api_key() -> Result<()>
   pub fn has_api_key() -> bool
   ```
   - Uses keyring (macOS Keychain, Windows Credential Manager)
   - Fallback to `~/.handy/.claude-api-key` file
   - Service: `com.meetingcoder.app`
   - Account: `claude_api_key`

2. **Claude API Client**
   ```rust
   pub async fn call_claude_api(
       model: &str,
       system_prompt: &str,
       user_prompt: &str,
   ) -> Result<String>
   ```
   - Full HTTP client using `reqwest`
   - Proper headers (x-api-key, anthropic-version, content-type)
   - Error handling with status codes
   - JSON request/response parsing

3. **Structured Extraction**
   ```rust
   pub struct ExtractionResult {
       pub new_features: Vec<ExtractedFeature>,
       pub technical_decisions: Vec<String>,
       pub questions: Vec<String>,
       pub project_type: Option<String>,  // First update only
   }

   pub struct ExtractedFeature {
       pub title: String,
       pub description: String,
       pub priority: String,  // "high", "medium", "low"
       pub confidence: f64,   // 0.0-1.0
   }
   ```

4. **Intelligent Prompts**
   - System prompt: Expert requirement extraction assistant
   - Guidelines: Actionable requirements, priority inference, deduplication
   - User prompt: Transcript + JSON schema
   - First update: Includes `project_type` detection
   - Output: Valid JSON only (no markdown)

5. **Integration with Meeting Manager**
   ```rust
   // In meeting.rs update loop
   let summary = if settings_now.use_llm_summarization && has_api_key() {
       match summarize_with_llm(...).await {
           Ok(summary) => summary,
           Err(e) => {
               log::warn!("LLM failed: {}, falling back", e);
               heuristic_summarize(...)
           }
       }
   } else {
       heuristic_summarize(...)
   };
   ```
   - Checks settings flag
   - Verifies API key exists
   - Automatic fallback on failure
   - Comprehensive logging

6. **New Settings**
   - `use_llm_summarization: bool` - Default: `false`
   - `llm_model: String` - Default: `"claude-sonnet-4-5-20250929"`
   - Persisted in `settings_store.json`

7. **New Tauri Commands**
   ```typescript
   await invoke('store_claude_api_key', { apiKey: 'sk-...' });
   const hasKey = await invoke('has_claude_api_key');
   await invoke('delete_claude_api_key');
   ```

**Testing Status**: Manual testing required
- Test API key storage/retrieval
- Test Claude API calls
- Test fallback to heuristic agent
- Test first update project type detection

---

#### 7.1.3 GitHub OAuth Device Flow UI ⏳

**Status**: ⏳ Not Started

**Requirements**:
1. Settings panel with "GitHub Integration" section
2. OAuth Device Flow implementation:
   - Click "Connect GitHub" button
   - Display user code + verification URL
   - Poll for token approval
   - Store token in keyring
3. Show connection status (connected account name)
4. "Disconnect" button to remove token
5. Backend already exists: `commands::github::{set,remove,test}_github_token`
6. Need frontend UI components

**Files to Create/Modify**:
- `Handy/src/components/settings/GitHubIntegration.tsx` (new)
- `Handy/src/components/settings/Settings.tsx` (add GitHub section)

**Acceptance Criteria**:
- User can authenticate with GitHub via Device Flow
- Token stored securely in system keyring
- Connection status visible in Settings
- Can disconnect and reconnect

---

#### 7.1.4 GitHub Branch & PR Management ⏳

**Status**: ⏳ Not Started (backend 60% complete)

**Existing Backend** (`commands/github.rs`):
- ✅ Token storage (keyring + fallback)
- ✅ Repo cloning: `ensure_local_repo_clone(owner, repo, token)`
- ✅ State tracking: `.claude/.github-state.json`
- ❌ Branch creation/management
- ❌ PR creation/updating via GitHub API

**Requirements**:
1. **Branch Management**
   ```rust
   pub async fn create_feature_branch(
       meeting_id: &str,
       pattern: &str,  // "discovery/{meeting_id}"
   ) -> Result<String>
   pub async fn push_branch(branch: &str) -> Result<()>
   ```

2. **PR Management**
   ```rust
   pub async fn create_draft_pr(
       owner: &str,
       repo: &str,
       branch: &str,
       title: &str,
       body: &str,
   ) -> Result<PullRequest>

   pub async fn update_pr_body(pr_number: u64, body: &str) -> Result<()>
   ```

3. **Integration with Meeting Flow**
   - On meeting start (developer mode): Create `discovery/{meeting_id}` branch
   - After first code commit: Create draft PR
   - On each update: Push commits, update PR body with meeting context

**Files to Modify**:
- `Handy/src-tauri/src/integrations/github.rs` (add branch/PR functions)
- `Handy/src-tauri/src/commands/github.rs` (expose commands)
- `Handy/src-tauri/src/managers/meeting.rs` (integrate with meeting lifecycle)

**Acceptance Criteria**:
- Meeting starts → branch created automatically
- First commit → draft PR opened
- Subsequent commits → PR body updated with features/decisions
- PR URL visible in UI

---

### 7.2 Developer Mode Features (Week 3)

#### 7.2.1 Codebase Context Ingestion ⏳

**Status**: ⏳ Not Started (template includes instructions)

**Requirements**:
1. On meeting start in developer mode:
   - Scan repository structure
   - Detect framework (Next.js, React, Vue, etc.) from package.json
   - Identify languages (TypeScript, JavaScript)
   - Find entry points (src/app/layout.tsx, src/main.tsx)
   - List key directories (src/components/, src/lib/)

2. Write analysis to `.claude/.meeting-state.json`:
   ```json
   {
     "project_metadata": {
       "framework": "nextjs",
       "languages": ["typescript", "tsx"],
       "entry_points": ["src/app/layout.tsx"],
       "key_directories": ["src/components/", "src/app/"],
       "safe_paths": ["experiments/"]
     }
   }
   ```

3. Include in first `.meeting-updates.jsonl` record as system message

**Implementation Approach**:
- Rust module: `Handy/src-tauri/src/analysis/codebase.rs`
- Functions: `analyze_repository(path) -> ProjectMetadata`
- Called from meeting manager on first update

---

#### 7.2.2 Intelligent File Isolation ⏳

**Status**: ⏳ Not Started (template includes behavior)

**Requirements**:
1. Default to `experiments/{meeting_id}/` for new code
2. Only modify existing files when:
   - Explicitly mentioned in transcript
   - `target_files` field specified in update
   - Integration explicitly requested
3. Add `.claudeignore` to protect sensitive files:
   ```
   .git/
   node_modules/
   .env
   .env.local
   package.json
   package-lock.json
   ```

**Implementation**:
- Update `/meeting` command template (already done ✅)
- Add `.claudeignore` creation to project initializer
- Teach LLM to detect file mentions from transcript

---

#### 7.2.3 Feature Branch Workflow ⏳

**Status**: ⏳ Not Started (depends on 7.1.4)

**Requirements**:
1. Branch pattern: `discovery/{meeting_id}`
2. Never commit to default branch (main/master)
3. Auto-push after each commit
4. Track branch in `.claude/.github-state.json`

**Implementation**:
- Reuse 7.1.4 branch management
- Add to meeting lifecycle hooks

---

#### 7.2.4 Code-Aware Transcript Analysis ⏳

**Status**: ⏳ Not Started

**Requirements**:
1. Enhance LLM summarization prompt with codebase manifest
2. Extract file references from transcript:
   - Regex patterns: `*.tsx`, `*.py`, class names
   - Component names mentioned
3. Add `target_files` field to update records:
   ```json
   {
     "update_id": "u5",
     "target_files": ["src/components/UserProfile.tsx"],
     "new_features_structured": [...]
   }
   ```

**Implementation**:
- Modify `summarization/llm.rs` to include file context in prompt
- Parse file mentions from LLM response
- Add `target_files` to `SummarizationOutput`

---

### 7.3 Starter Kit Mode Features (Week 4)

#### 7.3.1 Starter Pack System ⏳

**Status**: ⏳ Not Started

**Requirements**:
1. Create template directory structure:
   ```
   Handy/src-tauri/resources/templates/packs/
   └── vercel-supabase-nextjs/
       ├── manifest.json
       ├── package.json
       ├── next.config.js
       ├── tailwind.config.js
       ├── src/
       │   ├── app/
       │   │   ├── layout.tsx
       │   │   ├── page.tsx
       │   │   ├── login/
       │   │   └── signup/
       │   └── lib/
       │       └── supabase.ts
       └── .env.local.example
   ```

2. Manifest schema:
   ```json
   {
     "name": "Vercel + Supabase + Next.js",
     "description": "Full-stack web app with auth",
     "install": "npm install",
     "dev": "npm run dev",
     "build": "npm run build",
     "port": 3000,
     "safePaths": ["src/components/", "src/app/", "src/lib/"],
     "previewUrlHint": "http://localhost:3000"
   }
   ```

3. Pack copying logic:
   ```rust
   pub fn initialize_with_pack(
       meeting_id: &str,
       pack_id: &str,
   ) -> Result<String>
   ```

**Implementation**:
- Create pack template files
- Add pack initialization to `project/initializer.rs`
- Bundle with Tauri resources

---

#### 7.3.2 Project Type Detection ⏳

**Status**: ⏳ Partially implemented in LLM (needs integration)

**Requirements**:
1. LLM extracts `project_type` from first 2-3 minutes:
   - `web_app`, `mobile_app`, `api_backend`, `cli_tool`, `other`
2. Write to first `.meeting-updates.jsonl` record
3. `/meeting` command uses to select starter pack
4. Optionally prompt user to confirm: "Detected web app with auth. Use Vercel + Supabase starter?"

**Implementation**:
- ✅ LLM already extracts `project_type` in first update
- ⏳ Need integration with project initialization
- ⏳ Need confirmation UI

---

#### 7.3.3 Live Dev Server Integration ⏳

**Status**: ⏳ Not Started

**Requirements**:
1. Spawn dev server process after pack initialization:
   ```rust
   pub async fn start_dev_server(
       project_path: &str,
       command: &str,  // "npm run dev"
   ) -> Result<DevServer>

   pub struct DevServer {
       pub pid: u32,
       pub port: u16,
       pub url: String,
   }
   ```

2. Parse stdout for "Local: http://localhost:3000"
3. Emit Tauri event: `dev-server-ready` with URL
4. Track in `.claude/.meeting-state.json`:
   ```json
   {
     "dev_server": {
       "pid": 12345,
       "port": 3000,
       "url": "http://localhost:3000"
     }
   }
   ```

5. Frontend displays "🌐 Preview available at http://localhost:3000"

**Implementation**:
- Rust module: `Handy/src-tauri/src/dev_server/manager.rs`
- Process spawning with output capture
- Event emission

---

#### 7.3.4 Real-Time Scaffolding ⏳

**Status**: ⏳ Not Started (template includes instructions)

**Requirements**:
1. `/meeting` command generates complete components:
   - "I need a contact form" → `src/components/ContactForm.tsx`
   - "Add API endpoint for users" → `src/app/api/users/route.ts`
2. Follow Next.js App Router conventions
3. Use Tailwind CSS for styling
4. Use Supabase client for data/auth
5. TypeScript strict mode

**Implementation**:
- ✅ Template already includes these instructions
- ⏳ Need testing with real meetings

---

### 7.4 Full Automation (Week 5)

#### 7.4.1 AppleScript Terminal Automation ⏳

**Status**: ⏳ Partially implemented (logic exists, gated by setting)

**Existing Code**: `Handy/src-tauri/src/automation/claude_trigger.rs`

**Requirements**:
1. Enable by default in settings
2. Trigger `/meeting` every `auto_trigger_min_interval_seconds` (default 30s)
3. AppleScript workflow:
   - Find Terminal.app window with project path
   - If not found, launch new Terminal, cd to project
   - Send `/meeting` + Enter
   - Wait for completion (monitor `.meeting-state.json` update)
4. Guard: Don't trigger if previous command still running

**Implementation**:
- ✅ Backend logic exists
- ⏳ Need to enable by default
- ⏳ Need UI toggle

---

#### 7.4.2 Auto-Accept for Claude Code Changes ⏳

**Status**: ⏳ Not Started (setting exists)

**Requirements**:
1. Add `auto_accept_changes: true` to default settings
2. Modify `/meeting` command to skip confirmations
3. Safety limits:
   - Only auto-accept in safe paths (experiments/, src/)
   - Require manual approval for core file changes
4. Emergency stop button in UI

**Implementation**:
- Modify `/meeting` template to use Claude Code auto-approve flags
- Add safety path checking
- Frontend "Pause Automation" button

---

#### 7.4.3 Real-Time Update Streaming ⏳

**Status**: ⏳ Not Started

**Requirements**:
1. Capture stdout/stderr from `/meeting` command process
2. Parse for file changes: "Writing to src/components/Button.tsx"
3. Emit Tauri event: `code-generation-progress`
   ```json
   {
     "file_path": "src/components/Button.tsx",
     "status": "writing",
     "line_count": 42
   }
   ```
4. Frontend shows live indicator: "Generating Button.tsx..."
5. Progress bar: "Processing update 5/12"

**Implementation**:
- Modify AppleScript to capture process output
- Parse output for progress indicators
- Event streaming to frontend

---

#### 7.4.4 Error Recovery & Validation ⏳

**Status**: ⏳ Partially implemented in template

**Requirements**:
1. After each code generation cycle:
   - Run `npm run build` (or `tsc --noEmit`)
   - If build fails: capture errors
   - Feed errors back to Claude Code with retry prompt
   - Limit retries to 3 per feature
2. Validation checks:
   - Syntax errors (TypeScript compiler)
   - Missing dependencies → auto-run `npm install {package}`
   - Import errors → correct import paths
3. Display errors in Meeting Insights panel
4. "Fix Automatically" button for manual intervention

**Implementation**:
- ✅ Template includes validation logic
- ⏳ Need build runner integration
- ⏳ Need error feedback loop

---

### 7.5 UI/UX Polish (Week 6)

#### 7.5.1 Mode Selection Interface ⏳

**Status**: ⏳ Not Started

**Requirements**:
1. "New Meeting" modal with two cards:
   - **Developer Mode**: "Working on existing repo" → GitHub repo picker
   - **Starter Kit Mode**: "Starting from scratch" → starter pack selector
2. Persist choice in `.claude/.meeting-state.json`: `mode: "developer" | "starter_kit"`
3. Settings: default mode preference

**Implementation**:
- Frontend component: `MeetingModeSelector.tsx`
- Integrate with meeting start flow

---

#### 7.5.2 Live Preview Window ⏳

**Status**: ⏳ Not Started

**Requirements**:
1. Split-screen layout:
   - Left pane (60%): LiveTranscript + MeetingUpdates
   - Right pane (40%): iframe with dev server preview
2. Preview controls:
   - Refresh button
   - Open in browser
   - Responsive mode toggle (mobile/tablet/desktop)
3. Only show in Starter Kit mode

**Implementation**:
- Modify `MeetingControls.tsx` layout
- Add iframe component
- Responsive design

---

#### 7.5.3 Code Diff Viewer ⏳

**Status**: ⏳ Not Started

**Requirements**:
1. Use `react-diff-viewer` library
2. Display in MeetingUpdates panel
3. Collapsible sections per file: "src/components/Button.tsx (+15, -3)"
4. Syntax highlighting for TypeScript/JSX
5. Toggle: "Show Diffs" checkbox in Settings

**Implementation**:
- Install `react-diff-viewer`
- Add `DiffViewer.tsx` component
- Capture diffs from git

---

#### 7.5.4 Meeting Insights Panel ⏳

**Status**: ⏳ Not Started

**Requirements**:
1. Right sidebar (collapsible) with:
   - **Suggestions**: "Detected database query, consider adding indexing"
   - **Warnings**: "Build failed, retrying with error context"
   - **Questions**: "Unclear requirement: which page should the form go on?"
2. Powered by LLM analysis of transcript + codebase state
3. Actionable items: click to add comment to PR

**Implementation**:
- Backend: Enhance LLM summarization to extract insights
- Frontend: `MeetingInsights.tsx` component

---

## Implementation Timeline

### Week 1 (Nov 4-8) - Foundational Blockers ✅
- [x] 7.1.1 Enhanced `/meeting` command template
- [x] 7.1.2 LLM-based summarization with Claude API

### Week 2 (Nov 11-15) - GitHub Integration ⏳
- [ ] 7.1.3 GitHub OAuth Device Flow UI
- [ ] 7.1.4 GitHub branch & PR management

### Week 3 (Nov 18-22) - Developer Mode
- [ ] 7.2.1 Codebase context ingestion
- [ ] 7.2.2 Intelligent file isolation
- [ ] 7.2.3 Feature branch workflow
- [ ] 7.2.4 Code-aware transcript analysis

### Week 4 (Nov 25-29) - Starter Kit Mode
- [ ] 7.3.1 Starter pack system (Vercel + Supabase + Next.js)
- [ ] 7.3.2 Project type detection integration
- [ ] 7.3.3 Live dev server integration
- [ ] 7.3.4 Real-time scaffolding

### Week 5 (Dec 2-6) - Full Automation
- [ ] 7.4.1 AppleScript terminal automation
- [ ] 7.4.2 Auto-accept for Claude Code changes
- [ ] 7.4.3 Real-time update streaming
- [ ] 7.4.4 Error recovery & validation

### Week 6 (Dec 9-13) - UI/UX Polish
- [ ] 7.5.1 Mode selection interface
- [ ] 7.5.2 Live preview window
- [ ] 7.5.3 Code diff viewer
- [ ] 7.5.4 Meeting insights panel

---

## Testing Strategy

### Unit Tests
- LLM API client (mock responses)
- Codebase analysis (sample repos)
- Branch/PR management (GitHub API mocks)

### Integration Tests
- End-to-end developer mode flow (existing repo)
- End-to-end starter kit mode flow (new project)
- Fallback to heuristic agent
- Error recovery loops

### Manual Testing
- Live meeting with real audio
- Multiple meeting sessions
- Build failures and retries
- Different project types

### Performance Testing
- 60+ minute meetings
- 100+ transcript segments
- Large codebases (1000+ files)
- LLM API rate limiting

---

## Risks & Mitigations

### Risk: LLM API Costs
- **Mitigation**: Make LLM optional (default: off), use heuristic agent
- **Monitoring**: Track API usage in settings
- **Fallback**: Always fall back to heuristic if API fails

### Risk: Build Failures in Real-Time
- **Mitigation**: Retry logic (max 3 attempts), show clear errors
- **Fallback**: Continue with next updates, flag failures in state

### Risk: Code Quality from Natural Language
- **Mitigation**: Validation checks, TypeScript strict mode, linting
- **Safety**: Developer mode uses experiments folder

### Risk: Automation Too Aggressive
- **Mitigation**: Emergency stop button, require approval for core files
- **Transparency**: Show all changes in PR, diffs visible

---

## Success Metrics

### Quantitative
- **Completion Rate**: 100% of features (23/23) implemented
- **Test Coverage**: >80% for critical paths
- **Build Success**: >90% on first attempt (or after auto-retry)
- **LLM Accuracy**: >80% of discussed features extracted
- **Performance**: <90s from discussion to code

### Qualitative
- **Developer Mode**: Code appears in experiments, no core files touched
- **Starter Kit Mode**: Full app scaffolded and running within 2 minutes
- **User Feedback**: "This feels like magic" reactions
- **Code Quality**: Generated code is production-ready (linted, typed, tested)

---

## Dependencies

### Phase 2 Dependencies (Partially Complete)
- ✅ `.meeting-updates.jsonl` protocol
- ✅ `/meeting` command template
- ⚠️ LLM-based summarization (now complete in Phase 7)
- ❌ Automation trigger (planned for Phase 7.4.1)

### Phase 4 Dependencies (Partially Complete)
- ✅ GitHub token storage
- ✅ Repo cloning
- ❌ OAuth Device Flow UI (planned for Phase 7.1.3)
- ❌ Branch/PR management (planned for Phase 7.1.4)

### External Dependencies
- Claude API (Anthropic)
- GitHub API
- Next.js, Supabase (starter pack)
- Node.js, npm (dev environment)

---

## Acceptance Criteria Summary

Phase 7 is **complete** when:

1. ✅ Both modes work end-to-end without manual intervention
2. ✅ Developer mode: code in experiments, PR created, builds successfully
3. ✅ Starter Kit mode: app scaffolded, dev server running, live preview visible
4. ✅ LLM summarization extracts >80% of features accurately
5. ✅ Automation triggers every 30-60s during meetings
6. ✅ Build errors detected and fixed automatically
7. ✅ UI shows real-time progress and allows emergency stop
8. ✅ All 23 tasks marked complete in todo list

**Current Status**: 5/23 tasks complete (22%), Weeks 1-2 complete (foundations + GitHub integration)

---

## Next Steps

**Immediate (Week 2)**:
1. Implement GitHub OAuth Device Flow UI
2. Add branch creation and PR management
3. Test GitHub integration end-to-end

**Then (Weeks 3-6)**:
- Developer mode features
- Starter kit mode features
- Full automation
- UI/UX polish

**Documentation**:
- Update README with Phase 7 features
- Create user guide for both modes
- Video demos of both user stories
