use crate::managers::meeting::TranscriptSegment;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

#[derive(Serialize)]
struct TranscriptJsonlRecord<'a> {
    meeting_id: &'a str,
    segment_index: usize,
    speaker: &'a str,
    start_time: f64,
    end_time: f64,
    confidence: f32,
    text: &'a str,
    timestamp: String, // RFC3339
}

pub fn append_segment(
    project_path: &str,
    meeting_id: &str,
    segment_index: usize,
    segment: &TranscriptSegment,
) -> Result<()> {
    let path = Path::new(project_path).join(".transcript.jsonl");
    let timestamp: DateTime<Utc> = segment.timestamp.into();
    let record = TranscriptJsonlRecord {
        meeting_id,
        segment_index,
        speaker: &segment.speaker,
        start_time: segment.start_time,
        end_time: segment.end_time,
        confidence: segment.confidence,
        text: &segment.text,
        timestamp: timestamp.to_rfc3339(),
    };
    let line = serde_json::to_string(&record)? + "\n";
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(line.as_bytes())?;
    file.flush()?;
    Ok(())
}
