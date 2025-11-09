# Phase 7 Week 3 Handoff - Developer Mode Features

**Date**: November 7, 2025
**Status**: 7/23 tasks complete (30%)
**Current Focus**: Week 3 - Developer Mode Features (2/4 tasks remaining)

---

## 📋 Executive Summary

You're continuing Phase 7 implementation of MeetingCoder's intelligent real-time code generation system. **Weeks 1-2 are 100% complete** (foundations + GitHub integration). **Week 3 is 50% complete** (codebase analysis + file isolation done).

**Your mission**: Complete the remaining 2 Week 3 tasks to finish Developer Mode features:
1. **7.2.3**: Feature Branch Workflow Integration - Auto-create branches/PRs during meetings
2. **7.2.4**: Code-Aware Transcript Analysis - Correlate transcript with codebase files

---

## ✅ What's Already Complete (Context)

### Week 1-2: Foundations + GitHub Integration (5 tasks)
- ✅ Enhanced `/meeting` command template with dual-mode support
- ✅ LLM-based summarization (Claude API integration)
- ✅ GitHub OAuth Device Flow UI
- ✅ GitHub branch & PR management (backend already existed)
- ✅ GitHub repo picker (already existed)

### Week 3 (So Far): Developer Mode Features (2 tasks)
- ✅ **7.2.1**: Codebase Context Ingestion
  - File: `Handy/src-tauri/src/codebase/analyzer.rs` (554 lines)
  - Detects framework (Next.js, React, Django, etc.), languages, dependencies
  - Generates manifest saved to `.claude/.meeting-state.json`
  - Auto-runs on meeting start in background

- ✅ **7.2.2**: Intelligent File Isolation
  - File: `Handy/src-tauri/src/codebase/isolation.rs` (380 lines)
  - Generates `.claudeignore` to protect core files
  - Creates `experiments/{meeting_id}/` safe workspace
  - Enforces path safety in `/meeting` template

---

## 🎯 Your Tasks (Week 3 - Remaining)

### Task 7.2.3: Feature Branch Workflow Integration

**Goal**: Automatically create Git branches and PRs as part of meeting lifecycle.

**Current State**:
- Backend functions already exist in `integrations/github.rs`:
  - `create_branch(owner, repo, branch_name, base_branch, token)` - Creates branch
  - `push_to_remote(project_path, remote, branch, token)` - Pushes commits
  - `create_pull_request(owner, repo, title, body, head, base, token)` - Creates PR
  - `update_pull_request(owner, repo, pr_number, title, body, token)` - Updates PR
  - `get_prs_for_branch(owner, repo, branch, token)` - Checks existing PRs
- State tracking exists: `.claude/.github-state.json` with `last_branch`, `last_pr_url`, `last_pr_number`

**What You Need to Do**:

#### Step 1: Hook Branch Creation into Meeting Start

**File**: `Handy/src-tauri/src/managers/meeting.rs`

**Location**: In `start_meeting()` function, after the codebase analysis code (~line 257), add branch creation logic.

**Code to Add**:
```rust
// Create feature branch if GitHub integration enabled
if let Some(ref path) = {
    let meetings = self.active_meetings.lock().await;
    meetings.get(&meeting_id).and_then(|m| m.project_path.clone())
} {
    let settings = settings::get_settings(&self.app_handle);
    if settings.github_enabled
        && settings.github_repo_owner.is_some()
        && settings.github_repo_name.is_some()
    {
        let owner = settings.github_repo_owner.unwrap();
        let repo = settings.github_repo_name.unwrap();
        let branch_name = format!("discovery/{}", meeting_id);
        let project_path_clone = path.clone();
        let meeting_id_clone = meeting_id.clone();

        tokio::spawn(async move {
            match crate::integrations::github::get_github_token() {
                Ok(token) => {
                    // Create and checkout branch
                    match crate::integrations::github::create_branch(
                        &owner,
                        &repo,
                        &branch_name,
                        "main", // or get from settings.github_default_branch
                        &token,
                    ).await {
                        Ok(_) => {
                            log::info!("Created branch {} for meeting {}", branch_name, meeting_id_clone);

                            // Update GitHub state
                            let mut state = crate::integrations::github::read_github_state(&project_path_clone);
                            state.last_branch = Some(branch_name.clone());
                            let _ = crate::integrations::github::write_github_state(&project_path_clone, &state);
                        }
                        Err(e) => {
                            log::warn!("Failed to create branch {}: {}", branch_name, e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("No GitHub token available for branch creation: {}", e);
                }
            }
        });
    }
}
```

**Same for `start_offline_meeting()`** - add identical logic after line 366.

#### Step 2: Auto-Commit and Push After Code Generation

**Challenge**: The `/meeting` command runs in Claude Code (external terminal), not in Rust. You need to trigger commit/push after Claude Code finishes.

**Approach**: Use the automation trigger system.

**File**: `Handy/src-tauri/src/commands/automation.rs`

**Find**: The `trigger_meeting_command_now()` function (around line 50-100)

**After**: The part where it triggers the `/meeting` command, add post-execution logic to commit and push.

**Conceptual Flow**:
```rust
// After triggering /meeting command and waiting for completion...
// (You'll need to add a way to detect when Claude Code finishes)

// Then auto-commit
let commit_message = format!("Update meeting: {} ({})", meeting_name, update_count);
let result = Command::new("git")
    .arg("-C")
    .arg(&project_path)
    .arg("add")
    .arg("experiments/")
    .arg(".meeting-updates.jsonl")
    .arg(".claude/")
    .output();

// Then commit
Command::new("git")
    .arg("-C")
    .arg(&project_path)
    .arg("commit")
    .arg("-m")
    .arg(&commit_message)
    .output();

// Then push
if let Ok(token) = github::get_github_token() {
    let branch_name = format!("discovery/{}", meeting_id);
    github::push_to_remote(&project_path, "origin", &branch_name, &token).await?;
}
```

**Better Approach**: Hook into the meeting update cycle in `managers/meeting.rs` around the summarization loop (line 400-500 area where it writes to `.meeting-updates.jsonl`).

Add a function:
```rust
async fn auto_commit_and_push(
    project_path: &str,
    meeting_id: &str,
    update_count: usize,
) -> Result<()> {
    let settings = settings::get_settings(&app_handle);
    if !settings.github_enabled {
        return Ok(());
    }

    // Git add
    tokio::process::Command::new("git")
        .arg("-C")
        .arg(project_path)
        .arg("add")
        .arg(".")
        .output()
        .await?;

    // Git commit
    let commit_msg = format!("Meeting update {} - auto-generated", update_count);
    tokio::process::Command::new("git")
        .arg("-C")
        .arg(project_path)
        .arg("commit")
        .arg("-m")
        .arg(&commit_msg)
        .output()
        .await?;

    // Git push
    if let Ok(token) = crate::integrations::github::get_github_token() {
        let branch = format!("discovery/{}", meeting_id);
        crate::integrations::github::push_to_remote(
            project_path,
            "origin",
            &branch,
            &token,
        ).await?;
    }

    Ok(())
}
```

Call this after writing to `.meeting-updates.jsonl`.

#### Step 3: Auto-Create PR After First Commit

**File**: Same location as above, in the meeting update loop.

**Logic**:
```rust
// After first push, create PR
let state = github::read_github_state(project_path);
if state.last_pr_number.is_none() {
    // First commit - create PR
    let owner = settings.github_repo_owner.unwrap();
    let repo = settings.github_repo_name.unwrap();
    let branch = format!("discovery/{}", meeting_id);

    let pr_title = format!("Meeting: {}", meeting_name);
    let pr_body = format!(
        "## Meeting: {}\n\n\
        This PR contains code generated during the meeting.\n\n\
        **Updates processed**: {}\n\n\
        **Branch**: `{}`\n\n\
        ---\n\
        🤖 Generated with MeetingCoder",
        meeting_name,
        update_count,
        branch
    );

    match github::create_pull_request(
        &owner,
        &repo,
        &pr_title,
        &pr_body,
        &branch,
        "main",
        &token,
    ).await {
        Ok(pr) => {
            let mut state = github::read_github_state(project_path);
            state.last_pr_url = Some(pr.html_url);
            state.last_pr_number = Some(pr.number);
            github::write_github_state(project_path, &state)?;

            log::info!("Created PR #{}: {}", pr.number, pr.html_url);
        }
        Err(e) => {
            log::warn!("Failed to create PR: {}", e);
        }
    }
}
```

#### Step 4: Update PR Body on Subsequent Updates

**Logic** (in same loop, for non-first updates):
```rust
// If PR exists, update it
if let Some(pr_number) = state.last_pr_number {
    let updated_body = format!(
        "## Meeting: {}\n\n\
        **Updates processed**: {}\n\n\
        ### Features Implemented:\n{}\n\n\
        ---\n\
        🤖 Generated with MeetingCoder",
        meeting_name,
        update_count,
        feature_list // Build this from .meeting-updates.jsonl
    );

    github::update_pull_request(
        &owner,
        &repo,
        pr_number,
        None, // Don't change title
        Some(&updated_body),
        &token,
    ).await?;
}
```

**Acceptance Criteria**:
- ✅ Branch created immediately on meeting start (visible in `git branch`)
- ✅ Commits happen automatically after code generation
- ✅ PR appears on GitHub after first commit
- ✅ PR body updates with each new feature

---

### Task 7.2.4: Code-Aware Transcript Analysis

**Goal**: Scan transcripts for file mentions and cross-reference with codebase.

**Current State**:
- Transcript segments saved to `.transcript.jsonl` with `text` field
- Codebase manifest in `.claude/.meeting-state.json` has `entry_points`, `key_directories`
- Update records in `.meeting-updates.jsonl` need `target_files` field

**What You Need to Do**:

#### Step 1: Enhance LLM Extraction Prompt to Identify File Mentions

**File**: `Handy/src-tauri/src/summarization/llm.rs`

**Find**: The `summarize_with_llm()` function (around line 150-200)

**Current Prompt**: The function builds a prompt for Claude API.

**Enhance the Prompt**:
Find where the user prompt is constructed and add:

```rust
// Add to the user_prompt string
let file_mention_instruction = "\n\n\
**File and Path Detection**:\n\
- Scan the transcript for any mentions of files, components, or directories\n\
- Examples: \"in the API routes\", \"update HomePage.tsx\", \"the authentication module\", \"in src/components/\"\n\
- Extract these as specific file paths when possible\n\
- Return them in a new field: `mentioned_files`\n\
";

user_prompt.push_str(file_mention_instruction);
```

**Update the JSON Schema** in the extraction instructions:
```json
{
  "new_features_structured": [...],
  "technical_decisions": [...],
  "questions": [...],
  "project_type": "...",
  "mentioned_files": [
    {
      "mention": "API routes",
      "inferred_path": "src/app/api/",
      "confidence": "medium"
    },
    {
      "mention": "HomePage component",
      "inferred_path": "src/components/HomePage.tsx",
      "confidence": "high"
    }
  ]
}
```

**Update the `ExtractionResult` struct**:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub new_features: Vec<ExtractedFeature>,
    pub technical_decisions: Vec<String>,
    pub questions: Vec<String>,
    pub project_type: Option<String>,
    pub mentioned_files: Option<Vec<MentionedFile>>, // NEW
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MentionedFile {
    pub mention: String,
    pub inferred_path: String,
    pub confidence: String,
}
```

#### Step 2: Cross-Reference with Codebase Manifest

**File**: `Handy/src-tauri/src/managers/meeting.rs`

**Location**: In the summarization loop where you process `ExtractionResult`.

**Add Function**:
```rust
fn resolve_file_mentions(
    mentioned_files: &[MentionedFile],
    manifest: &CodebaseManifest,
) -> Vec<String> {
    let mut resolved_files = Vec::new();

    for mention in mentioned_files {
        // Try exact match first
        let exact_match = manifest.entry_points.iter()
            .chain(manifest.key_directories.values())
            .find(|path| path.to_string_lossy().contains(&mention.inferred_path));

        if let Some(matched) = exact_match {
            resolved_files.push(matched.to_string_lossy().to_string());
            log::info!("Matched '{}' to {}", mention.mention, matched.display());
        } else {
            // Try fuzzy match on file name
            let file_name = mention.inferred_path.split('/').last().unwrap_or("");
            if !file_name.is_empty() {
                // Search all files in key directories
                // This is simplified - you may want to walk the actual filesystem
                log::warn!("File mentioned but not found: '{}' ({})", mention.mention, mention.inferred_path);
            }
        }
    }

    resolved_files
}
```

**Call it**:
```rust
// After getting ExtractionResult from LLM
if let Some(mentioned_files) = &extraction_result.mentioned_files {
    // Load manifest
    let state_path = project_path.join(".claude/.meeting-state.json");
    if let Ok(state_content) = fs::read_to_string(&state_path) {
        if let Ok(state) = serde_json::from_str::<serde_json::Value>(&state_content) {
            if let Some(manifest) = state.get("codebase_manifest") {
                if let Ok(manifest) = serde_json::from_value::<CodebaseManifest>(manifest.clone()) {
                    let target_files = resolve_file_mentions(mentioned_files, &manifest);
                    // Store in update record...
                }
            }
        }
    }
}
```

#### Step 3: Add `target_files` to Update Records

**File**: Same as above, where you write to `.meeting-updates.jsonl`

**Find**: The code that creates the update JSON object (around line 450-500)

**Add Field**:
```rust
// Build the update record
let mut update = serde_json::json!({
    "update_id": update_id,
    "meeting_id": meeting_id,
    "meeting_name": meeting_name,
    "timestamp": timestamp,
    "segment_range": [start_idx, end_idx],
    "new_features": summary.new_features,
    "technical_decisions": summary.technical_decisions,
    "questions": summary.questions,
    // ... existing fields
});

// Add target_files if available
if !target_files.is_empty() {
    update["target_files"] = serde_json::json!(target_files);
}
```

#### Step 4: Update `/meeting` Template to Use Target Files

**File**: `Handy/src-tauri/templates/meeting_command.md`

**Find**: Step 5 where it says "Process subsequent updates" (around line 98-105)

**Add**:
```markdown
   - **Target files** (if specified): Use `target_files` field to identify which files to modify
     - If `target_files` is present and non-empty, focus changes on those specific files
     - Example: `"target_files": ["src/components/HomePage.tsx", "src/app/api/auth/route.ts"]`
     - Provide these to Claude Code as context: "The user mentioned these files: [list]"
     - If a file doesn't exist, create it in the appropriate location
```

**Acceptance Criteria**:
- ✅ `.meeting-updates.jsonl` includes `target_files: []` array
- ✅ Files mentioned in transcript are resolved to actual paths
- ✅ Warnings logged when mentioned files don't exist
- ✅ `/meeting` command receives target files as context

---

## 🔧 Technical Details

### File Locations
- **Backend**: `Handy/src-tauri/src/`
  - `managers/meeting.rs` - Meeting lifecycle (where you'll add branch/PR logic)
  - `integrations/github.rs` - GitHub API functions (already complete)
  - `summarization/llm.rs` - LLM extraction (enhance for file mentions)
  - `codebase/analyzer.rs` - Codebase manifest (use for file resolution)
  - `commands/automation.rs` - Automation triggers (commit/push)
- **Template**: `Handy/src-tauri/templates/meeting_command.md`
- **State Files**:
  - `.claude/.meeting-state.json` - Codebase manifest
  - `.claude/.github-state.json` - GitHub state (branch, PR)
  - `.meeting-updates.jsonl` - Update records

### Key Functions to Use
```rust
// GitHub operations (in integrations/github.rs)
github::create_branch(owner, repo, branch_name, base_branch, token)
github::push_to_remote(project_path, remote, branch, token)
github::create_pull_request(owner, repo, title, body, head, base, token)
github::update_pull_request(owner, repo, pr_number, title, body, token)
github::get_prs_for_branch(owner, repo, branch, token)
github::read_github_state(project_path)
github::write_github_state(project_path, &state)

// Settings
settings::get_settings(app_handle)

// Codebase analysis (use existing manifest)
codebase::CodebaseManifest // struct in .meeting-state.json
```

### Data Structures

**GitHub State** (`.claude/.github-state.json`):
```json
{
  "repo_owner": "user",
  "repo_name": "project",
  "default_branch": "main",
  "branch_pattern": "meeting/{meeting_id}",
  "last_branch": "discovery/abc-123",
  "last_pr_url": "https://github.com/user/project/pull/42",
  "last_pr_number": 42,
  "last_push_time": "2025-11-07T16:30:00Z"
}
```

**Update Record** (`.meeting-updates.jsonl`):
```json
{
  "update_id": "u1",
  "meeting_id": "abc-123",
  "meeting_name": "Feature Planning",
  "timestamp": "2025-11-07T16:30:00Z",
  "segment_range": [0, 10],
  "new_features": ["User authentication"],
  "technical_decisions": ["Use Supabase"],
  "questions": [],
  "target_files": ["src/app/login/page.tsx", "src/lib/supabase.ts"]
}
```

---

## 🧪 Testing Checklist

### Task 7.2.3 Testing
- [ ] Start meeting with GitHub repo attached
- [ ] Check `git branch` - should see `discovery/{meeting_id}`
- [ ] Generate some code via `/meeting` command
- [ ] Check `git log` - should see auto-commit
- [ ] Check GitHub - should see branch pushed
- [ ] Check GitHub - should see draft PR created
- [ ] Generate more code
- [ ] Check GitHub - PR body should update with new features
- [ ] Verify `.claude/.github-state.json` has correct `last_branch`, `last_pr_number`

### Task 7.2.4 Testing
- [ ] Start meeting, say "let's update the HomePage component"
- [ ] Check logs for file mention detection
- [ ] Check `.meeting-updates.jsonl` - should have `target_files: ["src/components/HomePage.tsx"]`
- [ ] Say "in the API routes, add authentication"
- [ ] Check logs for "Matched 'API routes' to src/app/api/"
- [ ] Check update record has `target_files` with API path
- [ ] Run `/meeting` command - verify context includes target files
- [ ] Mention non-existent file - should log warning but not crash

---

## 🚨 Common Issues

### Task 7.2.3
- **Issue**: Branch creation fails with "already exists"
  - **Fix**: Check existing branches first with `get_prs_for_branch()`, skip if exists

- **Issue**: Git commits fail with "nothing to commit"
  - **Fix**: Check `git status` output before committing, skip if no changes

- **Issue**: PR creation fails with 422 (validation error)
  - **Fix**: Ensure branch is pushed before creating PR, check PR doesn't already exist

### Task 7.2.4
- **Issue**: LLM doesn't extract file mentions
  - **Fix**: Make the prompt more explicit with examples, use few-shot prompting

- **Issue**: File resolution fails for valid files
  - **Fix**: Expand search to include actual filesystem walk, not just manifest

- **Issue**: Too many false positives ("the file" → matches every file)
  - **Fix**: Add confidence thresholds, require file extensions or clear indicators

---

## 📊 Success Metrics

When you're done:
- ✅ Meeting start → branch created within 2 seconds
- ✅ Code generation → commit + push within 5 seconds
- ✅ First commit → PR visible on GitHub within 10 seconds
- ✅ File mentions → 80%+ accurate path resolution
- ✅ Target files → passed to Claude Code in 100% of cases when mentioned
- ✅ Zero manual git commands needed for workflow

---

## 🎯 Next Steps After You're Done

Once Tasks 7.2.3 and 7.2.4 are complete, **Week 3 is done** (4/4 tasks).

**Week 4** will focus on Starter Kit Mode:
- 7.3.1: Starter Pack System (Vercel + Supabase + Next.js template)
- 7.3.2: Project Type Detection from transcript
- 7.3.3: Live Dev Server Integration (spawn `npm run dev`, capture URL)
- 7.3.4: Real-Time Scaffolding (generate app structure from scratch)

---

## 💡 Tips

1. **Test incrementally**: Don't implement both tasks at once. Do 7.2.3 first, test thoroughly, then 7.2.4.

2. **Use existing code**: All GitHub functions already exist - you're just wiring them up to the meeting lifecycle.

3. **Check logs**: Add `log::info!()` statements liberally. The meeting loop is complex.

4. **Read HANDOFF_PHASE7.md**: The previous handoff has more context on the overall architecture.

5. **Don't break existing flows**: The meeting manager is critical. Test that normal meetings still work.

6. **Async/await**: Remember most GitHub operations are async. Use `tokio::spawn()` for background tasks.

7. **Error handling**: Don't crash if GitHub operations fail. Log warnings and continue.

---

## 📞 Questions?

If you get stuck:
- Check `docs/HANDOFF_PHASE7.md` for overall Phase 7 context
- Check `docs/prd/07-PHASE7.md` for requirements
- Look at existing GitHub integration code in `integrations/github.rs` for examples
- The meeting manager loop is ~600 lines - take time to understand the flow

Good luck! You're building the core automation that makes MeetingCoder truly powerful. 🚀
