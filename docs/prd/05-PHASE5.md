# Phase 5: Discovery Mode — Live MVP Scaffolding

Phase 5 makes meetings produce repo‑native progress in real time. As soon as a discovery call starts, the app wires a repository, scaffolds a starter pack, installs dependencies, starts a dev server, and begins drafting specifications — all while keeping changes safe, reviewable, and under user control.

## Goals
- Accelerate discovery by scaffolding an MVP during the call.
- Keep everything repo‑native: docs, branches, PRs, and diffs.
- Remain safe by default: commit docs only unless explicitly approved to apply code scaffolds.
- Minimize ceremony for the other party; progress happens quietly in the background.

## Outcomes
- New repos: Private GitHub repo created and cloned locally; starter pack scaffolded; dev server running; draft PR opened on `discovery/{meeting_id}` with evolving docs.
- Existing repos: Repo cloned/attached; meeting docs and optional “experiment” pack are added safely (no default-branch changes); draft PR on `discovery/{meeting_id}`.
- Repo‑native artifacts: `.transcript.jsonl`, `docs/meeting/{id}/PRD.md`, `docs/meeting/{id}/decisions.md`, `docs/meeting/{id}/tasks.yml`.
- Optional, guarded code changes: small, unitizable scaffolds (pages/routes/stubs) applied behind approval, with diff previews and targeted commits.

## User Experience
- Discovery Wizard (on Start Meeting)
  - Choose: New repo vs Existing repo
  - If New: owner, repo name, privacy, starter pack selection
  - If Existing: select repo, placement (root vs `experiments/{meeting_id}`)
  - Toggles: auto-start dev server, auto-apply safe scaffolds (defaults on for new repos, off for existing)
- Live Setup Panel
  - Streaming logs: repo create/clone, scaffold copy, dependency install, dev server URL
  - Status preview: which files will be committed (docs vs code)
- Intent Panel (continuous)
  - Extracts acceptance criteria, tasks, architecture notes from transcript
  - Actions: “Add landing page”, “Add API route”, “Add form + validation”, “Add test stubs” → each shows a diff preview → Approve to apply
- PR Controls
  - Open draft PR at start; update PR body with PRD and task graph; major updates posted as comments with links

## Starter Packs
- Location: `resources/templates/packs/{pack_id}/` (bundled); each with `manifest.json`:
  - `name`, `description`, `install`, `dev`, `build`, `port`, `safePaths` (where generated code lands), `previewUrlHint`
- Initial packs (pilot):
  - `vite-react-tailwind`
  - `fastapi`
- Placement strategy:
  - New repo: scaffold in repo root
  - Existing repo: default to `experiments/{meeting_id}/` to avoid disrupting main code; opt-in to root placement

## Git + PR Workflow
- Local clone: `~/MeetingCoder/repos/{owner}/{repo}` (reused if present)
- Branch names: `discovery/{meeting_id}`; never commit to the default branch
- Commit scopes:
  - Docs-only (default): `.transcript.jsonl`, `docs/meeting/{id}/**`, `.claude/**`
  - Safe scaffolds: generated code in `safePaths`; opt-in toggle, diff preview required
- Pushing: each applied change commits and pushes to `discovery/{meeting_id}`
- Draft PR: opened at meeting start; labels `discovery`, `spec`, `needs-review`; PR body mirrors PRD and tasks; comments for deltas

## Backend Additions
- Repo + Clone
  - `create_or_select_repo(owner, name, private)` (create via GitHub API when new; or validate existing selection)
  - `ensure_local_repo_clone(owner, repo, token) -> local_path`
- Scaffolding
  - `scaffold_stack_pack(pack_id, local_path, placement)` → copies pack with manifest-driven commands
  - `seed_discovery_docs(local_path, meeting_id)` → create `docs/meeting/{id}/` and `.claude` state
- Installation + Dev Server
  - `install_dependencies(local_path, pack_id)` → stream logs (`bun install`/`npm ci`/`pip install -r requirements.txt`)
  - `start_dev_server(local_path, pack_id)` → spawn background process; record `{pid, port, url}` in `.claude/.discovery-state.json`; stream logs
- Status + Apply
  - `status_preview(meeting_id)` → changed files (scoped to docs and `safePaths`)
  - `apply_scaffold_action(meeting_id, action_id, params)` → generate diff, preview, commit if approved
- State Files
  - `.claude/.discovery-state.json` → `{ pack_id, branch, dev_server: { pid, port, url }, toggles }`

## Frontend Additions
- Components: `DiscoveryWizard`, `DiscoverySetupPanel`, `IntentPanel`, `DiffPreviewModal`
- Settings: toggles for doc-only commits, safe scaffolds, dev server auto-start
- Logs: install/dev server log streaming with truncation and “copy last N lines”
- Repo Picker: reuse Phase 4 picker for existing repos; add “Create New Repo” path

## Safety & Controls
- Defaults to safe: docs-only commits; code scaffolds require explicit approval with diff preview
- Isolation: for existing repos, scaffold under `experiments/{meeting_id}` by default
- Never push to default branch; use `discovery/{meeting_id}`
- Dry-run everywhere: status preview before any push; typed errors (token, rate-limit, conflicts)
- Ethics UX: honor recording consent policy; changes are visible in PR and repo history

## Pilot Scope (v0)
- Packs: `vite-react-tailwind` (first), `fastapi` (second)
- New repo path: create private repo via GitHub API, clone locally, scaffold pack in root
- Existing repo path: seed docs and `.claude`; scaffold pack under `experiments/{meeting_id}`
- Dev server: start automatically; show URL; allow quick open in browser
- Actions: “Add Landing Page”, “Add Contact Form” (web), “Add /health route”, “Add /items endpoint” (API)
- Commits: docs-only by default; safe scaffolds behind toggle; push per action

## Acceptance Criteria
- Starting a meeting in Discovery Mode:
  - Creates/attaches a repo; clones locally if not present
  - Scaffolds selected pack as per placement rules
  - Installs dependencies and starts a dev server; URL visible in UI
  - Opens a draft PR with PRD/decisions/tasks and ongoing updates
- Status preview shows exactly what will commit; docs-only by default
- Safe scaffold actions present diff previews and commit into `discovery/{id}`

## Metrics
- Time to first preview (target: < 90s for web pack)
- Docs commits per meeting; code scaffolds applied with approval
- PR open-to-merge cycle time for discovery branches
- Error rate on repo creation/clone/install steps

## Open Questions
- Create new repo by default or reuse an existing one when the user has one selected?
- Codespaces/Vercel/Netlify integration for shareable previews in v1 or v2?
- How aggressively to auto-apply safe scaffolds on existing repos by default?
- Where to persist pack manifests for updates (bundled only vs remote catalog)?

## Out of Scope (Phase 5)
- Full multi-pack catalog and remote updates
- Advanced CI gates (coverage, risk analysis)
- Organization-wide policy management

---
Owner: Product + Engineering
Timeline: 2–3 weeks for v0 pilot (2 packs), contingent on GitHub API + dev server stability.
