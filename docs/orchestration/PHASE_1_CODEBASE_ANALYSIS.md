# Phase 1: Codebase Analysis System

## Overview

Build an intelligent codebase scanner that understands project structure, architecture patterns, existing features, and conventions. This provides the foundational context needed for intelligent task generation and code orchestration.

## Goals

1. **Automatic Discovery** - Scan any codebase and identify key characteristics
2. **Architecture Understanding** - Detect patterns (MVC, Component-Based, etc.)
3. **Feature Mapping** - Catalog existing functionality
4. **Convention Extraction** - Learn naming and structural patterns
5. **Integration Points** - Identify where new code should be added

## Success Criteria

- ✅ Correctly identifies tech stack for React, Rust, Python, and Node.js projects
- ✅ Detects common architecture patterns with 80%+ accuracy
- ✅ Maps 90%+ of major features in a codebase
- ✅ Extracts naming conventions automatically
- ✅ Identifies all major integration points (routes, commands, etc.)
- ✅ Analysis completes in < 30 seconds for medium-sized projects (10k files)
- ✅ UI displays analysis results in readable format

## User Stories

### US-1: Automatic Analysis on Meeting Start
**As a** developer running a meeting
**I want** the codebase to be analyzed automatically when I start coding a feature
**So that** the system understands the project context without manual setup

**Acceptance Criteria:**
- Analysis triggers when meeting project path is set
- Results are cached for the meeting duration
- User sees analysis progress notification
- Analysis runs in background without blocking

### US-2: Tech Stack Identification
**As a** developer
**I want** the system to identify my tech stack automatically
**So that** generated code uses the right frameworks and libraries

**Acceptance Criteria:**
- Detects language (TypeScript, Python, Rust, etc.)
- Identifies frameworks (React, Tauri, Express, Django, etc.)
- Recognizes UI libraries (Tailwind, MUI, Bootstrap)
- Detects state management (Zustand, Redux, Context)
- Identifies build tools (Vite, Webpack, Rollup)

### US-3: Architecture Pattern Detection
**As a** developer
**I want** the system to understand my project's architecture
**So that** new code follows existing structural patterns

**Acceptance Criteria:**
- Detects MVC, MVVM, Component-Based, Feature-Based patterns
- Identifies layering (presentation, business logic, data)
- Recognizes monolith vs microservices
- Maps directory structure to architectural concepts

### US-4: Feature Inventory
**As a** developer
**I want** the system to map existing features
**So that** new features don't duplicate existing functionality

**Acceptance Criteria:**
- Lists major features/capabilities
- Identifies related files for each feature
- Maps UI components to features
- Detects API endpoints and services

### US-5: Integration Point Discovery
**As a** developer
**I want** the system to identify where new code should be added
**So that** integrations follow project conventions

**Acceptance Criteria:**
- Finds where to register new routes
- Identifies command registration points (Tauri)
- Locates component registration patterns
- Discovers middleware/plugin hooks

### US-6: View Analysis Results
**As a** developer
**I want** to view the codebase analysis in the UI
**So that** I can verify the system understands my project correctly

**Acceptance Criteria:**
- Displays tech stack summary
- Shows architecture pattern
- Lists existing features
- Shows integration points with examples
- Allows manual re-analysis trigger

## Technical Requirements

### TR-1: Data Structures

**CodebaseContext:**
```rust
pub struct CodebaseContext {
    pub project_path: String,
    pub analyzed_at: String,
    pub tech_stack: TechStack,
    pub architecture_pattern: ArchitecturePattern,
    pub file_tree: FileTree,
    pub existing_features: Vec<Feature>,
    pub ui_components: Vec<Component>,
    pub naming_conventions: NamingConventions,
    pub integration_points: Vec<IntegrationPoint>,
    pub total_files: usize,
    pub languages: HashMap<String, LanguageStats>,
}
```

**Storage Format:** JSON at `~/.handy/meetings/{meeting_id}/codebase_context.json`

### TR-2: Scanner Implementation

**File:** `src-tauri/src/codebase_analysis/scanner.rs`

**Methods:**
- `analyze() -> Result<CodebaseContext>` - Main entry point
- `identify_tech_stack() -> Result<TechStack>` - Parse package.json, Cargo.toml
- `detect_architecture() -> Result<ArchitecturePattern>` - Analyze directory structure
- `map_features() -> Result<Vec<Feature>>` - Identify major features
- `extract_conventions() -> Result<NamingConventions>` - Learn patterns
- `identify_integration_points() -> Result<Vec<IntegrationPoint>>` - Find hooks

### TR-3: Tech Stack Detection

**Supported Stacks:**
- **Frontend:** React, Vue, Svelte, Angular
- **Backend:** Node.js/Express, Rust/Actix, Python/FastAPI, Django
- **Desktop:** Tauri, Electron
- **Mobile:** React Native, Flutter
- **Build:** Vite, Webpack, Rollup, Parcel
- **State:** Zustand, Redux, MobX, Recoil
- **Styling:** Tailwind, CSS Modules, Styled Components, MUI

**Detection Strategy:**
1. Read package.json dependencies (Node.js)
2. Read Cargo.toml dependencies (Rust)
3. Read requirements.txt/pyproject.toml (Python)
4. Check for framework-specific files (vite.config.ts, next.config.js)
5. Scan imports in key files

### TR-4: Architecture Pattern Detection

**Patterns to Detect:**
- **MVC** - models/, views/, controllers/
- **MVVM** - views/, viewmodels/, models/
- **Component-Based** - components/, pages/, hooks/
- **Feature-Based** - features/{feature}/components/
- **Layered** - presentation/, business/, data/
- **Monorepo** - apps/, packages/

**Detection Algorithm:**
1. Scan directory structure
2. Look for characteristic directories
3. Analyze file organization patterns
4. Check for documentation (README mentions)
5. Default to most likely pattern if ambiguous

### TR-5: Feature Mapping

**Feature Detection:**
- In React: Analyze components/ and pages/ directories
- In APIs: Parse route definitions
- In Services: Identify service classes/modules
- Use file/directory names as feature names
- Group related files together

**Feature Structure:**
```rust
pub struct Feature {
    pub id: String,
    pub name: String,
    pub description: String,
    pub related_files: Vec<String>,
    pub entry_points: Vec<String>,
    pub dependencies: Vec<String>,
}
```

### TR-6: Convention Extraction

**Naming Conventions:**
- File naming: Analyze file names to detect camelCase, PascalCase, kebab-case, snake_case
- Component naming: Extract from React component names
- Function naming: Parse function declarations
- Test file patterns: Detect .test.ts, .spec.ts, _test.py patterns

**Code Patterns:**
- Component structure (functional vs class)
- Import organization
- Export patterns (default vs named)
- Hook usage patterns

### TR-7: Integration Points

**Points to Identify:**
- **Tauri Commands:** Where .invoke_handler() is called in lib.rs
- **API Routes:** Express app.use(), FastAPI route decorators
- **React Routes:** Route components in App.tsx or router config
- **Component Registration:** Where components are exported/imported
- **State Providers:** Context providers, store creation
- **Middleware:** Where middleware is registered

### TR-8: Tauri Commands

**Commands to Implement:**
```rust
#[tauri::command]
async fn analyze_codebase(meeting_id: String) -> Result<CodebaseContext, String>

#[tauri::command]
async fn get_codebase_context(meeting_id: String) -> Result<CodebaseContext, String>

#[tauri::command]
async fn refresh_codebase_analysis(meeting_id: String) -> Result<CodebaseContext, String>
```

### TR-9: Caching Strategy

- Cache analysis results in memory during meeting
- Persist to disk at `~/.handy/meetings/{meeting_id}/codebase_context.json`
- Auto-refresh if files change (watch for git commits)
- Manual refresh button in UI
- Cache TTL: until meeting ends

### TR-10: Performance Requirements

- Initial analysis: < 30 seconds for 10k files
- Incremental updates: < 5 seconds
- Memory usage: < 100MB for analysis data
- File scanning: Use parallel processing with rayon
- Ignore patterns: node_modules, .git, dist, build, target

## File Structure

```
src-tauri/src/
├── codebase_analysis/
│   ├── mod.rs                    # Module exports
│   ├── types.rs                  # Data structures (300 lines)
│   ├── scanner.rs                # Main scanner (500 lines)
│   ├── tech_stack_detector.rs   # Tech stack identification (300 lines)
│   ├── architecture_detector.rs # Architecture pattern detection (300 lines)
│   ├── feature_mapper.rs         # Feature mapping (400 lines)
│   ├── convention_extractor.rs  # Naming/pattern extraction (300 lines)
│   └── integration_finder.rs    # Integration points (300 lines)
├── commands/
│   └── codebase.rs               # Tauri commands (150 lines)
└── lib.rs                        # Register commands

src/components/meeting/
├── CodebaseAnalysisView.tsx      # Main UI (300 lines)
└── AnalysisCard.tsx              # Reusable card component (100 lines)

src/lib/
└── types.ts                      # Add CodebaseContext types (200 lines)
```

## Implementation Steps

### Step 1: Data Structures (Day 1)
1. Create `types.rs` with all data structures
2. Implement Serialize/Deserialize
3. Add TypeScript types to frontend
4. Create storage utilities

### Step 2: Basic Scanner (Day 2)
1. Implement file tree scanning with ignore patterns
2. Add language detection (count file extensions)
3. Create basic CodebaseContext builder
4. Add storage save/load functions

### Step 3: Tech Stack Detection (Day 3)
1. Implement package.json parser
2. Implement Cargo.toml parser
3. Add Python requirements.txt parser
4. Create tech stack identification logic
5. Add framework detection

### Step 4: Architecture Detection (Day 4)
1. Implement directory structure analyzer
2. Add pattern matching for common architectures
3. Create scoring system for ambiguous cases
4. Add fallback to "custom" architecture

### Step 5: Feature Mapping (Day 5)
1. Implement component scanner (React)
2. Add API route parser (Express, FastAPI)
3. Create feature grouping algorithm
4. Add entry point detection

### Step 6: Convention & Integration Points (Day 6)
1. Implement naming convention extractor
2. Add code pattern analyzer
3. Create integration point finder
4. Add specific detectors (Tauri, React Router, etc.)

### Step 7: Tauri Commands & Events (Day 7)
1. Create Tauri commands in `commands/codebase.rs`
2. Register commands in lib.rs
3. Add event emission for analysis progress
4. Implement caching logic

### Step 8: Frontend UI (Day 8)
1. Create `CodebaseAnalysisView` component
2. Add cards for tech stack, architecture, features
3. Implement analysis trigger button
4. Add loading states and error handling

### Step 9: Integration & Testing (Day 9)
1. Integrate with meeting start flow
2. Add auto-analysis when project path is set
3. Test with sample React/Tauri projects
4. Test with sample Node.js/Python projects

### Step 10: Polish & Documentation (Day 10)
1. Add logging throughout
2. Optimize performance (parallel scanning)
3. Write integration tests
4. Document API in this PRD

## Testing Requirements

### Unit Tests
- Tech stack detection for different project types
- Architecture pattern detection
- Feature name extraction
- Naming convention detection
- Integration point finding

### Integration Tests
- Full analysis of sample React project
- Full analysis of sample Rust project
- Full analysis of Tauri app (MeetingCoder itself!)
- Caching and retrieval
- Frontend command invocation

### Test Projects
Create minimal test projects:
- `tests/fixtures/react-app/` - React + TypeScript + Vite
- `tests/fixtures/rust-api/` - Actix web API
- `tests/fixtures/tauri-app/` - Tauri desktop app
- `tests/fixtures/python-api/` - FastAPI application

## Error Handling

**Scenarios:**
1. **Invalid Project Path** - Return error, prompt user to select valid path
2. **Permission Denied** - Log error, skip restricted files
3. **Large Project** - Show progress, allow cancellation
4. **Parsing Failures** - Log warning, continue with partial analysis
5. **Unknown Tech Stack** - Mark as "custom", provide manual override

## UI/UX Considerations

**Analysis Trigger:**
- Auto-trigger on meeting start if project path exists
- Manual "Analyze Codebase" button in UI
- Show progress notification during analysis
- Cache results to avoid re-analysis

**Display:**
- Clean card-based layout
- Collapsible sections for detailed info
- Color-coding for tech stack badges
- Icons for different components types
- Copy-to-clipboard for file paths

**Feedback:**
- Toast notifications for analysis start/complete/error
- Progress bar for long analyses
- Real-time file count updates
- "Last analyzed" timestamp

## Dependencies

**Rust Crates:**
- `serde` & `serde_json` - Serialization
- `tokio` - Async runtime
- `rayon` - Parallel processing
- `walkdir` or `ignore` - File tree traversal
- `toml` - Cargo.toml parsing
- `regex` - Pattern matching

**Frontend:**
- Existing React/TypeScript setup
- Existing Tauri API integration
- Icons from lucide-react

## Future Enhancements (Phase 5)

- LLM-powered deep analysis for complex patterns
- Automatic documentation generation
- Dependency graph visualization
- Code quality metrics
- Security vulnerability scanning
- Performance bottleneck detection

## Acceptance Testing

**Test Case 1: Analyze MeetingCoder Itself**
```bash
1. Start MeetingCoder
2. Create new meeting with project path = MeetingCoder root
3. Navigate to "Codebase Analysis" tab
4. Click "Analyze Codebase"
5. Verify results:
   - Tech Stack: TypeScript, React, Tauri, Rust
   - Architecture: Component-Based
   - Features: Meeting, Transcription, PRD, Settings, Audio
   - Integration Points: Tauri commands in lib.rs, Routes in App.tsx
```

**Test Case 2: Analyze External React Project**
```bash
1. Clone sample React project
2. Set as meeting project path
3. Trigger analysis
4. Verify correct tech stack detection
5. Verify features are mapped
6. Verify integration points found
```

## Rollout Plan

1. **Dev:** Implement and test with MeetingCoder itself
2. **Alpha:** Test with 3-5 different project types
3. **Beta:** Release to internal users
4. **GA:** Full release with Phase 2

## Questions for Clarification

1. Should analysis be automatic or require explicit trigger?
2. How deep should feature mapping go? (Only top-level or nested features?)
3. Should we analyze node_modules for dependency insights?
4. What's the priority order for tech stacks to support?
5. Should codebase context be versioned/tracked over time?
