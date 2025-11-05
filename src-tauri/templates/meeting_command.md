Read new updates from .meeting-updates.jsonl and update the codebase accordingly.

## Steps

1. **Check for updates file**
   - Look for `.meeting-updates.jsonl` in the current working directory
   - If not found, respond: "No .meeting-updates.jsonl file found. Is this a MeetingCoder project?"

2. **Load state**
   - Read `.claude/.meeting-state.json` to get `last_processed_update_id` (string like `"u12"`)
   - If file doesn't exist, initialize it with `{"last_processed_update_id": null, "last_processed_timestamp": null, "total_updates_processed": 0}`

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

4. **Process first update (new project initialization)**
   - If this is the FIRST update (`last_processed_update_id` is null):
     - Extract `project_name`, `project_type`, and `tech_stack` from the update
     - Create appropriate project structure based on `project_type`:
       - `react_web_app`: Vite + React + TypeScript + TailwindCSS
       - `node_api`: Express + TypeScript + basic middleware
       - `python_cli`: Click + proper package structure
       - `full_stack_app`: React frontend + Node.js backend in monorepo
       - `static_website`: HTML + CSS + vanilla JS
     - Implement HIGH priority features from the first batch
     - Create initial README with project setup instructions
     - Add package.json/requirements.txt with dependencies

5. **Process subsequent updates (incremental changes)**
   - For each new update after the first:
     - **New features**: Prefer `new_features_structured` to drive concrete work. Use `priority` to order work (`high` first), and `description` for acceptance
       - If `new_features_structured` is absent, fall back to legacy `new_features: string[]`
     - **Modified features**: Update existing code to match changes
     - **Clarifications**: Refine implementations based on answered questions (map keyed by `feature_id`)
     - **Technical decisions**: Apply architectural or tech stack changes

6. **Code generation guidelines**
   - Maintain consistency with existing code style
   - Add comments explaining key logic and non-obvious decisions
   - Include error handling and input validation
   - Follow best practices for the tech stack
   - Implement MVP functionality (don't over-engineer)
   - Use modern patterns and libraries
   - Ensure generated code is syntactically valid

7. **Update state**
   - After processing all new updates, update `.claude/.meeting-state.json`:
     ```json
     {
       "last_processed_update_id": "<latest_update_id>",
       "last_processed_timestamp": "<current_timestamp>",
       "total_updates_processed": <count>
     }
     ```

8. **Provide summary**
   - List all changes made in this batch
   - Format: "Update {update_id}: {feature_title}"
   - Include files created, modified, or deleted
   - Provide a brief overall summary of progress

## Example Workflow

**First invocation (u1 - new project):**
```
Reading .meeting-updates.jsonl...
Found first update: Initializing customer-feedback-app

Project type: React Web App
Tech stack: React, TypeScript, Node.js, PostgreSQL

Creating project structure...
- ✅ Created package.json with dependencies
- ✅ Created vite.config.ts
- ✅ Created src/App.tsx (main component)
- ✅ Created src/main.tsx (entry point)
- ✅ Created components/LoginForm.tsx
- ✅ Created components/FeedbackForm.tsx
- ✅ Created README.md with setup instructions

Implemented 2 high-priority features:
  1. User authentication (email/password login)
  2. Feedback submission form

Next steps:
- Run `npm install` to install dependencies
- Run `npm run dev` to start development server
```

**Subsequent invocation (u2-u3 - incremental updates):**
```
Reading .meeting-updates.jsonl...
Found 2 new updates since last check (u2, u3)

Update u2: CSV File Upload
- Created components/FileUpload.tsx
- Added file input with validation
- Updated App.tsx to include FileUpload component

Update u3: Column Validation with Error Display
- Enhanced FileUpload with column validation logic
- Created components/ValidationErrors.tsx for error display
- Added error state management to FileUpload

Summary: Added CSV upload functionality with column validation. Users can now upload CSV files and see specific errors for invalid columns.
```

## Context and Constraints

- **Incremental development**: Build on existing code, don't regenerate from scratch
- **Meeting context**: Requirements come from live meeting transcription, may have ambiguity
- **MVP focus**: Implement working functionality, not production-ready systems
- **User testing**: Generated code should be immediately runnable for demo purposes
- **Questions in updates**: If updates contain questions, note them in comments or README

## Error Handling

- If `.meeting-updates.jsonl` has malformed JSON, skip that line and continue
- If project structure is unclear, ask user for clarification
- If conflicting requirements detected, note in summary and ask for resolution
- If dependencies are missing, list them clearly in the summary
