# PRD Generator System - Design Document

## Overview

The PRD (Product Requirements Document) Generator is a new autonomous agentic feature that transforms meeting transcripts into structured, versioned product requirements documents. It operates continuously during meetings, creating and refining PRDs as requirements emerge and evolve.

**Key Value**: Automatically generate professional PRDs from natural conversation, eliminating manual documentation work and ensuring requirements are captured in real-time.

---

## Architecture

### High-Level Flow

```
Meeting Start
    ↓
Transcription (existing)
    ↓
Feature Extraction (existing - LLM/heuristic)
    ↓
.meeting-updates.jsonl (existing)
    ↓
┌─────────────────────────────────────┐
│  NEW: PRD Generator                 │
│  - Analyzes accumulated context     │
│  - Generates structured PRD         │
│  - Versions PRD over time           │
│  - Tracks requirement evolution     │
└─────────────────────────────────────┘
    ↓
meetings/{id}/prds/
├── v1_initial.md          (5-10 min mark)
├── v2_expanded.md         (20-30 min mark)
├── v3_refined.md          (40-50 min mark)
├── final.md               (meeting end)
└── changelog.json         (tracks changes)
```

### Integration Points

1. **Input Sources**:
   - `.meeting-updates.jsonl` - Structured feature extractions
   - `transcript.jsonl` - Raw conversation context
   - `.claude/.meeting-state.json` - Project metadata
   - Existing codebase analysis (if Developer Mode)

2. **Output Locations**:
   - `meetings/{meeting_id}/prds/` - Versioned PRD markdown files
   - `meetings/{meeting_id}/prds/changelog.json` - Change tracking
   - `meetings/{meeting_id}/prds/metadata.json` - PRD metadata

3. **Triggers**:
   - **Initial PRD** (v1): After 5-10 minutes or 15+ segments
   - **Incremental Updates**: Every 15-20 minutes
   - **Major Milestone Updates**: When significant new features detected
   - **Final PRD**: At meeting end

---

## Data Structures

### PRDVersion (Rust)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRDVersion {
    /// Version number (1, 2, 3, ...)
    pub version: u32,

    /// Timestamp when this version was generated
    pub generated_at: String,

    /// Meeting segment range covered (start, end)
    pub segment_range: (usize, usize),

    /// Total segments at time of generation
    pub total_segments: usize,

    /// Path to the markdown file
    pub file_path: String,

    /// Version type: "initial", "incremental", "milestone", "final"
    pub version_type: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Word count
    pub word_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRDContent {
    /// Executive summary
    pub executive_summary: String,

    /// User stories
    pub user_stories: Vec<UserStory>,

    /// Functional requirements
    pub functional_requirements: Vec<Requirement>,

    /// Non-functional requirements
    pub non_functional_requirements: Vec<Requirement>,

    /// Technical requirements
    pub technical_requirements: Vec<TechnicalRequirement>,

    /// Acceptance criteria
    pub acceptance_criteria: Vec<AcceptanceCriterion>,

    /// Dependencies
    pub dependencies: Vec<Dependency>,

    /// Risks
    pub risks: Vec<Risk>,

    /// Timeline milestones
    pub timeline: Vec<Milestone>,

    /// Open questions
    pub open_questions: Vec<Question>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStory {
    pub id: String,
    pub persona: String,  // "As a [persona]"
    pub want: String,     // "I want to [action]"
    pub so_that: String,  // "So that [benefit]"
    pub priority: String, // "high", "medium", "low"
    pub mentioned_at: Vec<usize>, // Transcript segment IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub status: String, // "planned", "discussed", "in_progress", "completed"
    pub mentioned_at: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalRequirement {
    pub id: String,
    pub category: String, // "framework", "library", "api", "infrastructure"
    pub description: String,
    pub rationale: String,
    pub alternatives_considered: Vec<String>,
    pub mentioned_at: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub requirement_id: String,
    pub description: String,
    pub testable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: String,
    pub name: String,
    pub type_: String, // "internal", "external", "third_party"
    pub description: String,
    pub blocking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: String,
    pub description: String,
    pub severity: String, // "high", "medium", "low"
    pub likelihood: String, // "high", "medium", "low"
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub title: String,
    pub description: String,
    pub target_date: Option<String>,
    pub deliverables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub question: String,
    pub context: String,
    pub asked_at: usize, // Segment ID
    pub resolved: bool,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRDChangelog {
    pub changes: Vec<PRDChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRDChange {
    pub from_version: u32,
    pub to_version: u32,
    pub timestamp: String,
    pub added_user_stories: Vec<String>,
    pub modified_user_stories: Vec<String>,
    pub removed_user_stories: Vec<String>,
    pub added_requirements: Vec<String>,
    pub modified_requirements: Vec<String>,
    pub removed_requirements: Vec<String>,
    pub resolved_questions: Vec<String>,
    pub new_questions: Vec<String>,
}
```

---

## Markdown Template Structure

### PRD Format

```markdown
# Product Requirements Document
**Project**: {project_name}
**Meeting**: {meeting_name}
**Version**: {version_number}
**Generated**: {timestamp}
**Segment Range**: {start_segment}-{end_segment} of {total_segments}

---

## Executive Summary

{1-2 paragraph overview of the project, key goals, and scope}

---

## User Stories

### Priority: High

**US-001**: As a {persona}, I want to {action}, so that {benefit}
- **Status**: {planned|in_progress|completed}
- **Mentioned**: Segment #{segment_id}
- **Dependencies**: {dep_ids}

### Priority: Medium

{...}

### Priority: Low

{...}

---

## Functional Requirements

### FR-001: {Title}
**Priority**: High
**Status**: Planned
**Description**: {description}

**Acceptance Criteria**:
- [ ] {criterion_1}
- [ ] {criterion_2}

**Mentioned**: Segment #{segment_id}

---

## Non-Functional Requirements

### NFR-001: {Title}
**Category**: Performance | Security | Scalability | Usability
**Priority**: {priority}
**Description**: {description}

---

## Technical Requirements

### Tech Stack

**Framework**: {framework}
**Rationale**: {why this was chosen}
**Alternatives Considered**: {alternatives}
**Mentioned**: Segment #{segment_id}

### Infrastructure

{...}

### APIs & Integrations

{...}

---

## Dependencies

### Internal Dependencies
- **DEP-001**: {description} (Blocking: Yes/No)

### External Dependencies
- **DEP-002**: {description}

### Third-Party Services
- **DEP-003**: {service_name} - {description}

---

## Risks & Mitigations

| Risk ID | Description | Severity | Likelihood | Mitigation |
|---------|-------------|----------|------------|------------|
| RISK-001 | {description} | High | Medium | {mitigation_strategy} |

---

## Timeline & Milestones

### Phase 1: {Title} (Week 1-2)
**Deliverables**:
- {deliverable_1}
- {deliverable_2}

### Phase 2: {Title} (Week 3-4)
{...}

---

## Open Questions

### Q-001: {Question text}
**Context**: {context}
**Asked**: Segment #{segment_id}
**Status**: Unresolved | Resolved
**Resolution**: {answer if resolved}

---

## Changelog

### Changes from v{prev_version}
- ✅ Added: {count} user stories, {count} requirements
- 🔄 Modified: {count} user stories, {count} requirements
- ❌ Removed: {count} items
- ✔️ Resolved: {count} questions
- ❓ New: {count} questions

---

## Appendix

### Traceability Matrix
Links requirements to transcript segments and code files (if Developer Mode)

### Meeting Context
- **Duration**: {duration}
- **Participants**: {speaker_list}
- **Project Type**: {web_app|mobile_app|api_backend}
- **Mode**: Developer | Starter Kit
```

---

## LLM Prompts for PRD Generation

### System Prompt

```
You are an expert product manager and technical writer. Your role is to analyze meeting transcripts and feature extractions to create comprehensive, professional Product Requirements Documents (PRDs).

Guidelines:
1. Extract clear, actionable user stories in "As a [persona], I want [action], so that [benefit]" format
2. Distinguish between functional (what the system does) and non-functional (how it performs) requirements
3. Capture technical decisions with rationale and alternatives considered
4. Write acceptance criteria that are specific, measurable, and testable
5. Identify dependencies, risks, and open questions
6. Link all items back to specific transcript segments for traceability
7. Prioritize based on language cues: "must", "critical" → high; "should", "nice to have" → medium/low
8. Maintain consistency across versions (preserve IDs, track changes)
9. Use professional but clear language, avoiding jargon where possible
10. Ensure the PRD is actionable: a developer should be able to implement from it

Output Format: JSON structure matching PRDContent schema
```

### User Prompt (Initial PRD - v1)

```
Generate an initial Product Requirements Document based on the following meeting context:

**Meeting Information**:
- Name: {meeting_name}
- Duration so far: {duration_minutes} minutes
- Segments covered: {segment_range}

**Project Context** (if available):
- Type: {project_type}
- Mode: {developer|starter_kit}
- Framework: {framework}
- Tech Stack: {tech_stack}

**Transcript Segments**:
{transcript_text}

**Extracted Features** (from .meeting-updates.jsonl):
{feature_extractions_json}

Please create a comprehensive PRD with:
1. Executive summary (2-3 sentences)
2. User stories (at least 3-5)
3. Functional requirements (at least 5-10)
4. Non-functional requirements (if mentioned)
5. Technical requirements (frameworks, libraries, architecture)
6. Acceptance criteria for each user story
7. Dependencies (if mentioned)
8. Risks (if mentioned)
9. Timeline milestones (if dates/phases mentioned)
10. Open questions (uncertainties or ambiguities)

Return only valid JSON matching the PRDContent schema.
```

### User Prompt (Incremental Update - v2+)

```
Update the existing Product Requirements Document with new information from the meeting.

**Previous PRD**:
{previous_prd_json}

**New Transcript Segments** (since last update):
{new_transcript_text}

**New Feature Extractions**:
{new_feature_extractions_json}

**Changes to Make**:
1. Add any NEW user stories, requirements, or technical decisions
2. MODIFY existing items if new information clarifies or changes them
3. RESOLVE open questions if answers were provided
4. Add NEW open questions if ambiguities arose
5. Update priorities if emphasis changed
6. Add risks or dependencies if mentioned
7. Maintain ALL item IDs from previous version (do not renumber)
8. For new items, use next sequential ID

Return:
1. Complete updated PRDContent JSON
2. PRDChange object documenting what changed

Return as JSON: { "content": PRDContent, "change": PRDChange }
```

---

## Implementation Plan

### Module Structure

```
src-tauri/src/
├── document_generation/
│   ├── mod.rs
│   ├── prd_generator.rs       # Main PRD generation logic
│   ├── prd_template.rs        # Markdown template rendering
│   ├── prd_storage.rs         # File I/O and versioning
│   ├── prd_analyzer.rs        # Diff and change detection
│   └── types.rs               # Data structures
├── commands/
│   └── prd.rs                 # Tauri commands
└── managers/
    └── meeting.rs             # Integration point (modify)
```

### Key Functions

```rust
// document_generation/prd_generator.rs

pub struct PRDGenerator {
    meeting_id: String,
    meeting_name: String,
    project_type: Option<String>,
    versions: Vec<PRDVersion>,
}

impl PRDGenerator {
    /// Create a new PRD generator for a meeting
    pub fn new(meeting_id: String, meeting_name: String) -> Self;

    /// Determine if it's time to generate a new PRD version
    pub fn should_generate_version(
        &self,
        total_segments: usize,
        time_since_last: Duration,
    ) -> bool;

    /// Generate initial PRD (v1)
    pub async fn generate_initial_prd(
        &mut self,
        transcript: &[TranscriptSegment],
        feature_extractions: &[SummarizationOutput],
        project_context: Option<&CodebaseInfo>,
    ) -> Result<PRDVersion>;

    /// Generate incremental update (v2+)
    pub async fn generate_incremental_update(
        &mut self,
        new_transcript: &[TranscriptSegment],
        new_extractions: &[SummarizationOutput],
    ) -> Result<PRDVersion>;

    /// Generate final PRD at meeting end
    pub async fn generate_final_prd(
        &mut self,
        transcript: &[TranscriptSegment],
        feature_extractions: &[SummarizationOutput],
        meeting_summary: &MeetingSummary,
    ) -> Result<PRDVersion>;

    /// Get latest PRD version
    pub fn get_latest_version(&self) -> Option<&PRDVersion>;

    /// Get all versions
    pub fn get_all_versions(&self) -> &[PRDVersion];

    /// Get changelog between versions
    pub fn get_changelog(&self, from: u32, to: u32) -> Result<PRDChange>;
}

// document_generation/prd_template.rs

/// Render PRDContent to markdown string
pub fn render_prd_markdown(
    content: &PRDContent,
    version: &PRDVersion,
    meeting_name: &str,
    project_name: &str,
) -> String;

/// Render changelog to markdown
pub fn render_changelog_markdown(changes: &[PRDChange]) -> String;

// document_generation/prd_storage.rs

/// Save PRD version to disk
pub fn save_prd_version(
    meeting_id: &str,
    version: &PRDVersion,
    content: &str,
) -> Result<String>;

/// Load PRD version from disk
pub fn load_prd_version(meeting_id: &str, version: u32) -> Result<(PRDVersion, String)>;

/// Save changelog
pub fn save_changelog(meeting_id: &str, changelog: &PRDChangelog) -> Result<()>;

/// Load changelog
pub fn load_changelog(meeting_id: &str) -> Result<PRDChangelog>;
```

### Tauri Commands

```rust
// commands/prd.rs

#[tauri::command]
pub async fn generate_prd_now(meeting_id: String) -> Result<PRDVersion, String>;

#[tauri::command]
pub async fn get_prd_versions(meeting_id: String) -> Result<Vec<PRDVersion>, String>;

#[tauri::command]
pub async fn get_prd_content(meeting_id: String, version: u32) -> Result<String, String>;

#[tauri::command]
pub async fn get_prd_changelog(meeting_id: String) -> Result<PRDChangelog, String>;

#[tauri::command]
pub async fn export_prd(meeting_id: String, version: u32, format: String) -> Result<String, String>;
// Formats: "markdown", "pdf", "html"
```

### Integration with MeetingManager

```rust
// In managers/meeting.rs

pub struct MeetingState {
    // ... existing fields ...

    /// PRD generator for this meeting
    prd_generator: Option<PRDGenerator>,
}

impl MeetingManager {
    pub fn start_meeting(&mut self, name: String) -> Result<String> {
        // ... existing logic ...

        // Initialize PRD generator
        let prd_gen = PRDGenerator::new(meeting_id.clone(), name.clone());

        // Store in state
        meeting_state.prd_generator = Some(prd_gen);

        // ... rest of logic ...
    }

    // In the update loop (called every 20 seconds)
    async fn process_meeting_updates(&mut self, meeting_id: &str) -> Result<()> {
        // ... existing summarization logic ...

        // Check if PRD should be generated
        if let Some(prd_gen) = meeting_state.prd_generator.as_mut() {
            if prd_gen.should_generate_version(total_segments, time_since_last) {
                // Generate PRD version
                let version = prd_gen.generate_incremental_update(
                    new_segments,
                    new_extractions,
                ).await?;

                // Emit event to frontend
                self.emit_event("prd-version-generated", &version)?;

                log::info!("Generated PRD v{} for meeting {}", version.version, meeting_id);
            }
        }

        // ... rest of logic ...
    }
}
```

---

## Settings & Configuration

### New Settings

```rust
// In settings.rs

pub struct Settings {
    // ... existing settings ...

    /// Enable automatic PRD generation
    pub enable_prd_generation: bool, // Default: true

    /// Minimum segments before initial PRD
    pub prd_initial_min_segments: usize, // Default: 15

    /// Time between PRD updates (minutes)
    pub prd_update_interval_minutes: u64, // Default: 15

    /// Include traceability matrix in PRD
    pub prd_include_traceability: bool, // Default: true

    /// Auto-export PRD on meeting end
    pub prd_auto_export: bool, // Default: true
}
```

### Frontend Controls

```typescript
// src/components/settings/PRDSettings.tsx

<SettingToggle
  label="Enable PRD Generation"
  description="Automatically generate Product Requirements Documents during meetings"
  setting="enable_prd_generation"
/>

<SettingSlider
  label="Initial PRD Threshold"
  description="Minimum segments before generating initial PRD"
  setting="prd_initial_min_segments"
  min={10}
  max={30}
  step={5}
/>

<SettingSlider
  label="Update Interval"
  description="Minutes between PRD updates"
  setting="prd_update_interval_minutes"
  min={10}
  max={30}
  step={5}
/>
```

---

## Frontend Components

### PRD View Component

```typescript
// src/components/meeting/PRDView.tsx

interface PRDViewProps {
  meetingId: string;
}

export function PRDView({ meetingId }: PRDViewProps) {
  const [versions, setVersions] = useState<PRDVersion[]>([]);
  const [selectedVersion, setSelectedVersion] = useState<number | null>(null);
  const [prdContent, setPrdContent] = useState<string>('');
  const [changelog, setChangelog] = useState<PRDChangelog | null>(null);

  // Load versions on mount
  useEffect(() => {
    invoke('get_prd_versions', { meetingId }).then(setVersions);
    invoke('get_prd_changelog', { meetingId }).then(setChangelog);
  }, [meetingId]);

  // Load content when version selected
  useEffect(() => {
    if (selectedVersion) {
      invoke('get_prd_content', { meetingId, version: selectedVersion })
        .then(setPrdContent);
    }
  }, [selectedVersion]);

  return (
    <div className="prd-view">
      <div className="prd-sidebar">
        <h3>PRD Versions</h3>
        {versions.map(v => (
          <div
            key={v.version}
            className={`version-item ${selectedVersion === v.version ? 'active' : ''}`}
            onClick={() => setSelectedVersion(v.version)}
          >
            <div className="version-badge">v{v.version}</div>
            <div className="version-info">
              <div className="version-type">{v.version_type}</div>
              <div className="version-time">{formatTime(v.generated_at)}</div>
              <div className="version-segments">
                Segments {v.segment_range[0]}-{v.segment_range[1]}
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="prd-content">
        {selectedVersion ? (
          <>
            <div className="prd-toolbar">
              <button onClick={() => exportPRD('markdown')}>
                Export Markdown
              </button>
              <button onClick={() => exportPRD('pdf')}>
                Export PDF
              </button>
              <button onClick={() => viewDiff()}>
                View Changes
              </button>
            </div>

            <div className="markdown-viewer">
              <ReactMarkdown>{prdContent}</ReactMarkdown>
            </div>
          </>
        ) : (
          <div className="empty-state">
            Select a PRD version to view
          </div>
        )}
      </div>

      <div className="prd-changelog">
        <h3>Changelog</h3>
        {changelog && renderChangelog(changelog)}
      </div>
    </div>
  );
}
```

### PRD Diff Viewer

```typescript
// src/components/meeting/PRDDiffView.tsx

export function PRDDiffView({ meetingId, fromVersion, toVersion }) {
  const [change, setChange] = useState<PRDChange | null>(null);

  useEffect(() => {
    // Load changelog entry
    invoke('get_prd_changelog', { meetingId }).then(changelog => {
      const change = changelog.changes.find(
        c => c.from_version === fromVersion && c.to_version === toVersion
      );
      setChange(change);
    });
  }, [fromVersion, toVersion]);

  return (
    <div className="prd-diff">
      <h3>Changes from v{fromVersion} to v{toVersion}</h3>

      {change && (
        <>
          <div className="diff-section">
            <h4>✅ Added User Stories ({change.added_user_stories.length})</h4>
            <ul>
              {change.added_user_stories.map(id => <li key={id}>{id}</li>)}
            </ul>
          </div>

          <div className="diff-section">
            <h4>🔄 Modified User Stories ({change.modified_user_stories.length})</h4>
            <ul>
              {change.modified_user_stories.map(id => <li key={id}>{id}</li>)}
            </ul>
          </div>

          <div className="diff-section">
            <h4>✔️ Resolved Questions ({change.resolved_questions.length})</h4>
            <ul>
              {change.resolved_questions.map(id => <li key={id}>{id}</li>)}
            </ul>
          </div>

          {/* Similar sections for requirements, questions, etc. */}
        </>
      )}
    </div>
  );
}
```

---

## File System Layout

```
~/.handy/meetings/{meeting_id}/
├── transcript.jsonl
├── metadata.json
├── .meeting-updates.jsonl
├── prds/
│   ├── metadata.json          # PRD generation metadata
│   ├── changelog.json         # Complete changelog
│   ├── v1_initial.md          # First PRD (5-10 min mark)
│   ├── v1_initial.json        # Structured data
│   ├── v2_expanded.md         # Second update (25 min mark)
│   ├── v2_expanded.json
│   ├── v3_refined.md          # Third update (45 min mark)
│   ├── v3_refined.json
│   ├── final.md               # Final PRD (meeting end)
│   └── final.json
└── .claude/
    └── .meeting-state.json
```

### metadata.json

```json
{
  "meeting_id": "uuid",
  "meeting_name": "Feature Planning Session",
  "total_versions": 4,
  "latest_version": 4,
  "first_generated_at": "2025-11-11T12:05:00Z",
  "last_updated_at": "2025-11-11T13:15:00Z"
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prd_generator_initialization() {
        let gen = PRDGenerator::new("test-id".to_string(), "Test Meeting".to_string());
        assert_eq!(gen.versions.len(), 0);
    }

    #[test]
    fn test_should_generate_version() {
        // Test threshold logic
        let mut gen = PRDGenerator::new("test-id".to_string(), "Test".to_string());

        // Should generate initial after 15 segments
        assert!(gen.should_generate_version(15, Duration::from_secs(600)));

        // Should not generate again immediately
        assert!(!gen.should_generate_version(16, Duration::from_secs(10)));
    }

    #[tokio::test]
    async fn test_generate_initial_prd() {
        // Mock transcript and extractions
        let transcript = vec![/* ... */];
        let extractions = vec![/* ... */];

        let mut gen = PRDGenerator::new("test-id".to_string(), "Test".to_string());
        let version = gen.generate_initial_prd(&transcript, &extractions, None).await.unwrap();

        assert_eq!(version.version, 1);
        assert_eq!(version.version_type, "initial");
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_prd_lifecycle() {
    // Simulate a full meeting with multiple PRD generations
    // 1. Start meeting
    // 2. Add segments
    // 3. Generate initial PRD
    // 4. Add more segments
    // 5. Generate update
    // 6. End meeting
    // 7. Generate final PRD
    // 8. Verify changelog
}
```

---

## Performance Considerations

### LLM API Costs

- **Initial PRD**: ~2000 tokens input, ~1500 tokens output (~$0.10)
- **Incremental Updates**: ~1000 tokens input, ~800 tokens output (~$0.05)
- **Per Meeting**: ~3-4 generations = ~$0.25-$0.40

**Optimization**:
- Only generate when significant new information added
- Cache previous PRD structure to reduce context size
- Use Claude Sonnet (good balance of cost/quality)

### Storage

- **Per PRD Version**: ~10-20 KB markdown + ~5-10 KB JSON
- **Per Meeting**: ~100-200 KB total for 3-4 versions
- **Manageable**: Even 1000 meetings = ~100-200 MB

### Processing Time

- **LLM API Call**: ~3-5 seconds
- **Markdown Rendering**: <100ms
- **File I/O**: <50ms
- **Total per generation**: ~5 seconds (non-blocking)

---

## Future Enhancements

1. **Export Formats**:
   - PDF generation with styling
   - HTML with interactive navigation
   - Confluence/Notion API integration

2. **Collaboration Features**:
   - Share PRD URL with team
   - Inline comments and annotations
   - Approval workflow

3. **Requirements Coverage**:
   - Map requirements to code files
   - Track implementation status
   - Highlight gaps

4. **Multi-Meeting PRDs**:
   - Aggregate PRDs across related meetings
   - Track requirement evolution over sprints

5. **AI-Powered Suggestions**:
   - Detect missing acceptance criteria
   - Suggest related requirements
   - Identify conflicting requirements

---

## Acceptance Criteria

PRD Generator is **complete** when:

1. ✅ Initial PRD generated after 15+ segments
2. ✅ Incremental updates every 15 minutes
3. ✅ Final PRD generated at meeting end
4. ✅ All PRD versions stored and accessible
5. ✅ Changelog tracks all changes between versions
6. ✅ Frontend displays PRD with version selector
7. ✅ Export to markdown and PDF works
8. ✅ PRD linked to transcript segments
9. ✅ Settings allow customization
10. ✅ Works in both Developer and Starter Kit modes

---

## References

- Existing LLM Integration: `src-tauri/src/summarization/llm.rs`
- Meeting Manager: `src-tauri/src/managers/meeting.rs`
- Feature Extraction: `src-tauri/src/summarization/agent.rs`
- Phase 7 PRD: `docs/prd/07-PHASE7.md`
