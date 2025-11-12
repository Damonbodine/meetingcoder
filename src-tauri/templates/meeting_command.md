Read new updates from .meeting-updates.jsonl and update the codebase accordingly.

## Steps

1. **Check for updates file**
   - Look for `.meeting-updates.jsonl` in the current working directory
   - If not found, respond: "No .meeting-updates.jsonl file found. Is this a MeetingCoder project?"

2. **Load state and determine mode**
   - Read `.claude/.meeting-state.json` to get:
     - `last_processed_update_id` (string like `"u12"`)
     - `mode` (either `"developer"` or `"starter_kit"`)
     - `project_metadata` (optional: repo info, framework, languages)
   - If file doesn't exist, initialize it with:
     ```json
     {
       "last_processed_update_id": null,
       "last_processed_timestamp": null,
       "total_updates_processed": 0,
       "mode": "starter_kit",
       "project_metadata": null
     }
     ```
   - Mode determines behavior:
     - **Developer Mode**: Working on existing repository, use experiments folder, preserve existing code
     - **Starter Kit Mode**: Building from scratch, scaffold complete application structure

3. **Read new updates**
   - Read `.meeting-updates.jsonl` line by line (JSONL)
   - Each line has: top-level metadata (`update_id` as string "uN", `meeting_id`, `meeting_name`, `model`, `source`), and flattened update fields
   - Update fields include both legacy lists and enriched schema:
     - Legacy: `timestamp`, `segment_range`, `new_features: string[]`, `technical_decisions: string[]`, `questions: string[]`
     - Enriched: `new_features_structured: Feature[]`, `modified_features: { [feature_id]: Partial<Feature> }`, `clarifications: { [feature_id]: string }`
   - Skip lines with malformed JSON safely and continue
   - Select only lines with `update_id` strictly greater than `last_processed_update_id` (compare numeric suffixes)
   - If no new updates, respond: "No new meeting updates to process. Waiting for more discussion..."

3a. **Optional transcript context**
   - If `.transcript.jsonl` exists in the project root, read the last 50 lines and use them as additional raw context for disambiguation and naming
   - Each line contains: `meeting_id`, `segment_index`, `speaker`, `start_time`, `end_time`, `confidence`, `text`, `timestamp`
   - Use these only as context; do not duplicate them into update summaries

3b. **Analyze existing codebase (Developer Mode only)**
   - If mode is `"developer"` and this is the FIRST update:
     - Analyze the existing repository structure:
       - List all source files (exclude node_modules, .git, build artifacts)
       - Detect framework/tech stack from package.json, tsconfig.json, etc.
       - Identify key directories (src/, components/, pages/, etc.)
       - Extract entry points and main components
     - Update `.claude/.meeting-state.json` with `project_metadata`:
       ```json
       {
         "framework": "nextjs|react|vue|express|fastapi|etc",
         "languages": ["typescript", "tsx"],
         "entry_points": ["src/app/layout.tsx"],
         "key_directories": ["src/components/", "src/app/", "src/lib/"],
         "safe_paths": ["experiments/"]
       }
       ```
     - Write summary to first update: "Analyzed existing codebase: Next.js 14 app with TypeScript, 45 components, App Router"

4. **Process first update (project initialization)**
   - If this is the FIRST update (`last_processed_update_id` is null):

   **Starter Kit Mode:**
     - Extract `project_name`, `project_type`, and `tech_stack` from the update
     - For web applications with authentication/database needs, use **Vercel + Supabase + Next.js**:
       - Initialize Next.js 14 with App Router: `npx create-next-app@latest --typescript --tailwind --app`
       - Install Supabase client: `npm install @supabase/supabase-js @supabase/ssr`
       - Create environment template (`.env.local.example`):
         ```
         NEXT_PUBLIC_SUPABASE_URL=your-project-url
         NEXT_PUBLIC_SUPABASE_ANON_KEY=your-anon-key
         ```
       - Set up Supabase client (`src/lib/supabase.ts`) with cookie-based auth
       - Create auth pages: `src/app/login/page.tsx`, `src/app/signup/page.tsx`
       - Create protected route middleware: `src/middleware.ts`
       - Set up basic layout with auth status: `src/app/layout.tsx`
     - Alternative project types (if explicitly requested):
       - `node_api`: Express + TypeScript + basic middleware
       - `python_cli`: Click + proper package structure
       - `static_website`: HTML + CSS + vanilla JS
     - Implement HIGH priority features from the first batch
     - Create comprehensive README with:
       - Setup instructions (npm install, env vars, supabase setup)
       - Development commands (npm run dev)
       - Project structure overview
       - Next steps for local development

   **Developer Mode:**
     - Create `experiments/{meeting_id}/` directory for new features
     - Do NOT modify existing core files without explicit instruction
     - Use codebase analysis from step 3b to understand existing patterns
     - Implement HIGH priority features in experiments folder first
     - Create `experiments/{meeting_id}/README.md` explaining the new features and how to integrate them

5. **Process subsequent updates (incremental changes)**
   - For each new update after the first:
     - **New features**: Prefer `new_features_structured` to drive concrete work. Use `priority` to order work (`high` first), and `description` for acceptance
       - If `new_features_structured` is absent, fall back to legacy `new_features: string[]`
     - **Modified features**: Update existing code to match changes
     - **Clarifications**: Refine implementations based on answered questions (map keyed by `feature_id`)
     - **Technical decisions**: Apply architectural or tech stack changes
     - **Target files** (if specified): Use `target_files` field to identify which files to modify

   **Mode-Specific Behavior:**

   **Starter Kit Mode:**
     - Create new features directly in appropriate directories:
       - React components: `src/components/`
       - Next.js pages: `src/app/[route]/page.tsx`
       - API routes: `src/app/api/[route]/route.ts`
       - Utility functions: `src/lib/`
       - Supabase queries: `src/lib/supabase/queries.ts`
     - Follow Next.js App Router conventions
     - Use Tailwind CSS for all styling
     - Use Supabase client for all database/auth operations
     - Ensure TypeScript strict mode compliance

   **Developer Mode:**
     - Default to `experiments/{meeting_id}/` for new code
     - Only modify existing files when:
       - Explicitly mentioned in transcript/target_files
       - Integration is specifically requested
       - Change is clearly safe (adding to exports, non-breaking additions)
     - When modifying existing files:
       - Preserve existing patterns and style
       - Add comments explaining integration with experiments
       - Use feature flags if appropriate
     - Provide integration instructions in experiments README

6. **Code generation guidelines**
   - Maintain consistency with existing code style
   - Add comments explaining key logic and non-obvious decisions
   - Include error handling and input validation
   - Follow best practices for the tech stack
   - Implement MVP functionality (don't over-engineer)
   - Use modern patterns and libraries
   - Ensure generated code is syntactically valid
   - **Type safety**: Use TypeScript strictly, no `any` types
   - **Error boundaries**: Wrap async operations in try-catch
   - **Loading states**: Include loading/error UI states for async operations
   - **Accessibility**: Use semantic HTML and ARIA labels
   - **Performance**: Avoid unnecessary re-renders, use React.memo where appropriate

7. **File isolation safety (Developer Mode only)**
   - Before creating or modifying ANY file in Developer Mode, verify path safety:
     - Check if `.claudeignore` exists in project root
     - Ensure target path is within allowed directories:
       - ✅ `experiments/{meeting_id}/` - Primary workspace for new code
       - ✅ `.claude/` - Meeting state and metadata
       - ✅ `tests/`, `test/`, `__tests__/` - New test files
       - ❌ `src/`, `app/`, `components/`, `lib/` - Core application code (read-only unless explicitly requested)
       - ❌ `package.json`, `*.config.*` - Configuration files (read-only)
       - ❌ `.env*`, `*.key`, `credentials.*` - Secrets (never access)
     - If attempting to modify protected files:
       - **STOP** and provide alternative in experiments folder
       - Explain: "For safety, new code should go in `experiments/{meeting_id}/`. To integrate with `{protected_file}`, see the integration guide in `experiments/{meeting_id}/README.md`."
       - Only proceed if user explicitly confirms modification in transcript
     - All new features default to `experiments/{meeting_id}/src/` structure
     - Create integration documentation showing how to merge into main codebase

8. **Validate generated code**
   - After generating code for each update, run validation checks:

   **TypeScript/JavaScript Projects:**
     - Check syntax: Run TypeScript compiler in check mode (`tsc --noEmit`)
     - Verify imports: Ensure all imports resolve correctly
     - Check for missing dependencies: Look for unresolved module imports
     - If validation fails:
       - Capture error messages
       - Attempt to fix (add missing imports, install dependencies, correct syntax)
       - Retry validation (max 3 attempts)
       - If still failing, note in summary and continue with next update

   **Build Validation (Starter Kit Mode only):**
     - After processing all updates, attempt to run build command:
       - Next.js: `npm run build`
       - React/Vite: `npm run build`
     - If build fails:
       - Parse error output for specific issues
       - Attempt automatic fixes for common errors:
         - Missing dependencies → run `npm install {package}`
         - Type errors → add proper types or fix type mismatches
         - Import errors → correct import paths
       - Retry build (max 2 attempts)
       - If build still fails, provide clear error summary with fix suggestions

   **Error Recovery:**
     - Track failed features in state: `"failed_features": [{"update_id": "u5", "feature": "contact form", "error": "..."}]`
     - Continue processing subsequent updates even if one fails
     - Provide diagnostic information for manual intervention

9. **Update state**
   - After processing all new updates, update `.claude/.meeting-state.json`:
     ```json
     {
       "last_processed_update_id": "<latest_update_id>",
       "last_processed_timestamp": "<current_timestamp>",
       "total_updates_processed": <count>,
       "mode": "developer|starter_kit",
       "project_metadata": {...},
       "failed_features": [...],
       "build_status": "success|failed|not_run"
     }
     ```

10. **Provide detailed summary**
   - List all changes made in this batch
   - Format: "Update {update_id}: {feature_title} [✅ Success / ⚠️ Partial / ❌ Failed]"
   - Include files created, modified, or deleted with line counts
   - Show validation/build results
   - Provide next steps or integration instructions
   - For failures, suggest fixes or clarifications needed
   - Overall progress indicator: "Processed 5 updates, 4 successful, 1 failed"

## Example Workflows

### Starter Kit Mode - First Invocation (u1)
```
Reading .meeting-updates.jsonl...
Found first update: Initializing customer-feedback-app

Mode: Starter Kit
Project type: Web application with authentication
Tech stack: Next.js 14, TypeScript, Supabase, Tailwind CSS

Creating project structure...
- ✅ Initialized Next.js 14 with App Router
- ✅ Installed @supabase/supabase-js and @supabase/ssr
- ✅ Created src/lib/supabase.ts (client setup)
- ✅ Created src/app/login/page.tsx
- ✅ Created src/app/signup/page.tsx
- ✅ Created src/middleware.ts (auth protection)
- ✅ Created .env.local.example
- ✅ Created README.md with setup instructions

Implemented 2 high-priority features:
  1. User authentication (email/password via Supabase)
  2. Protected dashboard route

Validation: ✅ TypeScript check passed
Build: ⚠️ Skipped (requires Supabase env vars)

Next steps:
1. Copy .env.local.example to .env.local
2. Create Supabase project at https://supabase.com
3. Add your SUPABASE_URL and SUPABASE_ANON_KEY
4. Run `npm install && npm run dev`
5. Visit http://localhost:3000
```

### Starter Kit Mode - Subsequent Updates (u2-u3)
```
Reading .meeting-updates.jsonl...
Found 2 new updates since last check (u2, u3)

Update u2: Feedback Submission Form ✅
- Created src/components/FeedbackForm.tsx (142 lines)
- Created src/app/api/feedback/route.ts (API endpoint)
- Created src/lib/supabase/queries.ts (database operations)
- Updated src/app/dashboard/page.tsx to include form

Update u3: Feedback History View ✅
- Created src/components/FeedbackList.tsx (98 lines)
- Updated src/app/dashboard/page.tsx with history section
- Added real-time subscription for new feedback

Validation: ✅ TypeScript check passed (0 errors)
Build: ✅ Next.js build successful

Summary: Added feedback submission and history features. Users can now submit feedback and see all their past submissions with real-time updates.

Files modified: 6 files, +340 lines, -15 lines
```

### Developer Mode - First Invocation (u1)
```
Reading .meeting-updates.jsonl...
Found first update in existing repository

Mode: Developer
Analyzing existing codebase...
- Detected: Next.js 14 app with TypeScript
- 47 existing components in src/components/
- App Router pattern with src/app/
- Tailwind CSS, Supabase client already configured

Analysis summary saved to .claude/.meeting-state.json

Creating experiments/20250107-feature-discussion/
- ✅ Created experiments directory for new features
- ✅ Created experiments/20250107-feature-discussion/README.md

Implemented 1 high-priority feature in experiments:
  1. User profile editing component (ProfileEditor.tsx)

Validation: ✅ TypeScript check passed

Next steps:
1. Review new component at experiments/20250107-feature-discussion/ProfileEditor.tsx
2. Test component in isolation
3. When ready, integrate into main app (see experiments/README.md for instructions)
```

### Developer Mode - Subsequent Updates (u2-u3)
```
Reading .meeting-updates.jsonl...
Found 2 new updates since last check (u2, u3)

Update u2: Avatar Upload Feature ✅
- Created experiments/20250107-feature-discussion/AvatarUpload.tsx (187 lines)
- Created experiments/20250107-feature-discussion/hooks/useFileUpload.ts
- Integrated with existing Supabase storage patterns

Update u3: Profile Integration with Existing UserSettings ✅
- Modified src/components/UserSettings.tsx (+45 lines, preserved existing logic)
- Integrated ProfileEditor from experiments
- Added feature flag: FEATURE_NEW_PROFILE (defaults to true)
- Updated experiments/README.md with rollback instructions

Validation: ✅ TypeScript check passed (0 errors)
Build: ✅ Next.js build successful

Summary: Added profile editing with avatar upload. Integrated safely into existing UserSettings with feature flag for easy rollback if needed.

Files modified: 2 existing files, 3 new experiment files, +280 lines
```

## Context and Constraints

- **Dual-mode operation**: Respect the mode (developer vs starter_kit) set in `.claude/.meeting-state.json`
- **Incremental development**: Build on existing code, don't regenerate from scratch
- **Meeting context**: Requirements come from live meeting transcription, may have ambiguity
- **MVP focus**: Implement working functionality, not production-ready systems
- **User testing**: Generated code should be immediately runnable for demo purposes
- **Questions in updates**: If updates contain questions, note them in comments or README
- **Safety first (Developer Mode)**: Default to experiments folder, minimize changes to core files
- **Real-time feedback**: This command may be triggered automatically every 20-60 seconds during meetings
- **Progressive enhancement**: Each invocation should add value incrementally, not redo previous work

## Error Handling

- If `.meeting-updates.jsonl` has malformed JSON, skip that line and continue
- If project structure is unclear, ask user for clarification via summary output
- If conflicting requirements detected, note in summary and ask for resolution
- If dependencies are missing, attempt auto-install (npm/pip), fallback to listing in summary
- If build/validation fails after retries, continue with next updates and report failures
- If mode is not set, default to `"starter_kit"` for new projects

## Dev Server Integration (Starter Kit Mode)

After successfully creating/updating a Next.js or React project:
1. Check if dev server is already running (look for process on port 3000/5173)
2. If not running and this is first update, start dev server in background:
   - Next.js: `npm run dev` (typically port 3000)
   - Vite/React: `npm run dev` (typically port 5173)
3. Include dev server URL in summary: "🌐 Preview available at http://localhost:3000"
4. Note: Dev server management (starting/stopping) is handled by the MeetingCoder app, not this command

## Integration with MeetingCoder

This command template is designed to work with the MeetingCoder app, which:
- Writes updates to `.meeting-updates.jsonl` every 20-60 seconds during live meetings
- May trigger this command automatically via AppleScript/terminal automation
- Tracks meeting state and manages GitHub integration
- Provides real-time transcription context in `.transcript.jsonl`

For manual invocation: Simply run `/meeting` in a terminal within the project directory.
