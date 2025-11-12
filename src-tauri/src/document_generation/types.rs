use serde::{Deserialize, Serialize};

/// A version of a PRD document
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

/// Complete PRD content structure
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

impl Default for PRDContent {
    fn default() -> Self {
        Self {
            executive_summary: String::new(),
            user_stories: Vec::new(),
            functional_requirements: Vec::new(),
            non_functional_requirements: Vec::new(),
            technical_requirements: Vec::new(),
            acceptance_criteria: Vec::new(),
            dependencies: Vec::new(),
            risks: Vec::new(),
            timeline: Vec::new(),
            open_questions: Vec::new(),
        }
    }
}

/// User story in standard format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStory {
    pub id: String,
    pub persona: String,  // "As a [persona]"
    pub want: String,     // "I want to [action]"
    pub so_that: String,  // "So that [benefit]"
    pub priority: String, // "high", "medium", "low"
    pub status: String,   // "planned", "in_progress", "completed"
    pub mentioned_at: Vec<usize>, // Transcript segment IDs
}

/// Functional or non-functional requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: String, // "high", "medium", "low"
    pub status: String,   // "planned", "discussed", "in_progress", "completed"
    pub category: Option<String>, // For NFRs: "performance", "security", "scalability", "usability"
    pub mentioned_at: Vec<usize>,
}

/// Technical requirement with rationale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalRequirement {
    pub id: String,
    pub category: String, // "framework", "library", "api", "infrastructure"
    pub title: String,
    pub description: String,
    pub rationale: String,
    pub alternatives_considered: Vec<String>,
    pub mentioned_at: Vec<usize>,
}

/// Acceptance criterion for a requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub requirement_id: String,
    pub description: String,
    pub testable: bool,
}

/// Project dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String, // "internal", "external", "third_party"
    pub description: String,
    pub blocking: bool,
}

/// Risk with mitigation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: String,
    pub description: String,
    pub severity: String,   // "high", "medium", "low"
    pub likelihood: String, // "high", "medium", "low"
    pub mitigation: String,
}

/// Timeline milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub title: String,
    pub description: String,
    pub target_date: Option<String>,
    pub deliverables: Vec<String>,
}

/// Open question that needs resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub question: String,
    pub context: String,
    pub asked_at: usize, // Segment ID
    pub resolved: bool,
    pub resolution: Option<String>,
}

/// Changelog tracking changes between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRDChangelog {
    pub changes: Vec<PRDChange>,
}

impl Default for PRDChangelog {
    fn default() -> Self {
        Self {
            changes: Vec::new(),
        }
    }
}

/// A single change record between two versions
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
    pub added_technical_requirements: Vec<String>,
    pub added_risks: Vec<String>,
    pub added_dependencies: Vec<String>,
}

/// Metadata for the PRD generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRDMetadata {
    pub meeting_id: String,
    pub meeting_name: String,
    pub total_versions: u32,
    pub latest_version: u32,
    pub first_generated_at: String,
    pub last_updated_at: String,
}
