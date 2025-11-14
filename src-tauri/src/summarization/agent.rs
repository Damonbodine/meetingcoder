use crate::managers::meeting::TranscriptSegment;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_notes: Option<Vec<String>>,
    pub mentioned_by: String,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizationOutput {
    pub timestamp: String,             // RFC3339
    pub segment_range: (usize, usize), // inclusive range in full transcript

    // Back-compat flat lists
    pub new_features: Vec<String>,
    pub technical_decisions: Vec<String>,
    pub questions: Vec<String>,

    // Enriched fields
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub new_features_structured: Vec<Feature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_features: Option<HashMap<String, serde_json::Value>>, // Partial<Feature>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clarifications: Option<HashMap<String, String>>,
    // Code-aware: files mentioned in transcript
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub target_files: Vec<String>,
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    let lower = haystack.to_lowercase();
    needles.iter().any(|n| lower.contains(n))
}

fn normalize_sentence(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut prev_ws = false;
    for ch in lower.chars() {
        if ch.is_alphanumeric() || ch == ' ' {
            if ch.is_whitespace() {
                if !prev_ws {
                    out.push(' ');
                }
                prev_ws = true;
            } else {
                out.push(ch);
                prev_ws = false;
            }
        }
    }
    out.trim().trim_end_matches('.').to_string()
}

fn hash_id(s: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    format!("f{:016x}", hasher.finish())
}

fn infer_priority(s: &str) -> Priority {
    let l = s.to_lowercase();
    if l.contains("must") || l.contains("need") || l.contains("urgent") {
        Priority::High
    } else if l.contains("should") || l.contains("important") {
        Priority::Medium
    } else {
        Priority::Low
    }
}

fn load_seen_feature_ids(project_path: Option<&str>, max_lines: usize) -> HashSet<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let Some(path) = project_path.map(|p| Path::new(p).join(".meeting-updates.jsonl")) else {
        return seen;
    };
    let Ok(file) = File::open(path) else {
        return seen;
    };
    let reader = BufReader::new(file);
    // Keep only last max_lines using a ring buffer of strings
    let mut buf: std::collections::VecDeque<String> =
        std::collections::VecDeque::with_capacity(max_lines);
    for line in reader.lines().flatten() {
        if buf.len() == max_lines {
            buf.pop_front();
        }
        buf.push_back(line);
    }
    for line in buf {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
            // Collect structured ids
            if let Some(arr) = v.get("new_features_structured").and_then(|x| x.as_array()) {
                for f in arr {
                    if let Some(id) = f.get("id").and_then(|x| x.as_str()) {
                        seen.insert(id.to_string());
                    }
                }
            }
            // Collect from legacy strings (hash of text)
            if let Some(arr) = v.get("new_features").and_then(|x| x.as_array()) {
                for s in arr.iter().filter_map(|x| x.as_str()) {
                    let nid = hash_id(&normalize_sentence(s));
                    seen.insert(nid);
                }
            }
        }
    }
    seen
}

fn process_sentence(
    sentence: &str,
    speaker: &str,
    time_seconds: f64,
    new_features: &mut Vec<String>,
    technical_decisions: &mut Vec<String>,
    questions: &mut Vec<String>,
    structured: &mut Vec<Feature>,
    seen: &mut HashSet<String>,
) {
    let s = sentence.trim();
    if s.is_empty() {
        return;
    }

    if s.ends_with('?') {
        questions.push(s.to_string());
        return;
    }

    // Feature extraction
    if contains_any(s, &["need", "should", "can", "add", "support"]) {
        let norm = normalize_sentence(s);
        let id = hash_id(&norm);
        if !seen.contains(&id) {
            new_features.push(s.to_string());
            structured.push(Feature {
                id: id.clone(),
                title: s.to_string(),
                description: s.to_string(),
                priority: infer_priority(s),
                technical_notes: None,
                mentioned_by: speaker.to_string(),
                timestamp: time_seconds,
            });
            seen.insert(id);
        }
    }
    // Technical decisions
    if contains_any(s, &["decide", "require", "decision", "choose"]) {
        technical_decisions.push(s.to_string());
    }
}

pub fn summarize_segments(
    segments: &[TranscriptSegment],
    start_index: usize,
    end_index: usize,
) -> SummarizationOutput {
    summarize_segments_with_context(None, segments, start_index, end_index)
}

pub fn summarize_segments_with_context(
    project_path: Option<&str>,
    segments: &[TranscriptSegment],
    start_index: usize,
    end_index: usize,
) -> SummarizationOutput {
    let mut new_features = Vec::new();
    let mut technical_decisions = Vec::new();
    let mut questions = Vec::new();
    let mut new_features_structured: Vec<Feature> = Vec::new();
    let mut seen = load_seen_feature_ids(project_path, 50);

    for seg in segments {
        // Split into rough sentences
        let mut sentence = String::new();
        for ch in seg.text.chars() {
            sentence.push(ch);
            if ch == '.' || ch == '!' || ch == '?' {
                process_sentence(
                    &sentence,
                    &seg.speaker,
                    seg.end_time,
                    &mut new_features,
                    &mut technical_decisions,
                    &mut questions,
                    &mut new_features_structured,
                    &mut seen,
                );
                sentence.clear();
            }
        }
        if !sentence.trim().is_empty() {
            process_sentence(
                &sentence,
                &seg.speaker,
                seg.end_time,
                &mut new_features,
                &mut technical_decisions,
                &mut questions,
                &mut new_features_structured,
                &mut seen,
            );
        }
    }

    SummarizationOutput {
        timestamp: Utc::now().to_rfc3339(),
        segment_range: (start_index, end_index),
        new_features,
        technical_decisions,
        questions,
        new_features_structured,
        modified_features: None,
        clarifications: None,
        target_files: Vec::new(), // Will be populated by LLM or file extraction logic
    }
}
