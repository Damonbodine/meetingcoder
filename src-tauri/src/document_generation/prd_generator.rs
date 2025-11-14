use super::prd_analyzer::*;
use super::prd_storage::*;
use super::prd_template::*;
use super::types::*;
use crate::managers::meeting::TranscriptSegment;
use crate::summarization::agent::SummarizationOutput;
use anyhow::{Context, Result};
use std::time::Duration;

/// Main PRD generator for a meeting
pub struct PRDGenerator {
    meeting_id: String,
    meeting_name: String,
    project_type: Option<String>,
    versions: Vec<PRDVersion>,
    last_segment_processed: usize,
}

impl PRDGenerator {
    /// Create a new PRD generator for a meeting
    pub fn new(meeting_id: String, meeting_name: String) -> Self {
        Self {
            meeting_id,
            meeting_name,
            project_type: None,
            versions: Vec::new(),
            last_segment_processed: 0,
        }
    }

    /// Load existing PRD generator state from disk
    pub fn load(meeting_id: String, meeting_name: String) -> Result<Self> {
        let versions = get_all_versions(&meeting_id)?;

        let last_segment_processed = versions.last().map(|v| v.segment_range.1).unwrap_or(0);

        Ok(Self {
            meeting_id,
            meeting_name,
            project_type: None,
            versions,
            last_segment_processed,
        })
    }

    /// Set the project type (extracted from meeting context)
    pub fn set_project_type(&mut self, project_type: String) {
        self.project_type = Some(project_type);
    }

    /// Determine if it's time to generate a new PRD version
    pub fn should_generate_version(
        &self,
        total_segments: usize,
        time_since_last: Duration,
        min_segments: usize,
        update_interval_minutes: u64,
    ) -> bool {
        // Initial PRD: after minimum segments threshold
        if self.versions.is_empty() {
            return total_segments >= min_segments;
        }

        // Subsequent updates: based on time interval and new segments
        let new_segments = total_segments.saturating_sub(self.last_segment_processed);
        let time_threshold = Duration::from_secs(update_interval_minutes * 60);

        // Require at least 5 new segments and time interval passed
        new_segments >= 5 && time_since_last >= time_threshold
    }

    /// Generate initial PRD (v1)
    pub async fn generate_initial_prd(
        &mut self,
        transcript: &[TranscriptSegment],
        feature_extractions: &[SummarizationOutput],
        project_context: Option<String>,
    ) -> Result<PRDVersion> {
        log::info!("Generating initial PRD for meeting {}", self.meeting_id);

        // Extract PRD content using LLM
        let content = self
            .extract_prd_content_with_llm(
                transcript,
                feature_extractions,
                project_context,
                None, // No previous PRD
            )
            .await?;

        // Create version metadata
        let version = PRDVersion {
            version: 1,
            generated_at: chrono::Utc::now().to_rfc3339(),
            segment_range: (0, transcript.len()),
            total_segments: transcript.len(),
            file_path: String::new(), // Will be set by storage
            version_type: "initial".to_string(),
            confidence: 0.8, // Default confidence for initial
            word_count: count_words(&content),
        };

        // Render markdown
        let markdown = render_prd_markdown(
            &content,
            &version,
            &self.meeting_name,
            &self.project_type.as_deref().unwrap_or("Unnamed Project"),
        );

        // Save to disk
        let file_path = save_prd_version(&self.meeting_id, &version, &markdown, &content)?;

        let mut version_with_path = version;
        version_with_path.file_path = file_path;

        // Update metadata
        update_metadata(&self.meeting_id, &self.meeting_name, 1)?;

        // Update internal state
        self.versions.push(version_with_path.clone());
        self.last_segment_processed = transcript.len();

        log::info!(
            "Generated initial PRD v1 with {} segments",
            transcript.len()
        );

        Ok(version_with_path)
    }

    /// Generate incremental update (v2+)
    pub async fn generate_incremental_update(
        &mut self,
        new_transcript: &[TranscriptSegment],
        new_extractions: &[SummarizationOutput],
    ) -> Result<PRDVersion> {
        let version_number = (self.versions.len() + 1) as u32;

        log::info!(
            "Generating incremental PRD v{} for meeting {}",
            version_number,
            self.meeting_id
        );

        // Load previous PRD
        let previous_version = self.versions.last().context("No previous version found")?;
        let (_, previous_content, _) =
            load_prd_version(&self.meeting_id, previous_version.version)?;

        // Extract updated PRD content using LLM
        let updated_content = self
            .extract_prd_content_with_llm(
                new_transcript,
                new_extractions,
                None,
                Some(&previous_content),
            )
            .await?;

        // Analyze changes
        let mut change = analyze_changes(&previous_content, &updated_content);
        change.from_version = previous_version.version;
        change.to_version = version_number;

        // Create version metadata
        let start_segment = self.last_segment_processed;
        let end_segment = start_segment + new_transcript.len();

        let version = PRDVersion {
            version: version_number,
            generated_at: chrono::Utc::now().to_rfc3339(),
            segment_range: (start_segment, end_segment),
            total_segments: end_segment,
            file_path: String::new(),
            version_type: "incremental".to_string(),
            confidence: 0.85,
            word_count: count_words(&updated_content),
        };

        // Render markdown
        let markdown = render_prd_markdown(
            &updated_content,
            &version,
            &self.meeting_name,
            &self.project_type.as_deref().unwrap_or("Unnamed Project"),
        );

        // Save to disk
        let file_path = save_prd_version(&self.meeting_id, &version, &markdown, &updated_content)?;

        let mut version_with_path = version;
        version_with_path.file_path = file_path;

        // Update changelog
        let mut changelog = load_changelog(&self.meeting_id)?;
        changelog.changes.push(change);
        save_changelog(&self.meeting_id, &changelog)?;

        // Update metadata
        update_metadata(&self.meeting_id, &self.meeting_name, version_number)?;

        // Update internal state
        self.versions.push(version_with_path.clone());
        self.last_segment_processed = end_segment;

        log::info!(
            "Generated incremental PRD v{} with {} new segments",
            version_number,
            new_transcript.len()
        );

        Ok(version_with_path)
    }

    /// Generate final PRD at meeting end
    pub async fn generate_final_prd(
        &mut self,
        all_transcript: &[TranscriptSegment],
        all_extractions: &[SummarizationOutput],
    ) -> Result<PRDVersion> {
        let version_number = (self.versions.len() + 1) as u32;

        log::info!(
            "Generating final PRD v{} for meeting {}",
            version_number,
            self.meeting_id
        );

        // Load previous PRD if exists
        let previous_content = if let Some(previous_version) = self.versions.last() {
            let (_, content, _) = load_prd_version(&self.meeting_id, previous_version.version)?;
            Some(content)
        } else {
            None
        };

        // Extract complete PRD content using LLM with full context
        let final_content = self
            .extract_prd_content_with_llm(
                all_transcript,
                all_extractions,
                None,
                previous_content.as_ref(),
            )
            .await?;

        // Create version metadata
        let version = PRDVersion {
            version: version_number,
            generated_at: chrono::Utc::now().to_rfc3339(),
            segment_range: (0, all_transcript.len()),
            total_segments: all_transcript.len(),
            file_path: String::new(),
            version_type: "final".to_string(),
            confidence: 0.9, // Higher confidence for final
            word_count: count_words(&final_content),
        };

        // Render markdown
        let markdown = render_prd_markdown(
            &final_content,
            &version,
            &self.meeting_name,
            &self.project_type.as_deref().unwrap_or("Unnamed Project"),
        );

        // Save to disk
        let file_path = save_prd_version(&self.meeting_id, &version, &markdown, &final_content)?;

        let mut version_with_path = version;
        version_with_path.file_path = file_path;

        // Analyze changes if there was a previous version
        if let Some(prev_content) = previous_content {
            let mut change = analyze_changes(&prev_content, &final_content);
            change.from_version = self.versions.last().unwrap().version;
            change.to_version = version_number;

            let mut changelog = load_changelog(&self.meeting_id)?;
            changelog.changes.push(change);
            save_changelog(&self.meeting_id, &changelog)?;
        }

        // Update metadata
        update_metadata(&self.meeting_id, &self.meeting_name, version_number)?;

        // Update internal state
        self.versions.push(version_with_path.clone());
        self.last_segment_processed = all_transcript.len();

        log::info!(
            "Generated final PRD v{} with {} total segments",
            version_number,
            all_transcript.len()
        );

        Ok(version_with_path)
    }

    /// Get latest PRD version
    pub fn get_latest_version(&self) -> Option<&PRDVersion> {
        self.versions.last()
    }

    /// Get all versions
    pub fn get_all_versions(&self) -> &[PRDVersion] {
        &self.versions
    }

    /// Get changelog between versions
    pub fn get_changelog(&self, from: u32, to: u32) -> Result<PRDChange> {
        let changelog = load_changelog(&self.meeting_id)?;

        changelog
            .changes
            .iter()
            .find(|c| c.from_version == from && c.to_version == to)
            .cloned()
            .context("Changelog entry not found")
    }

    // Private helper methods

    async fn extract_prd_content_with_llm(
        &self,
        transcript: &[TranscriptSegment],
        extractions: &[SummarizationOutput],
        project_context: Option<String>,
        previous_content: Option<&PRDContent>,
    ) -> Result<PRDContent> {
        use crate::summarization::llm::{call_claude_api, has_api_key};

        // Check if LLM is available
        if !has_api_key() {
            log::warn!("No Claude API key found, falling back to heuristic PRD extraction");
            return self.extract_prd_content_heuristic(extractions, previous_content);
        }

        // Prepare prompts
        let system_prompt = get_prd_system_prompt();
        let user_prompt = if let Some(prev) = previous_content {
            get_prd_update_prompt(transcript, extractions, project_context, prev)
        } else {
            get_prd_initial_prompt(transcript, extractions, project_context)
        };

        // Call Claude API
        match call_claude_api("claude-sonnet-4-5-20250929", &system_prompt, &user_prompt).await {
            Ok(response) => {
                // Parse JSON response
                let content: PRDContent = serde_json::from_str(&response)
                    .context("Failed to parse LLM response as PRDContent")?;

                Ok(content)
            }
            Err(e) => {
                log::warn!(
                    "LLM PRD extraction failed: {}, falling back to heuristic",
                    e
                );
                self.extract_prd_content_heuristic(extractions, previous_content)
            }
        }
    }

    fn extract_prd_content_heuristic(
        &self,
        extractions: &[SummarizationOutput],
        previous_content: Option<&PRDContent>,
    ) -> Result<PRDContent> {
        // Start with previous content or empty
        let mut content = previous_content.cloned().unwrap_or_default();

        // Extract from summarization outputs
        for extraction in extractions {
            // Add new features as user stories
            for (idx, feature) in extraction.new_features.iter().enumerate() {
                let story_id = format!("US-{:03}", content.user_stories.len() + idx + 1);
                let story = UserStory {
                    id: story_id,
                    persona: "user".to_string(),
                    want: feature.clone(),
                    so_that: "achieve goals".to_string(),
                    priority: "medium".to_string(),
                    status: "planned".to_string(),
                    mentioned_at: vec![extraction.segment_range.0],
                };
                content.user_stories.push(story);
            }

            // Add technical decisions as technical requirements
            for (idx, decision) in extraction.technical_decisions.iter().enumerate() {
                let tech_id = format!("TECH-{:03}", content.technical_requirements.len() + idx + 1);
                let tech_req = TechnicalRequirement {
                    id: tech_id,
                    category: "technology".to_string(),
                    title: decision.clone(),
                    description: decision.clone(),
                    rationale: "Discussed in meeting".to_string(),
                    alternatives_considered: vec![],
                    mentioned_at: vec![extraction.segment_range.0],
                };
                content.technical_requirements.push(tech_req);
            }

            // Add questions as open questions
            for (idx, question) in extraction.questions.iter().enumerate() {
                let q_id = format!("Q-{:03}", content.open_questions.len() + idx + 1);
                let q = Question {
                    id: q_id,
                    question: question.clone(),
                    context: "From meeting".to_string(),
                    asked_at: extraction.segment_range.0,
                    resolved: false,
                    resolution: None,
                };
                content.open_questions.push(q);
            }
        }

        // Generate basic executive summary if empty
        if content.executive_summary.is_empty() && !content.user_stories.is_empty() {
            content.executive_summary = format!(
                "This project aims to implement {} features with {} technical requirements.",
                content.user_stories.len(),
                content.technical_requirements.len()
            );
        }

        Ok(content)
    }
}

// Helper functions

fn count_words(content: &PRDContent) -> usize {
    let mut count = content.executive_summary.split_whitespace().count();

    for story in &content.user_stories {
        count += story.persona.split_whitespace().count();
        count += story.want.split_whitespace().count();
        count += story.so_that.split_whitespace().count();
    }

    for req in &content.functional_requirements {
        count += req.title.split_whitespace().count();
        count += req.description.split_whitespace().count();
    }

    count
}

fn get_prd_system_prompt() -> String {
    r#"You are an expert product manager and technical writer. Your role is to analyze meeting transcripts and feature extractions to create comprehensive, professional Product Requirements Documents (PRDs).

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

Output Format: JSON structure matching PRDContent schema. Return ONLY valid JSON, no markdown formatting."#.to_string()
}

fn get_prd_initial_prompt(
    transcript: &[TranscriptSegment],
    extractions: &[SummarizationOutput],
    project_context: Option<String>,
) -> String {
    let transcript_text = format_transcript(transcript);
    let extractions_json = serde_json::to_string_pretty(extractions).unwrap_or_default();
    let context_text = project_context.unwrap_or_else(|| "Not specified".to_string());

    format!(
        r#"Generate an initial Product Requirements Document based on the following meeting context:

**Project Context**: {}

**Transcript Segments**:
{}

**Extracted Features**:
{}

Please create a comprehensive PRD with:
1. Executive summary (2-3 sentences)
2. User stories (at least 3-5)
3. Functional requirements (at least 5-10)
4. Non-functional requirements (if mentioned)
5. Technical requirements (frameworks, libraries, architecture)
6. Acceptance criteria for key requirements
7. Dependencies (if mentioned)
8. Risks (if mentioned)
9. Timeline milestones (if dates/phases mentioned)
10. Open questions (uncertainties or ambiguities)

Return only valid JSON matching the PRDContent schema."#,
        context_text, transcript_text, extractions_json
    )
}

fn get_prd_update_prompt(
    new_transcript: &[TranscriptSegment],
    new_extractions: &[SummarizationOutput],
    _project_context: Option<String>,
    previous_content: &PRDContent,
) -> String {
    let transcript_text = format_transcript(new_transcript);
    let extractions_json = serde_json::to_string_pretty(new_extractions).unwrap_or_default();
    let previous_json = serde_json::to_string_pretty(previous_content).unwrap_or_default();

    format!(
        r#"Update the existing Product Requirements Document with new information from the meeting.

**Previous PRD**:
{}

**New Transcript Segments**:
{}

**New Feature Extractions**:
{}

**Changes to Make**:
1. Add any NEW user stories, requirements, or technical decisions
2. MODIFY existing items if new information clarifies or changes them
3. RESOLVE open questions if answers were provided
4. Add NEW open questions if ambiguities arose
5. Update priorities if emphasis changed
6. Add risks or dependencies if mentioned
7. Maintain ALL item IDs from previous version (do not renumber)
8. For new items, use next sequential ID (e.g., if last was US-005, new is US-006)

Return complete updated PRDContent as JSON."#,
        previous_json, transcript_text, extractions_json
    )
}

fn format_transcript(segments: &[TranscriptSegment]) -> String {
    segments
        .iter()
        .map(|s| format!("[{}] {}", s.speaker, s.text))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prd_generator_new() {
        let gen = PRDGenerator::new("test-id".to_string(), "Test Meeting".to_string());
        assert_eq!(gen.meeting_id, "test-id");
        assert_eq!(gen.meeting_name, "Test Meeting");
        assert_eq!(gen.versions.len(), 0);
        assert_eq!(gen.last_segment_processed, 0);
    }

    #[test]
    fn test_should_generate_version_initial() {
        let gen = PRDGenerator::new("test-id".to_string(), "Test".to_string());

        // Should generate after reaching minimum segments
        assert!(gen.should_generate_version(15, Duration::from_secs(0), 15, 15));

        // Should not generate before minimum segments
        assert!(!gen.should_generate_version(10, Duration::from_secs(0), 15, 15));
    }

    #[test]
    fn test_count_words() {
        let mut content = PRDContent::default();
        content.executive_summary = "This is a test summary with seven words".to_string();

        assert_eq!(count_words(&content), 8);
    }
}
