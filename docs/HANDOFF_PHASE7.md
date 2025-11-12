# Phase 7 Development Handoff - Week 3+ Ready

**Date**: November 7, 2025
**Status**: Weeks 1-2 Complete (22% done - 5/23 tasks)
**Next**: Week 3 - Developer Mode Features
**Project**: MeetingCoder - Real-Time Code Generation from Meetings

---

## Executive Summary

You're taking over **Phase 7** of MeetingCoder, a system that transforms meeting transcripts into working code in real-time. The goal: developers start a meeting, discuss features, and **half the work is done before they touch their IDE**.

**What works now**:
- ✅ Real-time transcription (Phase 1)
- ✅ LLM-based requirement extraction with Claude API
- ✅ Enhanced `/meeting` command template (dual-mode: Developer vs Starter Kit)
- ✅ GitHub OAuth Device Flow integration
- ✅ Automatic branch creation and PR management

**What you're building next**:
- Week 3: Developer Mode (codebase analysis, file isolation, code-aware AI)
- Week 4: Starter Kit Mode (Vercel + Supabase + Next.js scaffolding)
- Week 5: Full automation (AppleScript, auto-accept, error recovery)
- Week 6: UI polish (mode selector, live preview, diff viewer)

---

## Critical Context: How MeetingCoder Works

### Architecture Flow

```
1. User starts meeting (Tauri app)
   ↓
2. System audio captured (BlackHole virtual device on macOS)
   ↓
3. Whisper transcribes in 10s chunks
   ↓
4. Every 20-60s: Segments sent to LLM (Claude API)
   ↓
5. LLM extracts structured requirements (features, decisions, questions)
   ↓
6. Written to `.meeting-updates.jsonl` (append-only JSONL)
   ↓
7. (Optional) AppleScript triggers `/meeting` command in Terminal
   ↓
8. Claude Code reads `.meeting-updates.jsonl` via `/meeting` slash command
   ↓
9. Generates code based on mode (Developer vs Starter Kit)
   ↓
10. Auto-commit → Auto-push → Auto-create/update PR
```

### Two Operational Modes

**Developer Mode**:
- Work on existing repository
- Code goes to `experiments/{meeting_id}/` by default
- Only touch core files when explicitly mentioned
- Branch: `discovery/{meeting_id}`
- PR: Draft with meeting context

**Starter Kit Mode**:
- Start from scratch (Vercel + Supabase + Next.js)
- Scaffold complete app structure
- Code goes to `src/components/`, `src/app/`, etc.
- Dev server auto-starts
- Live preview in UI

### Key Files & Data Flow

**Meeting Session**:
```
~/MeetingCoder/
  projects/{name}/              # Or repos/{owner}/{repo}/
    .meeting-updates.jsonl      # Structured requirements (append-only)
    .transcript.jsonl           # Raw transcript segments
    .claude/
      .meeting-state.json       # Processing state
      .github-state.json        # Branch/PR tracking
      commands/
        meeting.md              # Slash command template
```

**State Schema** (`.claude/.meeting-state.json`):
```json
{
  "last_processed_update_id": "u12",
  "last_processed_timestamp": "2025-11-07T15:30:00Z",
  "total_updates_processed": 12,
  "mode": "developer",  // or "starter_kit"
  "project_metadata": {
    "framework": "nextjs",
    "languages": ["typescript", "tsx"],
    "entry_points": ["src/app/layout.tsx"],
    "key_directories": ["src/components/", "src/app/"],
    "safe_paths": ["experiments/"]
  },
  "failed_features": [],
  "build_status": "success"
}
```

---

## What's Been Built (Weeks 1-2)

### Week 1: Foundations ✅

**1. Enhanced `/meeting` Command Template**
- **Location**: `Handy/src-tauri/templates/meeting_command.md`
- **What it does**: Provides instructions for Claude Code to read `.meeting-updates.jsonl` and generate code
- **Key features**:
  - Dual-mode support (reads `mode` from state)
  - Developer Mode: Creates `experiments/{meeting_id}/`, analyzes codebase
  - Starter Kit Mode: Scaffolds Vercel + Supabase + Next.js
  - Build validation (TypeScript, npm build)
  - Error recovery with retries
  - Comprehensive example workflows

**2. LLM-Based Summarization**
- **Location**: `Handy/src-tauri/src/summarization/llm.rs`
- **What it does**: Calls Claude API to extract features, decisions, questions from transcript
- **Key features**:
  - Secure API key storage (keyring + fallback)
  - Structured JSON extraction with confidence scores
  - Project type detection (first update only)
  - Automatic fallback to heuristic agent
  - Settings: `use_llm_summarization`, `llm_model`
- **Commands**: `store_claude_api_key`, `has_claude_api_key`, `delete_claude_api_key`
- **Integration**: Meeting manager calls `summarize_with_llm()` every 20-60s

### Week 2: GitHub Integration ✅

**3. GitHub OAuth Device Flow**
- **Backend**: `Handy/src-tauri/src/integrations/github.rs`
  - `begin_device_auth()` - Initiates OAuth, returns user code
  - `poll_device_token()` - Polls for token approval
- **Commands**: `github_begin_device_auth`, `github_poll_device_token`
- **Frontend**: `Handy/src/components/settings/GitHubOAuth.tsx`
  - Beautiful UI with user code display
  - Auto-opens verification URL
  - Polling with loading states
  - Success/error feedback

**4. GitHub Branch & PR Management**
- **Backend**: Already in `integrations/github.rs`
  - `create_branch()` - Creates `discovery/{meeting_id}` branch
  - `push_to_remote()` - Pushes commits
  - `create_pull_request()` - Creates draft PRs
  - `update_pull_request()` - Updates PR body
  - `post_pr_comment()` - Posts meeting updates
- **Commands**: `push_meeting_changes`, `create_or_update_pr`, `post_meeting_update_comment`
- **State**: `.claude/.github-state.json` tracks branch, PR number, last push

**5. Repo Picker** (already existed)
- **Component**: `GitHubRepoPicker.tsx`
- Lists user repos, search/filter

---

## What You're Building Next

### Week 3: Developer Mode Features (4 tasks)

#### 3.1 Codebase Context Ingestion (7.2.1) ⏳

**Goal**: Analyze existing repository structure so AI understands the codebase.

**What to build**:

1. **New Rust module**: `Handy/src-tauri/src/analysis/codebase.rs`

```rust
use anyhow::Result;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectMetadata {
    pub framework: Option<String>,  // "nextjs", "react", "vue", "express"
    pub languages: Vec<String>,      // ["typescript", "tsx", "javascript"]
    pub entry_points: Vec<String>,   // ["src/app/layout.tsx", "src/main.tsx"]
    pub key_directories: Vec<String>, // ["src/components/", "src/app/"]
    pub safe_paths: Vec<String>,     // ["experiments/"]
    pub package_manager: Option<String>, // "npm", "yarn", "pnpm", "bun"
}

pub fn analyze_repository(path: &Path) -> Result<ProjectMetadata> {
    // 1. Check for package.json → detect framework
    //    - "next": "nextjs"
    //    - "react": "react"
    //    - "express": "express"

    // 2. Check for tsconfig.json → languages include TypeScript

    // 3. Find entry points:
    //    - Next.js: src/app/layout.tsx or pages/_app.tsx
    //    - React: src/main.tsx or src/index.tsx
    //    - Express: src/index.ts or index.js

    // 4. List key directories (src/components/, src/lib/, etc.)

    // 5. Add experiments/ to safe_paths by default

    // Return structured metadata
}
```

2. **Integration point**: `Handy/src-tauri/src/managers/meeting.rs`
   - On first meeting update (when `last_sent_index == 0` in developer mode)
   - Call `analyze_repository(project_path)`
   - Write to `.claude/.meeting-state.json` under `project_metadata`
   - Include in first `.meeting-updates.jsonl` record as `source: "system:repo_analysis"`

3. **Update JSONL schema**: Add optional `project_metadata` field to update records

**Acceptance Criteria**:
- [ ] Detects Next.js, React, Vue, Express correctly
- [ ] Finds entry points for each framework
- [ ] Lists key directories (src/*, components/*, etc.)
- [ ] Writes metadata to state on first update
- [ ] Metadata available to `/meeting` command

---

#### 3.2 Intelligent File Isolation (7.2.2) ⏳

**Goal**: Protect core files from accidental modification.

**What to build**:

1. **Create `.claudeignore` template**: `Handy/src-tauri/resources/templates/claudeignore`

```
# MeetingCoder - Protected Files
.git/
.github/
node_modules/
.next/
dist/
build/
out/
.env
.env.local
.env.production
package.json
package-lock.json
yarn.lock
pnpm-lock.yaml
tsconfig.json
next.config.js
vite.config.ts
```

2. **Update project initializer**: `Handy/src-tauri/src/project/initializer.rs`
   - In `seed_in_existing_dir_with_app()`, copy `.claudeignore` to project root
   - Bundle with Tauri resources

3. **Enhance `/meeting` command template**:
   - Already has instructions for `experiments/{meeting_id}/`
   - Add: "Respect `.claudeignore` patterns. Never modify protected files."

**Acceptance Criteria**:
- [ ] `.claudeignore` created in all projects
- [ ] Protected files: package.json, .env, config files
- [ ] `/meeting` command respects ignore patterns

---

#### 3.3 Feature Branch Workflow (7.2.3) ⏳

**Goal**: Hook GitHub branch/PR creation into meeting lifecycle.

**What to build**:

1. **Meeting lifecycle hooks**: `Handy/src-tauri/src/managers/meeting.rs`

```rust
// In start_meeting() or first update loop
async fn handle_first_update_developer_mode(
    meeting: &Meeting,
    app: &AppHandle,
) -> Result<()> {
    let settings = settings::get_settings(app);

    if !settings.github_enabled {
        return Ok(());
    }

    // Get repo info
    let owner = settings.github_repo_owner.ok_or(...)?;
    let repo = settings.github_repo_name.ok_or(...)?;
    let token = github::get_github_token()?;

    // Ensure repo cloned
    let project_path = github::ensure_local_repo_clone(&owner, &repo, &token)?;

    // Initialize git repo
    let repo_obj = github::init_git_repo(&project_path)?;

    // Create branch
    let branch_name = github::generate_branch_name(
        &settings.github_branch_pattern,
        &meeting.id,
        &meeting.name,
    );

    github::create_branch(&repo_obj, &branch_name)?;

    // Update state
    let mut github_state = github::read_github_state(&project_path);
    github_state.last_branch = Some(branch_name.clone());
    github::write_github_state(&project_path, &github_state)?;

    Ok(())
}
```

2. **After each `/meeting` execution** (via AppleScript or manual):
   - Detect new commits: `git log HEAD^..HEAD`
   - If commits exist:
     - Push: `push_to_remote()`
     - If first commit: `create_or_update_pr()`
     - Else: `update_pull_request()` with new features from latest update

3. **State tracking**: Use `.claude/.github-state.json`

**Acceptance Criteria**:
- [ ] Branch created on first developer mode update
- [ ] Auto-push after each commit
- [ ] PR created after first commit
- [ ] PR updated with each subsequent update

---

#### 3.4 Code-Aware Transcript Analysis (7.2.4) ⏳

**Goal**: Correlate transcript mentions with actual codebase files.

**What to build**:

1. **Enhance LLM summarization prompt**: `Handy/src-tauri/src/summarization/llm.rs`

```rust
pub fn build_extraction_prompt_with_codebase(
    transcript_text: &str,
    is_first_update: bool,
    codebase_manifest: Option<&ProjectMetadata>,
) -> String {
    let mut prompt = format!(
        r#"Extract requirements from this meeting transcript segment:

<transcript>
{}
</transcript>"#,
        transcript_text
    );

    // Add codebase context if available
    if let Some(manifest) = codebase_manifest {
        prompt.push_str("\n\n<codebase_context>\n");
        prompt.push_str(&format!("Framework: {}\n", manifest.framework.as_deref().unwrap_or("unknown")));
        prompt.push_str(&format!("Languages: {}\n", manifest.languages.join(", ")));
        prompt.push_str("Key files:\n");
        for entry_point in &manifest.entry_points {
            prompt.push_str(&format!("  - {}\n", entry_point));
        }
        prompt.push_str("Key directories:\n");
        for dir in &manifest.key_directories {
            prompt.push_str(&format!("  - {}\n", dir));
        }
        prompt.push_str("</codebase_context>\n\n");
    }

    prompt.push_str(r#"
When extracting features, if the transcript mentions specific files or components (e.g., "update the UserProfile component"), include them in a "target_files" array.

Return JSON in this format:
{
  "new_features": [...],
  "technical_decisions": [...],
  "questions": [...],
  "target_files": ["src/components/UserProfile.tsx"]  // Optional, only if specific files mentioned
}
"#);

    prompt
}
```

2. **Update `SummarizationOutput`**: `Handy/src-tauri/src/summarization/agent.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizationOutput {
    pub timestamp: String,
    pub segment_range: (usize, usize),
    pub new_features: Vec<String>,
    pub technical_decisions: Vec<String>,
    pub questions: Vec<String>,
    pub new_features_structured: Vec<Feature>,
    pub modified_features: Option<HashMap<String, serde_json::Value>>,
    pub clarifications: Option<HashMap<String, String>>,
    pub target_files: Option<Vec<String>>,  // NEW: Files mentioned in transcript
}
```

3. **Write to `.meeting-updates.jsonl`**: Include `target_files` in JSONL records

**Acceptance Criteria**:
- [ ] LLM receives codebase context in prompt
- [ ] Transcript mentions like "UserProfile component" → `target_files: ["src/components/UserProfile.tsx"]`
- [ ] `/meeting` command uses `target_files` to know which files to modify

---

### Week 4: Starter Kit Mode (4 tasks)

#### 4.1 Starter Pack System (7.3.1) ⏳

**Goal**: Create Vercel + Supabase + Next.js template.

**What to build**:

1. **Template structure**: `Handy/src-tauri/resources/templates/packs/vercel-supabase-nextjs/`

```
vercel-supabase-nextjs/
├── manifest.json
├── package.json
├── next.config.js
├── tailwind.config.js
├── tsconfig.json
├── .env.local.example
├── src/
│   ├── app/
│   │   ├── layout.tsx
│   │   ├── page.tsx
│   │   ├── login/
│   │   │   └── page.tsx
│   │   └── signup/
│   │       └── page.tsx
│   ├── lib/
│   │   └── supabase.ts
│   └── middleware.ts
└── README.md
```

2. **Manifest** (`manifest.json`):

```json
{
  "name": "Vercel + Supabase + Next.js",
  "description": "Full-stack web app with authentication and database",
  "install": "npm install",
  "dev": "npm run dev",
  "build": "npm run build",
  "port": 3000,
  "safePaths": ["src/components/", "src/app/", "src/lib/"],
  "previewUrlHint": "http://localhost:3000"
}
```

3. **Pack initialization**: `Handy/src-tauri/src/project/initializer.rs`

```rust
pub fn initialize_with_pack(
    meeting_id: &str,
    pack_id: &str,
    app: &AppHandle,
) -> Result<String> {
    // 1. Resolve pack template from resources
    let pack_path = app.path().resolve(
        format!("templates/packs/{}", pack_id),
        tauri::path::BaseDirectory::Resource
    )?;

    // 2. Create project directory
    let project_dir = get_project_base()?.join(meeting_id);
    fs::create_dir_all(&project_dir)?;

    // 3. Copy pack files recursively
    copy_dir_recursive(&pack_path, &project_dir)?;

    // 4. Read manifest
    let manifest: PackManifest = read_manifest(&project_dir)?;

    // 5. Run install command
    run_command(&project_dir, &manifest.install)?;

    // 6. Initialize .claude/ scaffolding
    seed_in_existing_dir_with_app(&project_dir, app)?;

    // 7. Write mode to state
    let mut state = read_meeting_state(&project_dir);
    state.mode = Some("starter_kit".to_string());
    write_meeting_state(&project_dir, &state)?;

    Ok(project_dir.to_string_lossy().to_string())
}
```

4. **Bundle with Tauri**: Update `tauri.conf.json` to include `resources/templates/packs/**`

**Acceptance Criteria**:
- [ ] Template includes working Next.js 14 + Supabase auth
- [ ] `npm install && npm run dev` works out of the box
- [ ] Pack copying function works
- [ ] Mode set to "starter_kit" automatically

---

#### 4.2 Project Type Detection (7.3.2) ⏳

**Goal**: Auto-detect project type from conversation.

**What to build**:

1. **LLM already extracts** `project_type` in first update (Week 1 feature)
2. **Integration needed**: `Handy/src-tauri/src/managers/meeting.rs`

```rust
// After first summarization in starter kit mode
let summary = summarize_with_llm(...).await?;

// Check if project_type was detected
if let Some(project_type) = summary.project_type {
    match project_type.as_str() {
        "web_app" => {
            // Suggest Vercel + Supabase + Next.js
            // Emit event to frontend
            app.emit("project-type-detected", json!({
                "type": "web_app",
                "suggested_pack": "vercel-supabase-nextjs",
            }))?;
        },
        "api_backend" => {
            // Suggest Express or FastAPI
        },
        _ => {}
    }
}
```

3. **Frontend modal**: `Handy/src/components/meeting/ProjectTypeSuggestion.tsx`
   - Shows: "Detected web app with authentication. Use Vercel + Supabase starter?"
   - Buttons: [Yes] [Choose Different] [Skip]
   - On Yes: Call `initialize_with_pack(meeting_id, "vercel-supabase-nextjs")`

**Acceptance Criteria**:
- [ ] LLM detects "web_app", "api_backend", etc. from conversation
- [ ] Frontend shows suggestion modal
- [ ] User can accept or choose different pack

---

#### 4.3 Live Dev Server Integration (7.3.3) ⏳

**Goal**: Auto-start dev server and show preview URL.

**What to build**:

1. **Dev server manager**: `Handy/src-tauri/src/dev_server/manager.rs`

```rust
use std::process::{Command, Stdio, Child};
use anyhow::Result;

pub struct DevServer {
    pub process: Child,
    pub pid: u32,
    pub port: u16,
    pub url: String,
}

pub fn start_dev_server(
    project_path: &str,
    command: &str,  // "npm run dev"
    expected_port: u16,
) -> Result<DevServer> {
    // 1. Spawn process
    let parts: Vec<&str> = command.split_whitespace().collect();
    let mut child = Command::new(parts[0])
        .args(&parts[1..])
        .current_dir(project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let pid = child.id();

    // 2. Monitor stdout for "Local: http://localhost:3000"
    // (Use a background thread to read stdout)

    // 3. Return DevServer struct
    Ok(DevServer {
        process: child,
        pid,
        port: expected_port,
        url: format!("http://localhost:{}", expected_port),
    })
}
```

2. **Tauri command**: `Handy/src-tauri/src/commands/dev_server.rs`

```rust
#[tauri::command]
pub async fn start_dev_server_for_meeting(
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<DevServerInfo, String> {
    // Get meeting project path
    // Read manifest
    // Start dev server
    // Emit event when ready
    // Return DevServerInfo
}
```

3. **Frontend listener**: `Handy/src/components/meeting/MeetingView.tsx`
   - Listen for `dev-server-ready` event
   - Show: "🌐 Preview available at http://localhost:3000" with link

**Acceptance Criteria**:
- [ ] Dev server starts automatically after pack initialization
- [ ] URL captured and shown in UI
- [ ] Click link → opens browser

---

#### 4.4 Real-Time Scaffolding (7.3.4) ⏳

**Goal**: Test end-to-end scaffolding from conversation.

**What to test**:

1. Start meeting in Starter Kit mode
2. Say: "I need a contact form with name, email, and message fields"
3. Verify:
   - LLM extracts feature
   - Written to `.meeting-updates.jsonl`
   - `/meeting` command generates `src/components/ContactForm.tsx`
   - Includes Tailwind styling
   - Uses proper TypeScript types
   - Dev server reloads
   - Component visible in browser

**This is validation**, not new code. Just testing Week 1's `/meeting` template with Week 4's starter pack.

**Acceptance Criteria**:
- [ ] Contact form generates correctly
- [ ] Styling looks good
- [ ] TypeScript compiles
- [ ] Hot reload works

---

### Week 5: Full Automation (4 tasks)

#### 5.1 AppleScript Automation (7.4.1) ⏳

**Goal**: Auto-trigger `/meeting` command during meetings.

**What exists**:
- `Handy/src-tauri/src/automation/claude_trigger.rs` (partial implementation)
- Setting: `auto_trigger_meeting_command` (default: false)

**What to build**:

1. **Enable by default**: Change `default_auto_trigger_meeting_command()` to return `true`

2. **Trigger in update loop**: `Handy/src-tauri/src/managers/meeting.rs`

```rust
// After writing .meeting-updates.jsonl
if settings.auto_trigger_meeting_command {
    let should_trigger = last_trigger_instant.elapsed() >=
        Duration::from_secs(settings.auto_trigger_min_interval_seconds as u64);

    if should_trigger {
        // Spawn AppleScript to trigger /meeting
        automation::trigger_claude_code(&project_path, &meeting.id)?;
        last_trigger_instant = Instant::now();
    }
}
```

3. **AppleScript**: `Handy/src-tauri/src/automation/claude_trigger.rs`

```rust
pub fn trigger_claude_code(project_path: &str, meeting_id: &str) -> Result<()> {
    let script = format!(r#"
        tell application "Terminal"
            set foundWindow to false
            repeat with w in windows
                repeat with t in tabs of w
                    if processes of t contains "{}" then
                        set foundWindow to true
                        set frontmost to true
                        do script "cd {} && /meeting" in t
                        exit repeat
                    end if
                end repeat
                if foundWindow then exit repeat
            end repeat

            if not foundWindow then
                do script "cd {} && /meeting"
            end if
        end tell
    "#, meeting_id, project_path, project_path);

    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .spawn()?;

    Ok(())
}
```

**Acceptance Criteria**:
- [ ] Triggers every 30-60s during meeting
- [ ] Finds existing Terminal window or creates new one
- [ ] `/meeting` executes successfully
- [ ] UI shows "⚙️ Generating code..."

---

#### 5.2 Auto-Accept for Claude Code (7.4.2) ⏳

**Goal**: Skip manual confirmations in Claude Code.

**What to build**:

1. **Setting already exists**: `auto_accept_changes` (default: false)
   - Change default to `true`

2. **Modify `/meeting` template**: Add instruction at top

```markdown
# Auto-Accept Mode

**IMPORTANT**: This session has auto-accept enabled. Apply all changes immediately without asking for confirmation.

Safety rules:
- Auto-accept for safe paths: `experiments/`, `src/components/`, `src/app/`, `src/lib/`
- Require manual approval for: `package.json`, config files, existing core logic
```

3. **Claude Code behavior**: Claude Code should respect the template instructions

**Acceptance Criteria**:
- [ ] Changes apply automatically in safe paths
- [ ] Manual approval still required for sensitive files

---

#### 5.3 Real-Time Update Streaming (7.4.3) ⏳

**Goal**: Show live code generation progress.

**What to build**:

1. **Capture Claude Code output**: In AppleScript or separate monitor

```rust
// When triggering /meeting command
let output = Command::new("osascript")
    .arg("-e")
    .arg(&trigger_script)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

// Read stdout in background thread
let stdout = output.stdout.take().unwrap();
let reader = BufReader::new(stdout);

for line in reader.lines() {
    let line = line?;

    // Parse for progress indicators
    if line.contains("Writing to") {
        let file_path = extract_file_path(&line);
        app.emit("code-generation-progress", json!({
            "file_path": file_path,
            "status": "writing",
        }))?;
    }
}
```

2. **Frontend**: `Handy/src/components/meeting/CodeGenerationProgress.tsx`
   - Listen for `code-generation-progress` events
   - Show: "✏️ Generating Button.tsx..."
   - Progress bar: "Processing update 5/12"

**Acceptance Criteria**:
- [ ] Real-time progress shown in UI
- [ ] File names appear as they're generated
- [ ] Progress bar updates

---

#### 5.4 Error Recovery (7.4.4) ⏳

**Goal**: Detect build errors and auto-retry.

**What to build**:

1. **After `/meeting` command completes**: Run build check

```rust
pub fn validate_build(project_path: &str) -> Result<BuildResult> {
    // 1. Run `npm run build` or `tsc --noEmit`
    let output = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(project_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // 2. Parse errors
        let errors = parse_build_errors(&stderr);

        // 3. Generate retry prompt
        let retry_prompt = format!(
            "Build failed with {} errors. Fix these issues:\n{}",
            errors.len(),
            errors.join("\n")
        );

        return Ok(BuildResult::Failed { errors, retry_prompt });
    }

    Ok(BuildResult::Success)
}
```

2. **Retry logic**: If build fails, append error context to `.meeting-updates.jsonl` as a special update

```json
{
  "update_id": "error_retry_1",
  "source": "system:build_error",
  "technical_decisions": [
    "Build failed: Type 'string' is not assignable to type 'number' in Button.tsx:42"
  ]
}
```

3. **Trigger `/meeting` again** with error context

**Acceptance Criteria**:
- [ ] Build errors detected automatically
- [ ] Error details fed back to `/meeting`
- [ ] Max 3 retries per feature
- [ ] Success/failure shown in UI

---

### Week 6: UI Polish (4 tasks)

#### 6.1 Mode Selection Interface (7.5.1) ⏳

**Component**: `Handy/src/components/meeting/MeetingModeSelector.tsx`

```tsx
export const MeetingModeSelector: React.FC<{
  onSelect: (mode: 'developer' | 'starter_kit') => void;
}> = ({ onSelect }) => {
  return (
    <div className="grid grid-cols-2 gap-6">
      <div
        onClick={() => onSelect('developer')}
        className="p-6 border-2 rounded-lg cursor-pointer hover:border-blue-500"
      >
        <h3 className="text-xl font-bold mb-2">Developer Mode</h3>
        <p className="text-gray-600 mb-4">
          Working on existing repository
        </p>
        <ul className="text-sm space-y-1">
          <li>✓ Code in experiments folder</li>
          <li>✓ Safe, isolated changes</li>
          <li>✓ Auto PR creation</li>
        </ul>
      </div>

      <div
        onClick={() => onSelect('starter_kit')}
        className="p-6 border-2 rounded-lg cursor-pointer hover:border-blue-500"
      >
        <h3 className="text-xl font-bold mb-2">Starter Kit Mode</h3>
        <p className="text-gray-600 mb-4">
          Starting from scratch
        </p>
        <ul className="text-sm space-y-1">
          <li>✓ Vercel + Supabase + Next.js</li>
          <li>✓ Complete app structure</li>
          <li>✓ Live preview</li>
        </ul>
      </div>
    </div>
  );
};
```

---

#### 6.2 Live Preview Window (7.5.2) ⏳

**Component**: `Handy/src/components/meeting/LivePreview.tsx`

```tsx
export const LivePreview: React.FC<{ url: string }> = ({ url }) => {
  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between p-2 border-b">
        <span className="text-sm font-medium">{url}</span>
        <button
          onClick={() => open(url)}
          className="text-sm text-blue-600"
        >
          Open in Browser
        </button>
      </div>
      <iframe
        src={url}
        className="flex-1 w-full"
      />
    </div>
  );
};
```

---

#### 6.3 Code Diff Viewer (7.5.3) ⏳

**Install**: `npm install react-diff-viewer`

**Component**: `Handy/src/components/meeting/DiffViewer.tsx`

```tsx
import ReactDiffViewer from 'react-diff-viewer';

export const DiffViewer: React.FC<{
  file: string;
  oldCode: string;
  newCode: string;
}> = ({ file, oldCode, newCode }) => {
  return (
    <div className="border rounded-lg overflow-hidden">
      <div className="bg-gray-100 px-4 py-2 font-mono text-sm">
        {file}
      </div>
      <ReactDiffViewer
        oldValue={oldCode}
        newValue={newCode}
        splitView={true}
        useDarkTheme={false}
      />
    </div>
  );
};
```

---

#### 6.4 Meeting Insights Panel (7.5.4) ⏳

**Component**: `Handy/src/components/meeting/MeetingInsights.tsx`

```tsx
interface Insight {
  type: 'suggestion' | 'warning' | 'question';
  message: string;
  timestamp: string;
}

export const MeetingInsights: React.FC = () => {
  const [insights, setInsights] = useState<Insight[]>([]);

  // Listen for insights from LLM

  return (
    <div className="space-y-3">
      {insights.map((insight, i) => (
        <div
          key={i}
          className={`p-3 rounded-md text-sm ${
            insight.type === 'suggestion' ? 'bg-blue-50 text-blue-800' :
            insight.type === 'warning' ? 'bg-yellow-50 text-yellow-800' :
            'bg-purple-50 text-purple-800'
          }`}
        >
          <div className="font-medium mb-1">
            {insight.type === 'suggestion' ? '💡' :
             insight.type === 'warning' ? '⚠️' : '❓'}
            {' '}{insight.type.toUpperCase()}
          </div>
          <div>{insight.message}</div>
        </div>
      ))}
    </div>
  );
};
```

---

## Project Structure Reference

### Backend (Rust)

```
Handy/src-tauri/src/
├── lib.rs                      # Main entry, invoke_handler
├── commands/
│   ├── mod.rs                  # Exports all commands
│   ├── github.rs               # GitHub commands ✅
│   ├── llm.rs                  # LLM API key commands ✅
│   ├── meeting.rs              # Meeting lifecycle
│   └── dev_server.rs           # NEW: Dev server commands
├── integrations/
│   └── github.rs               # GitHub API logic ✅
├── summarization/
│   ├── mod.rs
│   ├── agent.rs                # Heuristic summarization
│   └── llm.rs                  # Claude API summarization ✅
├── managers/
│   ├── audio.rs                # Audio capture
│   ├── transcription.rs        # Whisper
│   └── meeting.rs              # Meeting orchestration
├── project/
│   └── initializer.rs          # Project setup
├── analysis/                   # NEW: Week 3
│   └── codebase.rs             # Repo analysis
├── dev_server/                 # NEW: Week 4
│   └── manager.rs              # Dev server spawning
├── automation/
│   └── claude_trigger.rs       # AppleScript automation
├── settings.rs                 # App settings
└── templates/
    └── meeting_command.md      # Claude Code template ✅
```

### Frontend (React/TypeScript)

```
Handy/src/
├── components/
│   ├── meeting/
│   │   ├── MeetingView.tsx
│   │   ├── MeetingControls.tsx
│   │   ├── LiveTranscript.tsx
│   │   ├── MeetingUpdates.tsx
│   │   ├── MeetingModeSelector.tsx    # NEW: Week 6
│   │   ├── LivePreview.tsx            # NEW: Week 6
│   │   ├── DiffViewer.tsx             # NEW: Week 6
│   │   ├── MeetingInsights.tsx        # NEW: Week 6
│   │   └── CodeGenerationProgress.tsx # NEW: Week 5
│   └── settings/
│       ├── IntegrationsSettings.tsx   # Main settings page ✅
│       ├── GitHubOAuth.tsx            # OAuth UI ✅
│       └── GitHubToken.tsx            # Manual token (fallback) ✅
└── hooks/
    └── useSettings.ts
```

---

## Testing Checklist

### Before Shipping Each Feature

**Unit Tests** (Rust):
```bash
cd Handy/src-tauri
cargo test
```

**Integration Tests**:
- [ ] Start meeting → audio captured
- [ ] Transcription → LLM extracts features
- [ ] Features written to `.meeting-updates.jsonl`
- [ ] `/meeting` command reads JSONL correctly
- [ ] Code generated in correct mode (dev vs starter kit)
- [ ] GitHub branch created
- [ ] PR created with meeting context

**Manual Testing**:
- [ ] OAuth flow works end-to-end
- [ ] Dev server starts automatically
- [ ] Live preview shows in UI
- [ ] Build errors trigger retry
- [ ] Emergency stop button works

---

## Common Issues & Solutions

### Issue: LLM API rate limits
**Solution**: Falls back to heuristic agent automatically (already implemented)

### Issue: Build fails repeatedly
**Solution**: Max 3 retries, then flag for manual review (Week 5.4)

### Issue: AppleScript can't find Terminal
**Solution**: Creates new Terminal window if not found (Week 5.1)

### Issue: Dev server port already in use
**Solution**: Check port availability, increment if needed (Week 4.3)

---

## Success Criteria (Phase 7 Complete)

### Developer Mode
✅ Meeting starts → branch created
✅ Transcript analyzed → features extracted
✅ Code generated in `experiments/`
✅ Auto-commit → auto-push → auto-PR
✅ Build validates successfully
✅ Zero core file modifications

### Starter Kit Mode
✅ Meeting starts → project type detected
✅ Vercel + Supabase app scaffolded
✅ Dependencies installed
✅ Dev server running
✅ Live preview visible
✅ Features generate from conversation

### Automation
✅ Zero manual steps (except mode selection)
✅ Code appears within 90 seconds of discussion
✅ Error recovery automatic
✅ Emergency stop available

### Quality
✅ Generated code passes linting
✅ TypeScript compiles
✅ Build succeeds (>90% success rate)
✅ Production-ready code quality

---

## Resources & Documentation

**PRD**: `/docs/prd/07-PHASE7.md` (full spec)
**Week 1-2 Summary**: `/docs/WEEK2_SUMMARY.md` (what's done)
**Architecture**: `/docs/prd/TECHNICAL_ARCHITECTURE.md`
**API Specs**: `/docs/prd/API_SPECIFICATIONS.md`

**GitHub Repo**: TBD (user will provide)
**Client ID**: `Ov23liUutHAz1Qx5xvSy` (OAuth app)

---

## Your Mission

**Build Weeks 3-6 to make MeetingCoder production-ready.**

Focus on:
1. **Developer productivity** - Save every possible minute
2. **Code quality** - Generated code should be PR-ready
3. **Safety** - Never break existing code
4. **UX polish** - Make it feel magical

**You're building the future of software development.** Make it count. 🚀

---

**Questions?** Check the PRD, existing code, or ask the user.
**Stuck?** Look at similar patterns in existing codebase (GitHub integration is a good reference).
**Ready?** Start with Week 3, Task 1: Codebase Context Ingestion. Let's ship it! 💪
