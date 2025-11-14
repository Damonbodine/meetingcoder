use crate::integrations::github;
use crate::settings;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::AppHandle;

// Type definitions for meeting updates
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeetingUpdate {
    #[serde(default)]
    pub update_id: u32,
    #[serde(default)]
    pub new_features_structured: Vec<Feature>,
    #[serde(default)]
    pub technical_decisions: Vec<String>,
    #[serde(default)]
    pub questions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Feature {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub priority: String,
}

/// Automatically create a feature branch for a meeting on startup
/// This is called when a meeting starts in Developer Mode with GitHub enabled
pub async fn auto_create_branch(
    app: &AppHandle,
    project_path: &str,
    meeting_id: &str,
    meeting_name: &str,
) -> Result<String> {
    let settings = settings::get_settings(app);

    // Only proceed if GitHub is enabled
    if !settings.advanced_features_enabled {
        return Err(anyhow!("Advanced Automations disabled"));
    }
    if settings.offline_mode_enabled {
        return Err(anyhow!("Offline mode enabled"));
    }
    if !settings.github_enabled {
        return Err(anyhow!("GitHub integration not enabled"));
    }

    // Validate settings
    let owner = settings
        .github_repo_owner
        .as_ref()
        .ok_or_else(|| anyhow!("GitHub repository owner not set"))?;
    let repo = settings
        .github_repo_name
        .as_ref()
        .ok_or_else(|| anyhow!("GitHub repository name not set"))?;

    // Get token
    let _token = github::get_github_token().map_err(|e| anyhow!("No GitHub token: {}", e))?;

    // Initialize or open repo
    let repo_obj = github::init_git_repo(project_path)
        .map_err(|e| anyhow!("Failed to init git repo: {}", e))?;

    // Generate branch name
    let branch_name =
        github::generate_branch_name(&settings.github_branch_pattern, meeting_id, meeting_name);

    // Check if we're on the default branch
    let current_branch = github::get_current_branch(&repo_obj).unwrap_or_default();

    // Only create branch if we're currently on the default branch
    if current_branch == settings.github_default_branch {
        // Create and checkout the new branch
        github::create_branch(&repo_obj, &branch_name)
            .map_err(|e| anyhow!("Failed to create branch: {}", e))?;

        log::info!(
            "GITHUB_WORKFLOW auto-created branch '{}' for meeting '{}' in repo {}/{}",
            branch_name,
            meeting_name,
            owner,
            repo
        );
    } else if current_branch == branch_name {
        log::info!(
            "GITHUB_WORKFLOW already on correct branch '{}' for meeting",
            branch_name
        );
    } else {
        log::warn!(
            "GITHUB_WORKFLOW cannot create branch '{}' - currently on branch '{}' (not default)",
            branch_name,
            current_branch
        );
    }

    // Update GitHub state
    let mut github_state = github::read_github_state(project_path);
    github_state.repo_owner = Some(owner.clone());
    github_state.repo_name = Some(repo.clone());
    github_state.default_branch = settings.github_default_branch.clone();
    github_state.branch_pattern = settings.github_branch_pattern.clone();
    github_state.last_branch = Some(branch_name.clone());
    github::write_github_state(project_path, &github_state)
        .map_err(|e| anyhow!("Failed to write GitHub state: {}", e))?;

    Ok(branch_name)
}

/// Automatically commit and push meeting changes after an update
pub async fn auto_commit_and_push(
    app: &AppHandle,
    project_path: &str,
    meeting_id: &str,
    meeting_name: &str,
    update_id: u32,
) -> Result<String> {
    let settings = settings::get_settings(app);

    // Check if auto-commit-push is enabled
    if !settings.advanced_features_enabled {
        return Err(anyhow!("Advanced Automations disabled"));
    }
    if settings.offline_mode_enabled {
        return Err(anyhow!("Offline mode enabled"));
    }
    if !settings.github_enabled || !settings.github_auto_commit_push {
        return Err(anyhow!("GitHub auto-commit-push not enabled"));
    }

    // Validate settings
    let owner = settings
        .github_repo_owner
        .as_ref()
        .ok_or_else(|| anyhow!("GitHub repository owner not set"))?;
    let repo = settings
        .github_repo_name
        .as_ref()
        .ok_or_else(|| anyhow!("GitHub repository name not set"))?;

    // Get token
    let token = github::get_github_token().map_err(|e| anyhow!("No GitHub token: {}", e))?;

    // Initialize repo
    let repo_obj = github::init_git_repo(project_path)
        .map_err(|e| anyhow!("Failed to init git repo: {}", e))?;

    // Generate branch name
    let branch_name =
        github::generate_branch_name(&settings.github_branch_pattern, meeting_id, meeting_name);

    // Check current branch
    let current_branch = github::get_current_branch(&repo_obj).unwrap_or_default();

    // If we're still on default branch, create the meeting branch
    if current_branch == settings.github_default_branch {
        github::create_branch(&repo_obj, &branch_name)
            .map_err(|e| anyhow!("Failed to create branch: {}", e))?;
    } else if current_branch != branch_name {
        return Err(anyhow!(
            "Not on expected branch (current: {}, expected: {})",
            current_branch,
            branch_name
        ));
    }

    // Commit meeting files
    let commit_message = format!(
        "Update meeting: {} (update #{})\n\nAutomatically generated from Handy meeting session.",
        meeting_name, update_id
    );

    github::commit_meeting_files(
        &repo_obj,
        &commit_message,
        "Handy",
        "noreply@handy.computer",
    )
    .map_err(|e| anyhow!("Failed to commit: {}", e))?;

    // Push to remote
    github::push_to_remote(project_path, &branch_name, &token, owner, repo)
        .map_err(|e| anyhow!("Failed to push: {}", e))?;

    // Update GitHub state
    let mut github_state = github::read_github_state(project_path);
    github_state.last_branch = Some(branch_name.clone());
    github_state.last_push_time = Some(chrono::Utc::now().to_rfc3339());
    github::write_github_state(project_path, &github_state)
        .map_err(|e| anyhow!("Failed to write GitHub state: {}", e))?;

    log::info!(
        "GITHUB_WORKFLOW auto-committed and pushed update #{} to branch '{}'",
        update_id,
        branch_name
    );

    Ok(branch_name)
}

/// Automatically create or update a pull request after pushing changes
pub async fn auto_create_or_update_pr(
    app: &AppHandle,
    project_path: &str,
    meeting_id: &str,
    meeting_name: &str,
    is_first_update: bool,
) -> Result<(u32, String)> {
    let settings = settings::get_settings(app);

    // Check if auto-PR is enabled
    if !settings.advanced_features_enabled {
        return Err(anyhow!("Advanced Automations disabled"));
    }
    if settings.offline_mode_enabled {
        return Err(anyhow!("Offline mode enabled"));
    }
    if !settings.github_enabled {
        return Err(anyhow!("GitHub integration not enabled"));
    }

    // For first update, check auto_create; for subsequent, check auto_update
    if is_first_update && !settings.github_auto_create_pr {
        return Err(anyhow!("GitHub auto-create-PR not enabled"));
    }
    if !is_first_update && !settings.github_auto_update_pr {
        return Err(anyhow!("GitHub auto-update-PR not enabled"));
    }

    // Validate settings
    let owner = settings
        .github_repo_owner
        .as_ref()
        .ok_or_else(|| anyhow!("GitHub repository owner not set"))?;
    let repo = settings
        .github_repo_name
        .as_ref()
        .ok_or_else(|| anyhow!("GitHub repository name not set"))?;

    // Get token
    let token = github::get_github_token().map_err(|e| anyhow!("No GitHub token: {}", e))?;

    // Read GitHub state
    let github_state = github::read_github_state(project_path);

    // Generate branch name
    let branch_name =
        github::generate_branch_name(&settings.github_branch_pattern, meeting_id, meeting_name);

    // Read meeting updates to build PR body
    let updates = read_meeting_updates(project_path)?;

    // Generate PR title and body
    let pr_title = format!("Meeting: {}", meeting_name);
    let pr_body = generate_pr_body(meeting_id, meeting_name, &updates);

    // Check if PR already exists
    let existing_prs = github::get_prs_for_branch(&token, owner, repo, &branch_name)
        .await
        .map_err(|e| anyhow!("Failed to check existing PRs: {}", e))?;

    let pr = if let Some(existing_pr) = existing_prs.first() {
        // Update existing PR
        let updated_pr = github::update_pull_request(
            &token,
            owner,
            repo,
            existing_pr.number,
            Some(&pr_title),
            Some(&pr_body),
        )
        .await
        .map_err(|e| anyhow!("Failed to update PR: {}", e))?;

        log::info!(
            "GITHUB_WORKFLOW auto-updated PR #{}: {}",
            updated_pr.number,
            updated_pr.html_url
        );

        updated_pr
    } else {
        // Create new PR
        let new_pr = github::create_pull_request(
            &token,
            owner,
            repo,
            &pr_title,
            &pr_body,
            &branch_name,
            &settings.github_default_branch,
        )
        .await
        .map_err(|e| anyhow!("Failed to create PR: {}", e))?;

        log::info!(
            "GITHUB_WORKFLOW auto-created PR #{}: {}",
            new_pr.number,
            new_pr.html_url
        );

        new_pr
    };

    // Update GitHub state
    let mut github_state = github_state;
    github_state.last_pr_url = Some(pr.html_url.clone());
    github_state.last_pr_number = Some(pr.number);
    github::write_github_state(project_path, &github_state)
        .map_err(|e| anyhow!("Failed to write GitHub state: {}", e))?;

    Ok((pr.number, pr.html_url))
}

/// Read meeting updates from .meeting-updates.jsonl
fn read_meeting_updates(project_path: &str) -> Result<Vec<MeetingUpdate>> {
    let updates_path = Path::new(project_path).join(".meeting-updates.jsonl");

    if !updates_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(updates_path)
        .map_err(|e| anyhow!("Failed to read updates file: {}", e))?;

    let mut updates = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<MeetingUpdate>(line) {
            Ok(update) => updates.push(update),
            Err(e) => {
                log::warn!("Failed to parse update line: {}", e);
                continue;
            }
        }
    }

    Ok(updates)
}

/// Generate PR body from meeting updates
fn generate_pr_body(meeting_id: &str, meeting_name: &str, updates: &[MeetingUpdate]) -> String {
    use std::fmt::Write;

    let mut body = String::new();
    let _ = writeln!(body, "# Meeting Summary\n");
    let _ = writeln!(body, "**Meeting ID:** {}", meeting_id);
    let _ = writeln!(body, "**Meeting Name:** {}\n", meeting_name);

    // Aggregate all features
    let mut all_features = Vec::new();
    let mut all_decisions = Vec::new();
    let mut all_questions = Vec::new();

    for update in updates {
        all_features.extend(update.new_features_structured.clone());
        all_decisions.extend(update.technical_decisions.clone());
        all_questions.extend(update.questions.clone());
    }

    if !all_features.is_empty() {
        let _ = writeln!(body, "## Features\n");
        for (i, feature) in all_features.iter().enumerate() {
            let priority_emoji = match feature.priority.as_str() {
                "high" => "🔴",
                "medium" => "🟡",
                "low" => "🟢",
                _ => "⚪",
            };
            let _ = writeln!(
                body,
                "{}. {} **{}**\n   {}",
                i + 1,
                priority_emoji,
                feature.title,
                feature.description
            );
        }
        let _ = writeln!(body);
    }

    if !all_decisions.is_empty() {
        let _ = writeln!(body, "## Technical Decisions\n");
        for (i, decision) in all_decisions.iter().enumerate() {
            let _ = writeln!(body, "{}. {}", i + 1, decision);
        }
        let _ = writeln!(body);
    }

    if !all_questions.is_empty() {
        let _ = writeln!(body, "## Open Questions\n");
        for (i, question) in all_questions.iter().enumerate() {
            let _ = writeln!(body, "{}. {}", i + 1, question);
        }
        let _ = writeln!(body);
    }

    let _ = writeln!(
        body,
        "---\n*Automatically generated with [MeetingCoder](https://github.com/Damonbodine/meetingcoder) (based on [Handy](https://github.com/cjpais/handy))*"
    );

    body
}
