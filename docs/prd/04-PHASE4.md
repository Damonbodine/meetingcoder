# Phase 4: GitHub Integration

**Status:** ✅ Complete
**Objective:** Integrate GitHub to automatically push meeting changes and create pull requests.

## Overview

Phase 4 adds comprehensive GitHub integration to Handy, allowing users to:

- Securely store GitHub Personal Access Tokens using the system keychain
- Automatically push meeting transcripts and updates to GitHub repositories
- Create and update pull requests with meeting summaries
- Post meeting update comments to existing PRs
- Manage GitHub settings through the UI

## Features

### 1. Secure Token Storage

**Implementation:**
- Tokens are stored in the system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Uses the `keyring` crate for cross-platform support
- Tokens are never stored in plaintext in settings files
- Backend functions:
  - `store_github_token()` - Securely save token
  - `get_github_token()` - Retrieve token from keychain
  - `delete_github_token()` - Remove token from keychain

**Security:**
- Tokens are encrypted by the OS keychain service
- No token data is logged or exposed in UI
- Test connection feature validates token without exposing it

### 2. Repository Configuration

**Settings:**
- `github_repo_owner` - GitHub username or organization
- `github_repo_name` - Repository name
- `github_default_branch` - Base branch for PRs (default: "main")
- `github_branch_pattern` - Pattern for branch naming (default: "meeting/{meeting_id}")
- `github_enabled` - Master toggle for GitHub integration

**Branch Naming:**
The `github_branch_pattern` supports variables:
- `{meeting_id}` - Unique meeting identifier
- `{meeting_name}` - Sanitized meeting name

Example: `"meeting/{meeting_id}"` → `"meeting/abc123"`

### 3. Git Operations

**Implemented Operations:**

**Initialize Repository:**
```rust
init_git_repo(project_path: &str) -> Result<Repository>
```
- Opens existing git repo or initializes new one
- Sets up git in meeting project directory

**Create Branch:**
```rust
create_branch(repo: &Repository, branch_name: &str) -> Result<()>
```
- Creates new branch from current HEAD
- Automatically checks out the new branch

**Commit Changes:**
```rust
commit_changes(
    repo: &Repository,
    message: &str,
    author_name: &str,
    author_email: &str,
) -> Result<Oid>
```
- Stages all changes in the project
- Creates commit with provided message
- Uses "Handy" as author with noreply@handy.computer email

**Push to Remote:**
```rust
push_to_remote(
    project_path: &str,
    branch_name: &str,
    token: &str,
    owner: &str,
    repo: &str,
) -> Result<()>
```
- Pushes branch to GitHub using HTTPS with token auth
- Sets upstream tracking
- Uses git CLI for reliable authentication

### 4. GitHub API Integration

**Implemented API Calls:**

**Test Connection:**
```rust
test_github_connection(token: &str) -> Result<String>
```
- Validates token by calling `/user` endpoint
- Returns authenticated username
- Used by UI to verify token is valid

**Get Repository Info:**
```rust
get_repo_info(token: &str, owner: &str, repo: &str) -> Result<Value>
```
- Fetches repository metadata
- Validates repo exists and is accessible

**Create Pull Request:**
```rust
create_pull_request(
    token: &str,
    owner: &str,
    repo: &str,
    title: &str,
    body: &str,
    head: &str,
    base: &str,
) -> Result<GitHubPR>
```
- Creates new PR from head branch to base branch
- Returns PR number and URL
- Auto-generates title/body from meeting data if not provided

**Update Pull Request:**
```rust
update_pull_request(
    token: &str,
    owner: &str,
    repo: &str,
    pr_number: u32,
    title: Option<&str>,
    body: Option<&str>,
) -> Result<GitHubPR>
```
- Updates existing PR title and/or description
- Used to refresh PR with latest meeting info

**Post PR Comment:**
```rust
post_pr_comment(
    token: &str,
    owner: &str,
    repo: &str,
    pr_number: u32,
    comment: &str,
) -> Result<()>
```
- Posts comment to PR issue thread
- Used to add meeting update summaries

**Get PRs for Branch:**
```rust
get_prs_for_branch(
    token: &str,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<Vec<GitHubPR>>
```
- Lists open PRs for a specific branch
- Used to detect existing PRs before creating new ones

### 5. State Management

**GitHub State File:** `.claude/.github-state.json`

```json
{
  "repo_owner": "username",
  "repo_name": "repository",
  "default_branch": "main",
  "branch_pattern": "meeting/{meeting_id}",
  "last_branch": "meeting/abc123",
  "last_pr_url": "https://github.com/user/repo/pull/42",
  "last_pr_number": 42,
  "last_push_time": "2025-01-05T10:30:00Z"
}
```

**Purpose:**
- Tracks GitHub activity per meeting/project
- Persists PR associations for updates
- Stores last push timestamp for UI display

### 6. Tauri Commands

**Token Management:**
- `set_github_token(token: String)` - Save token to keychain
- `remove_github_token()` - Delete token from keychain
- `test_github_connection()` - Validate current token

**Repository Settings:**
- `set_github_repo(owner, name, default_branch?, branch_pattern?)` - Configure repo
- `set_github_enabled(enabled)` - Toggle integration on/off
- `get_github_repo_status(meeting_id)` - Get current GitHub status for meeting

**Git Operations:**
- `push_meeting_changes(meeting_id, commit_message?)` - Commit and push changes
- `create_or_update_pr(meeting_id, title?, body?)` - Create or update PR
- `post_meeting_update_comment(meeting_id, comment?)` - Post comment to PR

**Settings Commands:**
- `change_github_repo_owner_setting(owner)`
- `change_github_repo_name_setting(name)`
- `change_github_default_branch_setting(branch)`
- `change_github_branch_pattern_setting(pattern)`
- `change_github_enabled_setting(enabled)`

## Frontend Components

### Settings UI

**GitHubEnabled.tsx**
- Toggle switch to enable/disable GitHub integration
- Shows in General Settings under "GitHub Integration" section

**GitHubToken.tsx**
- Secure token input with show/hide toggle
- "Save" button to store token in keychain
- "Test Connection" button to validate token
- "Remove Token" button to delete token
- Displays connection status and authenticated username
- Link to GitHub token creation page

**GitHubRepo.tsx**
- Input fields for repository owner and name
- Live preview of repository URL
- Validates inputs and shows current configuration

**GitHubBranchSettings.tsx**
- Input for default branch (main/master)
- Input for branch naming pattern
- Documentation for available variables

### Meeting View Integration

**GitHubActions.tsx**
- Displays current GitHub status:
  - Active branch name
  - PR number and link (if exists)
  - Last push timestamp
- Action buttons:
  - "Push Changes" - Commits and pushes to GitHub
  - "Create/Update PR" - Opens or updates pull request
  - "Post Update" - Adds comment with latest meeting update
- Status messages for unconfigured/disabled states
- Real-time loading states for all operations

## Workflow Examples

### Setup Workflow

1. **Enable Integration:**
   - Go to Settings → GitHub Integration
   - Toggle "Enable GitHub Integration" on

2. **Configure Token:**
   - Create GitHub PAT at https://github.com/settings/tokens/new?scopes=repo
   - Paste token into "GitHub Personal Access Token" field
   - Click "Save"
   - Click "Test Connection" to verify

3. **Configure Repository:**
   - Enter repository owner (username or organization)
   - Enter repository name
   - Optionally customize default branch and branch pattern
   - Settings auto-save on change

### Meeting Workflow

1. **Start Meeting:**
   - Meeting creates project directory
   - Git repo is initialized automatically

2. **During Meeting:**
   - Transcripts and updates are saved to project files
   - Changes accumulate in working directory

3. **Push Changes:**
   - Click "Push Changes" in GitHub Actions panel
   - Handy commits all changes with auto-generated message
   - Creates feature branch using configured pattern
   - Pushes to GitHub
   - Updates UI with branch name and push time

4. **Create PR:**
   - Click "Create PR" after pushing
   - Handy generates PR title from meeting name
   - Auto-fills PR description with meeting summary
   - Opens PR against default branch
   - Displays PR number and link in UI

5. **Post Updates:**
   - As meeting continues, new updates are generated
   - Click "Push Changes" to update branch
   - Click "Post Update" to add comment to PR with latest summary
   - PR stays updated with meeting progress

## Technical Implementation

### Backend Structure

**Module:** `src-tauri/src/integrations/github.rs`

**Dependencies:**
- `git2` (v0.19) - Git operations
- `keyring` (v3.2) - Secure credential storage
- `base64` (v0.22) - Encoding utilities
- `reqwest` - HTTP client (already present)
- `serde`, `serde_json` - Serialization

**Key Functions:**
- Token management: `store_github_token`, `get_github_token`, `delete_github_token`
- State persistence: `read_github_state`, `write_github_state`
- Git operations: `init_git_repo`, `create_branch`, `commit_changes`, `push_to_remote`
- GitHub API: `test_github_connection`, `create_pull_request`, `update_pull_request`, `post_pr_comment`
- Utilities: `generate_branch_name`

**Command Module:** `src-tauri/src/commands/github.rs`
- Maps Tauri commands to integration functions
- Handles state management and error translation
- Validates settings before operations

### Frontend Structure

**Types:** `src/lib/types.ts`
- `GitHubRepoStatus` - Status response schema
- `GitHubConnectionTest` - Connection test result
- `PushResult` - Push operation result
- `PRResult` - PR creation/update result

**Settings Store:** `src/stores/settingsStore.ts`
- Added GitHub settings to schema
- Added updater functions for each setting
- Integrated with Tauri command bindings

**Components:**
- `src/components/settings/GitHub*.tsx` - Settings UI
- `src/components/meeting/GitHubActions.tsx` - Meeting view integration

## Error Handling

**Common Errors:**

1. **No Token:** Settings configured but token not found in keychain
   - Solution: Re-save token in settings

2. **Invalid Token:** Token expired or has insufficient permissions
   - Solution: Generate new token with `repo` scope

3. **Repository Not Found:** Owner/name incorrect or no access
   - Solution: Verify repository settings and token permissions

4. **Push Failed:** Authentication or network issues
   - Solution: Check token validity and network connection

5. **PR Creation Failed:** Branch doesn't exist or has no changes
   - Solution: Push changes before creating PR

**Error Logging:**
All GitHub operations log with "GITHUB" prefix:
- `GITHUB token stored securely in macOS Keychain`
- `GITHUB connection test successful for user: username`
- `GITHUB created and checked out branch: meeting/abc123`
- `GITHUB pushed branch meeting/abc123 to remote`
- `GITHUB created PR #42: https://github.com/user/repo/pull/42`

## Security Considerations

1. **Token Storage:**
   - Never stored in plaintext
   - OS-level encryption via keychain
   - No token logging or display

2. **Authentication:**
   - HTTPS with Bearer token authentication
   - Token validated before operations
   - Short-lived credential caching only

3. **Permissions:**
   - Minimum required scope: `repo`
   - Users control token access via GitHub settings
   - Token can be revoked at any time

4. **Data Privacy:**
   - Meeting content pushed only to user-specified repos
   - Users control repository visibility
   - No data sent to third parties

## Testing

**Manual Test Plan:**

1. **Token Management:**
   - [ ] Save token successfully
   - [ ] Test connection shows username
   - [ ] Test with invalid token shows error
   - [ ] Remove token clears keychain

2. **Repository Setup:**
   - [ ] Configure valid repo settings
   - [ ] Invalid repo shows error on operations
   - [ ] Branch pattern substitution works

3. **Git Operations:**
   - [ ] Push creates commit and branch
   - [ ] Branch naming follows pattern
   - [ ] Subsequent pushes update same branch

4. **PR Workflow:**
   - [ ] Create PR generates title and description
   - [ ] PR link opens in browser
   - [ ] Update PR modifies existing PR
   - [ ] Post comment adds to PR thread

5. **UI Integration:**
   - [ ] Settings show/hide based on enabled state
   - [ ] Meeting view shows GitHub actions
   - [ ] Status updates after operations
   - [ ] Error messages display clearly

## Future Enhancements

**Potential Additions:**

1. **Auto-push on Update:**
   - Automatically push after each meeting update
   - Configurable push frequency/threshold

2. **Multi-repo Support:**
   - Configure different repos per meeting
   - Repository templates/presets

3. **Enhanced PR Templates:**
   - Customizable PR title/body templates
   - Meeting-specific template variables

4. **Branch Management:**
   - Auto-delete merged branches
   - Branch cleanup on meeting end

5. **GitHub Actions Integration:**
   - Trigger CI/CD on push
   - Status checks in UI

6. **Conflict Resolution:**
   - Detect merge conflicts
   - UI for conflict resolution

7. **GitHub App:**
   - OAuth authentication flow
   - Fine-grained permissions
   - Organization-wide installation

## Dependencies

**Added to Cargo.toml:**
```toml
git2 = "0.19"
keyring = "3.2"
base64 = "0.22"
```

**Existing Dependencies Used:**
- `reqwest` (HTTP client)
- `serde`, `serde_json` (serialization)
- `tauri` (commands and state)

## Documentation

**User Documentation Needed:**

1. **Setup Guide:**
   - How to create GitHub PAT
   - Required token scopes
   - Repository configuration

2. **Usage Guide:**
   - Meeting workflow with GitHub
   - Understanding branch patterns
   - PR management best practices

3. **Troubleshooting:**
   - Common error solutions
   - Token permission issues
   - Network/connectivity problems

## Completion Checklist

- [x] Backend GitHub integration module
- [x] Secure token storage with keyring
- [x] Git operations (init, branch, commit, push)
- [x] GitHub API calls (PR, comments)
- [x] State management and persistence
- [x] Tauri commands for all operations
- [x] Settings schema updates
- [x] Frontend type definitions
- [x] Settings store integration
- [x] Settings UI components
- [x] Meeting view GitHub actions
- [x] PR status display
- [x] Error handling and logging
- [x] Documentation

## Notes

- All GitHub operations use structured logging with "GITHUB" prefix
- Token authentication uses Bearer token in Authorization header
- User-Agent set to "Handy-App" for GitHub API calls
- Git commits use "Handy <noreply@handy.computer>" as author
- PR descriptions include Handy attribution link
