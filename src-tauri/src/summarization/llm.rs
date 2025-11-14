use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

const KEYCHAIN_SERVICE: &str = "com.meetingcoder.app";
const KEYCHAIN_ACCOUNT: &str = "claude_api_key";

/// Store Claude API key securely using the OS keychain
pub fn store_api_key(api_key: &str) -> Result<()> {
    log::info!(
        "Attempting to store Claude API key (length: {})",
        api_key.len()
    );

    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .map_err(|e| anyhow!("Failed to create keyring entry: {}", e))?;
    entry
        .set_password(api_key)
        .map_err(|e| anyhow!("Failed to store API key in keyring: {}", e))?;
    log::info!("Successfully stored Claude API key in system keyring");
    Ok(())
}

/// Retrieve Claude API key from the OS keychain
pub fn get_api_key() -> Result<String> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .map_err(|e| anyhow!("Failed to create keyring entry: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("Failed to retrieve API key from keyring: {}", e))
}

/// Delete Claude API key from the OS keychain
pub fn delete_api_key() -> Result<()> {
    if let Ok(entry) = keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT) {
        let _ = entry.delete_credential();
    }
    Ok(())
}

/// Check if API key is configured
pub fn has_api_key() -> bool {
    get_api_key().is_ok()
}

// ===== Claude API Structures =====

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String, // "user" or "assistant"
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    #[allow(dead_code)]
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct ExtractionResult {
    pub new_features: Vec<ExtractedFeature>,
    pub technical_decisions: Vec<String>,
    pub questions: Vec<String>,
    #[serde(default)]
    pub project_type: Option<String>,
    #[serde(default)]
    pub target_files: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExtractedFeature {
    pub title: String,
    pub description: String,
    pub priority: String, // "high", "medium", "low"
    #[serde(default)]
    pub confidence: f64,
}

// ===== Claude API Client =====

pub async fn call_claude_api(
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    let api_key = get_api_key()?;

    let request = ClaudeRequest {
        model: model.to_string(),
        max_tokens: 4096,
        messages: vec![ClaudeMessage {
            role: "user".to_string(),
            content: user_prompt.to_string(),
        }],
        system: Some(system_prompt.to_string()),
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to send request to Claude API: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow!("Claude API error {}: {}", status, error_text));
    }

    let claude_response: ClaudeResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse Claude API response: {}", e))?;

    // Extract text from first content block
    let text = claude_response
        .content
        .first()
        .map(|c| c.text.clone())
        .ok_or_else(|| anyhow!("No content in Claude API response"))?;

    Ok(text)
}

// ===== Prompt Templates =====

pub fn get_system_prompt() -> &'static str {
    r#"You are an expert requirement extraction assistant for MeetingCoder, a system that converts meeting transcripts into code in real-time.

Your task is to analyze meeting transcript segments and extract structured information:
1. **New Features**: User stories, feature requests, or functionality requirements
2. **Technical Decisions**: Architecture choices, tech stack decisions, implementation approaches
3. **Questions**: Clarifications needed, open questions, or ambiguities
4. **Project Type** (first update only): Overall project category
5. **Target Files**: Files, components, or paths mentioned in the discussion

For each feature, include:
- title: Short, actionable title (5-10 words)
- description: Clear description of what needs to be built
- priority: "high" (must have, urgent), "medium" (should have, important), "low" (nice to have)
- confidence: 0.0-1.0 indicating how certain you are this is a real requirement

For target_files, extract:
- File paths mentioned (e.g., "src/components/Button.tsx", "api/users.ts")
- Component names (e.g., "UserProfile", "Dashboard")
- Directory references (e.g., "the components folder", "API routes")
- Code elements discussed (e.g., "HomePage.tsx", "login.py", "auth middleware")

Guidelines:
- Focus on actionable requirements, not general discussion
- Distinguish between requirements ("we need X") and questions ("should we use X?")
- Infer priority from language: "must", "need" → high; "should", "important" → medium; "could", "maybe" → low
- Be conservative: only extract clear requirements, not vague ideas
- Deduplicate: skip if very similar to previous features
- Extract file/component mentions even if paths aren't fully specified

Output valid JSON only (no markdown, no explanation)."#
}

pub fn build_extraction_prompt(transcript_text: &str, is_first_update: bool) -> String {
    let mut prompt = format!(
        r#"Extract requirements from this meeting transcript segment:

<transcript>
{}
</transcript>

Return JSON in this format:
{{"#,
        transcript_text
    );

    if is_first_update {
        prompt.push_str(
            r#"
  "project_type": "web_app|mobile_app|api_backend|cli_tool|other","#,
        );
    }

    prompt.push_str(
        r#"
  "new_features": [
    {
      "title": "Feature title",
      "description": "Detailed description",
      "priority": "high|medium|low",
      "confidence": 0.9
    }
  ],
  "technical_decisions": [
    "Decision or tech choice made in the discussion"
  ],
  "questions": [
    "Open question or clarification needed"
  ],
  "target_files": [
    "path/to/file.tsx",
    "ComponentName",
    "folder/name"
  ]
}"#,
    );

    prompt
}

// ===== Integration with existing agent =====

use crate::managers::meeting::TranscriptSegment;
use crate::summarization::agent::{Feature, Priority, SummarizationOutput};
use std::collections::HashSet;

pub async fn summarize_with_llm(
    model: &str,
    segments: &[TranscriptSegment],
    start_index: usize,
    end_index: usize,
    is_first_update: bool,
) -> Result<SummarizationOutput> {
    // Combine transcript segments into a single text
    let mut transcript_text = String::new();
    for seg in segments {
        transcript_text.push_str(&format!(
            "[{:.1}s] {}: {}\n",
            seg.start_time, seg.speaker, seg.text
        ));
    }

    let system_prompt = get_system_prompt();
    let user_prompt = build_extraction_prompt(&transcript_text, is_first_update);

    log::info!("Calling Claude API for summarization...");
    let response_text = call_claude_api(model, system_prompt, &user_prompt).await?;

    log::debug!("Claude API response: {}", response_text);

    // Parse JSON response
    let extraction: ExtractionResult = serde_json::from_str(&response_text).map_err(|e| {
        anyhow!(
            "Failed to parse Claude API JSON response: {}\nResponse: {}",
            e,
            response_text
        )
    })?;

    log::info!(
        "LLM extracted {} features, {} decisions, {} questions",
        extraction.new_features.len(),
        extraction.technical_decisions.len(),
        extraction.questions.len()
    );

    // Convert to existing SummarizationOutput format
    let mut new_features = Vec::new();
    let mut new_features_structured = Vec::new();
    let mut seen = HashSet::new();

    for (idx, feat) in extraction.new_features.iter().enumerate() {
        let id = format!("f{:016x}", idx); // Simple ID generation for now
        if seen.insert(id.clone()) {
            new_features.push(feat.title.clone());
            new_features_structured.push(Feature {
                id,
                title: feat.title.clone(),
                description: feat.description.clone(),
                priority: match feat.priority.as_str() {
                    "high" => Priority::High,
                    "medium" => Priority::Medium,
                    _ => Priority::Low,
                },
                technical_notes: None,
                mentioned_by: segments
                    .first()
                    .map(|s| s.speaker.clone())
                    .unwrap_or_else(|| "Unknown".to_string()),
                timestamp: segments.first().map(|s| s.start_time).unwrap_or(0.0),
            });
        }
    }

    Ok(SummarizationOutput {
        timestamp: chrono::Utc::now().to_rfc3339(),
        segment_range: (start_index, end_index),
        new_features,
        technical_decisions: extraction.technical_decisions,
        questions: extraction.questions,
        new_features_structured,
        modified_features: None,
        clarifications: None,
        target_files: extraction.target_files,
    })
}
