use crate::integrations::github::{self, DeviceCodeResponse, GitHubState, RepoInfo};
use crate::managers::meeting::MeetingManager;
use crate::settings;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[derive(serde::Serialize)]
pub struct GitHubConnectionTest {
    pub success: bool,
    pub username: Option<String>,
    pub error: Option<String>,
}

#[derive(serde::Serialize)]
pub struct GitHubRepoStatus {
    pub repo_owner: Option<String>,
    pub repo_name: Option<String>,
    pub default_branch: String,
    pub branch_pattern: String,
    pub has_token: bool,
    pub current_branch: Option<String>,
    pub last_pr_url: Option<String>,
    pub last_pr_number: Option<u32>,
    pub last_push_time: Option<String>,
}

#[derive(serde::Serialize)]
pub struct PushResult {
    pub success: bool,
    pub branch: String,
    pub commit_message: String,
    pub error: Option<String>,
}

#[derive(serde::Serialize)]
pub struct PRResult {
    pub success: bool,
    pub pr_number: Option<u32>,
    pub pr_url: Option<String>,
    pub error: Option<String>,
}

fn ensure_github_allowed(app: &AppHandle) -> Result<(), String> {
    let settings = settings::get_settings(app);
    if !settings.advanced_features_enabled {
        return Err("GitHub integrations require Advanced Automations to be enabled.".to_string());
    }
    if settings.offline_mode_enabled {
        return Err("Offline mode is enabled. Disable it to use GitHub features.".to_string());
    }
    Ok(())
}

/// Store GitHub token securely
#[tauri::command]
pub async fn set_github_token(app: AppHandle, token: String) -> Result<bool, String> {
    ensure_github_allowed(&app)?;
    github::store_github_token(&token).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Remove GitHub token
#[tauri::command]
pub async fn remove_github_token() -> Result<bool, String> {
    github::delete_github_token().map_err(|e| e.to_string())?;
    Ok(true)
}

/// Test GitHub connection with current token
#[tauri::command]
pub async fn test_github_connection(app: AppHandle) -> Result<GitHubConnectionTest, String> {
    ensure_github_allowed(&app)?;
    match github::get_github_token() {
        Ok(token) => match github::test_github_connection(&token).await {
            Ok(username) => Ok(GitHubConnectionTest {
                success: true,
                username: Some(username),
                error: None,
            }),
            Err(e) => Ok(GitHubConnectionTest {
                success: false,
                username: None,
                error: Some(e.to_string()),
            }),
        },
        Err(e) => Ok(GitHubConnectionTest {
            success: false,
            username: None,
            error: Some(format!("No token found: {}", e)),
        }),
    }
}

/// List user's GitHub repositories
#[tauri::command]
pub async fn list_github_repos(app: AppHandle) -> Result<Vec<RepoInfo>, String> {
    ensure_github_allowed(&app)?;
    let token = github::get_github_token().map_err(|e| format!("No GitHub token: {}", e))?;
    github::list_user_repos(&token)
        .await
        .map_err(|e| e.to_string())
}

/// Update GitHub repository settings
#[tauri::command]
pub async fn set_github_repo(
    app: AppHandle,
    owner: String,
    name: String,
    default_branch: Option<String>,
    branch_pattern: Option<String>,
) -> Result<bool, String> {
    let mut settings = settings::get_settings(&app);

    settings.github_repo_owner = Some(owner);
    settings.github_repo_name = Some(name);
    if let Some(branch) = default_branch {
        settings.github_default_branch = branch;
    }
    if let Some(pattern) = branch_pattern {
        settings.github_branch_pattern = pattern;
    }

    settings::write_settings(&app, settings);
    log::info!("GITHUB repo settings updated");
    Ok(true)
}

/// Enable or disable GitHub integration
#[tauri::command]
pub async fn set_github_enabled(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let mut settings = settings::get_settings(&app);
    settings.github_enabled = enabled;
    settings::write_settings(&app, settings);
    log::info!("GITHUB integration enabled: {}", enabled);
    Ok(true)
}

/// Get GitHub repository status
#[tauri::command]
pub async fn get_github_repo_status(
    app: AppHandle,
    meeting_id: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<GitHubRepoStatus, String> {
    let settings = settings::get_settings(&app);

    let has_token = github::get_github_token().is_ok();

    // Get meeting project path
    let meeting = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .map_err(|e| format!("{}", e))?;

    let current_branch = if let Some(path) = &meeting.project_path {
        match github::init_git_repo(path) {
            Ok(repo) => github::get_current_branch(&repo).ok(),
            Err(_) => None,
        }
    } else {
        None
    };

    // Read GitHub state for this project
    let github_state = if let Some(path) = &meeting.project_path {
        github::read_github_state(path)
    } else {
        GitHubState::new()
    };

    Ok(GitHubRepoStatus {
        repo_owner: settings.github_repo_owner.clone(),
        repo_name: settings.github_repo_name.clone(),
        default_branch: settings.github_default_branch.clone(),
        branch_pattern: settings.github_branch_pattern.clone(),
        has_token,
        current_branch,
        last_pr_url: github_state.last_pr_url,
        last_pr_number: github_state.last_pr_number,
        last_push_time: github_state.last_push_time,
    })
}

/// Push changes to GitHub
#[tauri::command]
pub async fn push_meeting_changes(
    app: AppHandle,
    meeting_id: String,
    commit_message: Option<String>,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<PushResult, String> {
    ensure_github_allowed(&app)?;
    let settings = settings::get_settings(&app);

    // Validate settings
    let owner = settings
        .github_repo_owner
        .as_ref()
        .ok_or("GitHub repository owner not set")?;
    let repo = settings
        .github_repo_name
        .as_ref()
        .ok_or("GitHub repository name not set")?;

    // Get token
    let token = github::get_github_token().map_err(|e| format!("No GitHub token: {}", e))?;

    // Get meeting
    let meeting = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .map_err(|e| format!("{}", e))?;

    let project_path = meeting
        .project_path
        .as_ref()
        .ok_or("Meeting has no project path")?;

    // Initialize repo
    let repo_obj = github::init_git_repo(project_path).map_err(|e| e.to_string())?;

    // Generate branch name
    let branch_name =
        github::generate_branch_name(&settings.github_branch_pattern, &meeting_id, &meeting.name);

    // Check if we need to create branch
    let current_branch = github::get_current_branch(&repo_obj).unwrap_or_default();
    if current_branch != branch_name && current_branch == settings.github_default_branch {
        // Create and checkout new branch
        github::create_branch(&repo_obj, &branch_name).map_err(|e| e.to_string())?;
    }

    // Commit meeting files only for safety (transcript + .claude)
    let message = commit_message.unwrap_or_else(|| {
        format!(
            "Update meeting: {}\n\nAutomatically generated from Handy meeting session.",
            meeting.name
        )
    });

    github::commit_meeting_files(&repo_obj, &message, "Handy", "noreply@handy.computer")
        .map_err(|e| e.to_string())?;

    // Push to remote
    github::push_to_remote(project_path, &branch_name, &token, owner, repo)
        .map_err(|e| e.to_string())?;

    // Update GitHub state
    let mut github_state = github::read_github_state(project_path);
    github_state.last_branch = Some(branch_name.clone());
    github_state.last_push_time = Some(chrono::Utc::now().to_rfc3339());
    github::write_github_state(project_path, &github_state).map_err(|e| e.to_string())?;

    Ok(PushResult {
        success: true,
        branch: branch_name,
        commit_message: message,
        error: None,
    })
}

/// Create or update pull request
#[tauri::command]
pub async fn create_or_update_pr(
    app: AppHandle,
    meeting_id: String,
    title: Option<String>,
    body: Option<String>,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<PRResult, String> {
    ensure_github_allowed(&app)?;
    let settings = settings::get_settings(&app);

    // Validate settings
    let owner = settings
        .github_repo_owner
        .as_ref()
        .ok_or("GitHub repository owner not set")?;
    let repo = settings
        .github_repo_name
        .as_ref()
        .ok_or("GitHub repository name not set")?;

    // Get token
    let token = github::get_github_token().map_err(|e| format!("No GitHub token: {}", e))?;

    // Get meeting
    let meeting = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .map_err(|e| format!("{}", e))?;

    let project_path = meeting
        .project_path
        .as_ref()
        .ok_or("Meeting has no project path")?;

    // Read GitHub state
    let mut github_state = github::read_github_state(project_path);

    // Generate branch name
    let branch_name =
        github::generate_branch_name(&settings.github_branch_pattern, &meeting_id, &meeting.name);

    // Generate PR title and body
    let pr_title = title.unwrap_or_else(|| format!("Meeting: {}", meeting.name));
    let pr_body = body.unwrap_or_else(|| {
        format!(
            "# Meeting Summary\n\n**Meeting ID:** {}\n**Meeting Name:** {}\n\nThis PR contains updates from the MeetingCoder meeting session.\n\n---\n*Automatically generated with [MeetingCoder](https://github.com/Damonbodine/meetingcoder) (based on [Handy](https://github.com/cjpais/handy))*",
            meeting_id, meeting.name
        )
    });

    // Check if PR already exists for this branch
    let existing_prs = github::get_prs_for_branch(&token, owner, repo, &branch_name)
        .await
        .map_err(|e| e.to_string())?;

    let pr = if let Some(existing_pr) = existing_prs.first() {
        // Update existing PR
        github::update_pull_request(
            &token,
            owner,
            repo,
            existing_pr.number,
            Some(&pr_title),
            Some(&pr_body),
        )
        .await
        .map_err(|e| e.to_string())?
    } else {
        // Create new PR
        github::create_pull_request(
            &token,
            owner,
            repo,
            &pr_title,
            &pr_body,
            &branch_name,
            &settings.github_default_branch,
        )
        .await
        .map_err(|e| e.to_string())?
    };

    // Update GitHub state
    github_state.last_pr_url = Some(pr.html_url.clone());
    github_state.last_pr_number = Some(pr.number);
    github::write_github_state(project_path, &github_state).map_err(|e| e.to_string())?;

    Ok(PRResult {
        success: true,
        pr_number: Some(pr.number),
        pr_url: Some(pr.html_url),
        error: None,
    })
}

/// Post a comment on the PR with meeting update summary
#[tauri::command]
pub async fn post_meeting_update_comment(
    app: AppHandle,
    meeting_id: String,
    comment: Option<String>,
    meeting_manager: State<'_, Arc<MeetingManager>>,
) -> Result<bool, String> {
    ensure_github_allowed(&app)?;
    let settings = settings::get_settings(&app);

    // Validate settings
    let owner = settings
        .github_repo_owner
        .as_ref()
        .ok_or("GitHub repository owner not set")?;
    let repo = settings
        .github_repo_name
        .as_ref()
        .ok_or("GitHub repository name not set")?;

    // Get token
    let token = github::get_github_token().map_err(|e| format!("No GitHub token: {}", e))?;

    // Get meeting
    let meeting = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .map_err(|e| format!("{}", e))?;

    let project_path = meeting
        .project_path
        .as_ref()
        .ok_or("Meeting has no project path")?;

    // Read GitHub state
    let github_state = github::read_github_state(project_path);

    let pr_number = github_state
        .last_pr_number
        .ok_or("No PR found for this meeting")?;

    // Generate comment
    let comment_text = if let Some(c) = comment {
        c
    } else {
        // Default comment with current timestamp
        format!(
            "## Meeting Update\n\n**Meeting:** {}\n**Timestamp:** {}\n\nNew updates have been added to this meeting session.\n\n---\n*Automatically posted from Handy meeting session*",
            meeting.name,
            chrono::Utc::now().to_rfc3339()
        )
    };

    // Post comment
    github::post_pr_comment(&token, owner, repo, pr_number, &comment_text)
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// Begin GitHub OAuth Device Flow
#[tauri::command]
pub async fn github_begin_device_auth(app: AppHandle) -> Result<DeviceCodeResponse, String> {
    ensure_github_allowed(&app)?;
    github::begin_device_auth().await.map_err(|e| e.to_string())
}

/// Poll for GitHub OAuth Device Flow token
#[tauri::command]
pub async fn github_poll_device_token(
    app: AppHandle,
    device_code: String,
) -> Result<Option<String>, String> {
    ensure_github_allowed(&app)?;
    github::poll_device_token(&device_code)
        .await
        .map_err(|e| e.to_string())
}
