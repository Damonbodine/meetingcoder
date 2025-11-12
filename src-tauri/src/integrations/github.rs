use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::fs;
use std::env;

const KEYCHAIN_SERVICE: &str = "com.handy.github";
const KEYCHAIN_ACCOUNT: &str = "github_token";

// Fallback token storage path for when keyring fails (development mode)
fn get_token_fallback_path() -> Result<std::path::PathBuf> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| anyhow!("Could not determine home directory"))?;
    let config_dir = Path::new(&home).join(".handy");
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join(".github-token"))
}

/// GitHub integration state persisted to .claude/.github-state.json
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct GitHubState {
    pub repo_owner: Option<String>,
    pub repo_name: Option<String>,
    pub default_branch: String,
    pub branch_pattern: String,
    pub last_branch: Option<String>,
    pub last_pr_url: Option<String>,
    pub last_pr_number: Option<u32>,
    pub last_push_time: Option<String>,
}

impl GitHubState {
    pub fn new() -> Self {
        Self {
            repo_owner: None,
            repo_name: None,
            default_branch: "main".to_string(),
            branch_pattern: "meeting/{meeting_id}".to_string(),
            last_branch: None,
            last_pr_url: None,
            last_pr_number: None,
            last_push_time: None,
        }
    }
}

/// GitHub API response for PR creation
#[derive(Deserialize, Debug)]
pub struct GitHubPR {
    pub number: u32,
    pub html_url: String,
    pub state: String,
}

/// GitHub API request for creating a PR
#[derive(Serialize, Debug)]
pub struct CreatePRRequest {
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
}

/// GitHub API request for updating a PR
#[derive(Serialize, Debug)]
pub struct UpdatePRRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// GitHub API request for creating a PR comment
#[derive(Serialize, Debug)]
pub struct CreateCommentRequest {
    pub body: String,
}

/// Read GitHub state from project directory
pub fn read_github_state(project_path: &str) -> GitHubState {
    let path = Path::new(project_path).join(".claude/.github-state.json");
    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(state) = serde_json::from_slice::<GitHubState>(&bytes) {
            return state;
        }
    }
    GitHubState::new()
}

/// Write GitHub state to project directory
pub fn write_github_state(project_path: &str, state: &GitHubState) -> Result<()> {
    let p = Path::new(project_path).join(".claude/.github-state.json");
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, serde_json::to_vec_pretty(state)?)?;
    Ok(())
}

/// Ensure a local clone of the selected GitHub repository exists and return its path
/// Layout: ~/MeetingCoder/repos/{owner}/{repo}
pub fn ensure_local_repo_clone(owner: &str, repo: &str, token: &str) -> Result<String> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let base = home.join("MeetingCoder").join("repos").join(owner);
    fs::create_dir_all(&base)?;
    let dest = base.join(repo);

    // If already cloned, return path
    if dest.join(".git").exists() {
        log::info!("GITHUB using existing local clone: {}", dest.display());
        return Ok(dest.to_string_lossy().to_string());
    }

    // Clone using token in URL for simplicity (dev mode); production should rely on keychain/credential helper
    let remote_url = format!("https://{}@github.com/{}/{}.git", token, owner, repo);
    let output = Command::new("git")
        .arg("clone")
        .arg(&remote_url)
        .arg(&dest)
        .output()?;
    if !output.status.success() {
        return Err(anyhow!(
            "Git clone failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    log::info!("GITHUB cloned repo to {}", dest.display());
    Ok(dest.to_string_lossy().to_string())
}

/// Store GitHub token securely using keyring with fallback
pub fn store_github_token(token: &str) -> Result<()> {
    log::info!("GITHUB attempting to store token (length: {})", token.len());

    // Try keyring first
    let keyring_result = (|| -> Result<()> {
        let entry = keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
            .map_err(|e| anyhow!("Failed to create keyring entry: {}", e))?;

        entry.set_password(token)
            .map_err(|e| anyhow!("Failed to set password in keyring: {}", e))?;

        log::info!("GITHUB token stored in system keyring");

        // Verify we can read it back
        entry.get_password()
            .map_err(|e| anyhow!("Verification failed: {}", e))?;

        log::info!("GITHUB token verified in keyring");
        Ok(())
    })();

    match keyring_result {
        Ok(_) => {
            log::info!("GITHUB token stored successfully via keyring");
            // Also store in fallback for reliability
            if let Ok(path) = get_token_fallback_path() {
                let _ = fs::write(&path, token);
                log::info!("GITHUB token also stored in fallback location");
            }
            Ok(())
        }
        Err(e) => {
            log::warn!("GITHUB keyring storage failed: {}, using fallback", e);
            // Use fallback storage
            let path = get_token_fallback_path()?;
            fs::write(&path, token)?;
            log::info!("GITHUB token stored in fallback location: {:?}", path);
            Ok(())
        }
    }
}

/// Retrieve GitHub token from keyring with fallback
pub fn get_github_token() -> Result<String> {
    log::info!("GITHUB attempting to retrieve token");

    // Try keyring first
    let keyring_result = (|| -> Result<String> {
        let entry = keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
            .map_err(|e| anyhow!("Failed to create keyring entry: {}", e))?;

        let token = entry.get_password()
            .map_err(|e| anyhow!("Failed to get password from keyring: {}", e))?;

        log::info!("GITHUB token retrieved from keyring (length: {})", token.len());
        Ok(token)
    })();

    match keyring_result {
        Ok(token) => Ok(token),
        Err(e) => {
            log::warn!("GITHUB keyring retrieval failed: {}, trying fallback", e);
            // Try fallback
            let path = get_token_fallback_path()?;
            if path.exists() {
                let token = fs::read_to_string(&path)?;
                log::info!("GITHUB token retrieved from fallback (length: {})", token.len());
                Ok(token)
            } else {
                Err(anyhow!("No token found in keyring or fallback"))
            }
        }
    }
}

/// Delete GitHub token from keyring and fallback
pub fn delete_github_token() -> Result<()> {
    // Try to delete from keyring
    if let Ok(entry) = keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT) {
        let _ = entry.delete_credential();
        log::info!("GITHUB token removed from keyring");
    }

    // Also delete fallback
    if let Ok(path) = get_token_fallback_path() {
        if path.exists() {
            fs::remove_file(&path)?;
            log::info!("GITHUB token removed from fallback");
        }
    }

    Ok(())
}

/// Test GitHub token by making an authenticated API call
pub async fn test_github_connection(token: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Handy-App")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "GitHub API error: {}",
            response.status()
        ));
    }

    let user_data: serde_json::Value = response.json().await?;
    let username = user_data["login"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    log::info!("GITHUB connection test successful for user: {}", username);
    Ok(username)
}

/// Repository info for listing
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RepoInfo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: RepoOwner,
    pub private: bool,
    pub description: Option<String>,
    pub html_url: String,
    pub default_branch: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RepoOwner {
    pub login: String,
}

/// Get list of user's repositories
pub async fn list_user_repos(token: &str) -> Result<Vec<RepoInfo>> {
    let client = reqwest::Client::new();

    // Fetch both user repos and org repos
    let mut all_repos = Vec::new();

    // Get user's own repos
    let user_url = "https://api.github.com/user/repos?per_page=100&sort=updated";
    let response = client
        .get(user_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Handy-App")
        .send()
        .await?;

    if response.status().is_success() {
        let mut repos: Vec<RepoInfo> = response.json().await?;
        all_repos.append(&mut repos);
    }

    log::info!("GITHUB fetched {} repositories", all_repos.len());
    Ok(all_repos)
}

/// Get repository information
pub async fn get_repo_info(token: &str, owner: &str, repo: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Handy-App")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to get repo info: {}",
            response.status()
        ));
    }

    let repo_data: serde_json::Value = response.json().await?;
    Ok(repo_data)
}

/// Initialize or get existing git repository
pub fn init_git_repo(project_path: &str) -> Result<git2::Repository> {
    let path = Path::new(project_path);

    // Try to open existing repo
    match git2::Repository::open(path) {
        Ok(repo) => {
            log::info!("GITHUB opened existing git repo at {}", project_path);
            Ok(repo)
        }
        Err(_) => {
            // Initialize new repo
            let repo = git2::Repository::init(path)?;
            log::info!("GITHUB initialized new git repo at {}", project_path);
            Ok(repo)
        }
    }
}

/// Create and checkout a new branch
pub fn create_branch(repo: &git2::Repository, branch_name: &str) -> Result<()> {
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;

    // Create branch
    repo.branch(branch_name, &commit, false)?;

    // Checkout branch
    let obj = repo.revparse_single(&format!("refs/heads/{}", branch_name))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&format!("refs/heads/{}", branch_name))?;

    log::info!("GITHUB created and checked out branch: {}", branch_name);
    Ok(())
}

/// Get current branch name
pub fn get_current_branch(repo: &git2::Repository) -> Result<String> {
    let head = repo.head()?;
    let branch_name = head.shorthand().unwrap_or("unknown").to_string();
    Ok(branch_name)
}

/// Commit changes with a message
/// Commit meeting-related files only (does not stage the whole tree)
pub fn commit_meeting_files(
    repo: &git2::Repository,
    message: &str,
    author_name: &str,
    author_email: &str,
) -> Result<git2::Oid> {
    let mut index = repo.index()?;

    // Add meeting artifacts only by default for safety
    // - transcript file and .claude metadata
    // This avoids committing unrelated source changes automatically.
    let specs = [".transcript.jsonl", ".claude/*"];
    index.add_all(specs.iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let signature = git2::Signature::now(author_name, author_email)?;

    // Get parent commit
    let parent_commit = match repo.head() {
        Ok(head) => Some(head.peel_to_commit()?),
        Err(_) => None,
    };

    let parents = match &parent_commit {
        Some(p) => vec![p],
        None => vec![],
    };

    let oid = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )?;

    log::info!("GITHUB committed meeting files: {}", oid);
    Ok(oid)
}

/// Push branch to remote using git command (libgit2 auth can be complex)
pub fn push_to_remote(
    project_path: &str,
    branch_name: &str,
    token: &str,
    owner: &str,
    repo: &str,
) -> Result<()> {
    // Set up remote URL with token authentication
    let remote_url = format!(
        "https://{}@github.com/{}/{}.git",
        token, owner, repo
    );

    // Use git command for push (simpler authentication)
    let output = Command::new("git")
        .current_dir(project_path)
        .arg("push")
        .arg(&remote_url)
        .arg(format!("{}:{}", branch_name, branch_name))
        .arg("--set-upstream")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Git push failed: {}", stderr));
    }

    log::info!("GITHUB pushed branch {} to remote", branch_name);
    Ok(())
}

/// Create a pull request on GitHub
pub async fn create_pull_request(
    token: &str,
    owner: &str,
    repo: &str,
    title: &str,
    body: &str,
    head: &str,
    base: &str,
) -> Result<GitHubPR> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo);

    let request = CreatePRRequest {
        title: title.to_string(),
        body: body.to_string(),
        head: head.to_string(),
        base: base.to_string(),
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Handy-App")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(anyhow!(
            "Failed to create PR ({}): {}",
            status,
            error_text
        ));
    }

    let pr: GitHubPR = response.json().await?;
    log::info!("GITHUB created PR #{}: {}", pr.number, pr.html_url);
    Ok(pr)
}

/// Update an existing pull request
pub async fn update_pull_request(
    token: &str,
    owner: &str,
    repo: &str,
    pr_number: u32,
    title: Option<&str>,
    body: Option<&str>,
) -> Result<GitHubPR> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, pr_number
    );

    let request = UpdatePRRequest {
        title: title.map(|s| s.to_string()),
        body: body.map(|s| s.to_string()),
        state: None,
    };

    let response = client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Handy-App")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(anyhow!(
            "Failed to update PR ({}): {}",
            status,
            error_text
        ));
    }

    let pr: GitHubPR = response.json().await?;
    log::info!("GITHUB updated PR #{}", pr.number);
    Ok(pr)
}

/// Post a comment on a pull request
pub async fn post_pr_comment(
    token: &str,
    owner: &str,
    repo: &str,
    pr_number: u32,
    comment: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues/{}/comments",
        owner, repo, pr_number
    );

    let request = CreateCommentRequest {
        body: comment.to_string(),
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Handy-App")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(anyhow!(
            "Failed to post comment ({}): {}",
            status,
            error_text
        ));
    }

    log::info!("GITHUB posted comment on PR #{}", pr_number);
    Ok(())
}

/// Get list of open PRs for a branch
pub async fn get_prs_for_branch(
    token: &str,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<Vec<GitHubPR>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls?head={}:{}&state=open",
        owner, repo, owner, branch
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Handy-App")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to get PRs: {}",
            response.status()
        ));
    }

    let prs: Vec<GitHubPR> = response.json().await?;
    Ok(prs)
}

/// Generate branch name from meeting ID using the pattern
pub fn generate_branch_name(pattern: &str, meeting_id: &str, meeting_name: &str) -> String {
    let sanitized_name = meeting_name
        .to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    pattern
        .replace("{meeting_id}", meeting_id)
        .replace("{meeting_name}", &sanitized_name)
}

// ===== OAuth Device Flow =====

const GITHUB_CLIENT_ID: &str = "Ov23liUutHAz1Qx5xvSy"; // MeetingCoder app client ID

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u32,
}

#[derive(Deserialize, Debug)]
pub struct DeviceTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DeviceTokenPollResponse {
    Success(DeviceTokenResponse),
    Error { error: String },
}

/// Initiate OAuth Device Flow
pub async fn begin_device_auth() -> Result<DeviceCodeResponse> {
    let client = reqwest::Client::new();

    let mut params = std::collections::HashMap::new();
    params.insert("client_id", GITHUB_CLIENT_ID);
    params.insert("scope", "repo");

    let response = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Device auth failed: {}", error_text));
    }

    let device_code: DeviceCodeResponse = response.json().await?;
    log::info!("GITHUB device auth initiated: {}", device_code.user_code);

    Ok(device_code)
}

/// Poll for OAuth Device Flow token
pub async fn poll_device_token(device_code: &str) -> Result<Option<String>> {
    let client = reqwest::Client::new();

    let mut params = std::collections::HashMap::new();
    params.insert("client_id", GITHUB_CLIENT_ID);
    params.insert("device_code", device_code);
    params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");

    let response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await?;

    let poll_response: DeviceTokenPollResponse = response.json().await?;

    match poll_response {
        DeviceTokenPollResponse::Success(token) => {
            log::info!("GITHUB device token received");
            Ok(Some(token.access_token))
        },
        DeviceTokenPollResponse::Error { error } => {
            if error == "authorization_pending" || error == "slow_down" {
                // User hasn't authorized yet, return None to continue polling
                Ok(None)
            } else {
                Err(anyhow!("Device token poll error: {}", error))
            }
        }
    }
}
