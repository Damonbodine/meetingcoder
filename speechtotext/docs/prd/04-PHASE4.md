# Phase 4: GitHub Integration (Repo Attach/Create)

## Overview

Phase 4 connects MeetingCoder projects to GitHub. Users can securely connect their GitHub account, choose an existing repository (including organization repos they have access to), or create a new repository, and attach their meeting project. When no repository exists, MeetingCoder initializes a Git repo locally, links the remote, and pushes an initial commit.

**Timeline**: 2–3 weeks
**Priority**: P1 (Distribution + collaboration)
**Dependencies**: Phase 2 + Phase 3 working

## Goals

1. Securely connect to GitHub using OAuth Device Flow
2. List and search personal and organization repositories
3. Create a new repository (public or private) and attach the local project
4. Attach an existing MeetingCoder project to a selected repository, safely handling existing remotes

## User Stories

- As a user, I can connect my GitHub account from Settings using a secure OAuth device flow.
- As a user, I can search and select an existing GitHub repository (including organization repos I have access to) to attach to my current meeting project.
- As a user, I can create a new GitHub repository (name defaults to project name, choose visibility, optionally select an organization) and attach it.
- As a user, I can safely link my local Git repo with proper remote handling (replace `origin` or add a secondary remote) and push the initial commit.

## UX Flow

1) Connect GitHub (Settings)
   - Start device flow → open GitHub verification page → enter code → poll until access token issued
   - Store token securely (macOS Keychain via `keyring`). Never log tokens
2) Choose Repo
   - List repos from `GET /user/repos` and organizations from `GET /user/orgs` then `GET /orgs/{org}/repos`
   - Search/filter by name; show visibility and owner
3) Attach or Create
   - If attaching: set remote on the local project. If `.git` missing, run `git init` (via `git2`), write `.gitignore` if absent, commit current files, set `origin`, and push `main`
   - If creating: `POST /user/repos` or `/orgs/{org}/repos` (optionally auto-init README), then attach as above
4) Safety
   - If an `origin` already exists: prompt to Replace or Add `github` as a secondary remote. Avoid destructive actions without user consent

## Technical Design

### Backend (Rust/Tauri)

- Module: `src-tauri/src/integrations/github.rs`
  - Device flow auth (GitHub OAuth) and token storage via `keyring`
  - REST calls with `reqwest` (respect rate limits; surface helpful errors)
  - Git operations via `git2`: init, add, commit, set remote, push HTTPS with token

- Commands
  - `github_begin_device_auth()` → returns `user_code`, `verification_uri`
  - `github_poll_token(device_code)` → resolves access token
  - `github_list_repos(query?, include_orgs?)` → returns repos user can access
  - `github_create_repo(name, private, org?)` → returns `full_name`, `clone_url`
  - `github_attach_project(meeting_id, repo_full_name, replace_origin?)` → performs local git wiring and push

### Frontend (React)

- Settings → GitHub Integration panel
  - Connect/Disconnect (status), Start Device Flow (copy code, open browser)
  - Repo Picker (with search), toggle include org repos
  - Create Repo form (name defaulted from project, visibility, org dropdown if member)

- Meeting View → Attach Repo shortcut
  - Shows current repo status and “Attach to GitHub…” for the active project

### Scopes & Security

- OAuth scopes: `public_repo` (public-only) or `repo` (for private repos); add `read:org` to enumerate org repos
- Store token in OS keychain; never persist in project files. Allow easy Disconnect (delete token)
- Respect org policies; surface permission errors clearly (e.g., request approval from org admin)

## Acceptance Criteria

- Can connect GitHub and persist token securely
- Can list personal and organization repositories and create a new repo
- Can attach the current project to a selected or newly created repository
- Handles pre-existing `.git` remotes with a clear Replace/Add choice; no accidental destructive actions
- Push succeeds for a new repo; for non-empty remotes, fetch guidance is provided (e.g., create new branch or pull/merge)

## Testing Plan

- Use a test GitHub account (with and without org membership)
- Verify device flow completes and token is stored; disconnect clears token
- Attach to an existing empty repo → initial push succeeds
- Attach to a non-empty repo → prompt, fetch, and non-destructive behavior verified
- Create private/public repos and attach; confirm visibility and ownership
- Ensure transcription/summarization loop continues unaffected during any failures

