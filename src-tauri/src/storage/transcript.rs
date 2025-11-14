use crate::managers::meeting::{MeetingSession, TranscriptSegment};
use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Metadata for a meeting transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptMetadata {
    pub meeting_id: String,
    pub name: String,
    pub start_time: String, // ISO 8601 format
    pub end_time: String,   // ISO 8601 format
    pub duration_seconds: u64,
    pub participants: Vec<String>,
}

/// Full transcript with metadata and segments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptData {
    pub meeting_id: String,
    pub segments: Vec<TranscriptSegment>,
}

/// Manages saving and loading meeting transcripts
pub struct TranscriptStorage {
    base_path: PathBuf,
}

impl TranscriptStorage {
    /// Create a new TranscriptStorage
    ///
    /// # Arguments
    /// * `base_path` - Base directory for storing transcripts (e.g., ~/MeetingCoder/meetings/)
    pub fn new(base_path: PathBuf) -> Result<Self> {
        // Create base directory if it doesn't exist with secure permissions
        if !base_path.exists() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                std::fs::DirBuilder::new()
                    .mode(0o700) // User-only access
                    .recursive(true)
                    .create(&base_path)?;
            }
            #[cfg(not(unix))]
            {
                fs::create_dir_all(&base_path)?;
            }
            log::info!(
                "Created transcript storage directory: {}",
                base_path.display()
            );
        }

        Ok(Self { base_path })
    }

    /// Get the default storage path in the user's home directory
    pub fn default_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        Ok(home.join("MeetingCoder").join("meetings"))
    }

    /// Create a new instance with the default path
    pub fn with_default_path() -> Result<Self> {
        Self::new(Self::default_path()?)
    }

    /// Generate a directory name for a meeting
    ///
    /// Format: YYYY-MM-DD_meeting-name
    fn generate_meeting_dir_name(meeting: &MeetingSession) -> String {
        let datetime: DateTime<Local> = meeting.start_time.into();
        let date_str = datetime.format("%Y-%m-%d").to_string();

        // Security: Comprehensive sanitization to prevent path traversal
        let sanitized_name: String = meeting
            .name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
            .collect::<String>()
            .replace(' ', "-")
            .to_lowercase();

        // Ensure we have a valid name
        let sanitized_name = if sanitized_name.is_empty() {
            "untitled".to_string()
        } else {
            sanitized_name.chars().take(100).collect() // Limit length
        };

        // Additional safety: Reject directory traversal patterns
        let sanitized_name = sanitized_name.replace("..", "");

        format!("{}_{}", date_str, sanitized_name)
    }

    /// Get the directory path for a specific meeting
    fn get_meeting_dir(&self, meeting: &MeetingSession) -> PathBuf {
        self.base_path
            .join(Self::generate_meeting_dir_name(meeting))
    }

    /// Save a complete meeting transcript
    pub fn save_transcript(&self, meeting: &MeetingSession) -> Result<PathBuf> {
        let meeting_dir = self.get_meeting_dir(meeting);

        // Create meeting directory with secure permissions
        if !meeting_dir.exists() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                std::fs::DirBuilder::new()
                    .mode(0o700) // User-only access
                    .recursive(true)
                    .create(&meeting_dir)?;
            }
            #[cfg(not(unix))]
            {
                fs::create_dir_all(&meeting_dir)?;
            }
        }

        // Save metadata.json
        let metadata = self.create_metadata(meeting)?;
        let metadata_path = meeting_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        fs::write(&metadata_path, metadata_json)?;

        // Save transcript.json
        let transcript_data = TranscriptData {
            meeting_id: meeting.id.clone(),
            segments: meeting.transcript_segments.clone(),
        };
        let transcript_path = meeting_dir.join("transcript.json");
        let transcript_json = serde_json::to_string_pretty(&transcript_data)?;
        fs::write(&transcript_path, transcript_json)?;

        // Save transcript.md
        let markdown_path = meeting_dir.join("transcript.md");
        let markdown = self.generate_markdown(meeting)?;
        fs::write(&markdown_path, markdown)?;

        log::info!("Saved transcript to: {}", meeting_dir.display());

        Ok(meeting_dir)
    }

    /// Create metadata from a meeting session
    fn create_metadata(&self, meeting: &MeetingSession) -> Result<TranscriptMetadata> {
        let end_time = meeting
            .end_time
            .ok_or_else(|| anyhow::anyhow!("Meeting has no end time"))?;

        let duration = end_time
            .duration_since(meeting.start_time)
            .unwrap_or(std::time::Duration::from_secs(0));

        let start_datetime: DateTime<Local> = meeting.start_time.into();
        let end_datetime: DateTime<Local> = end_time.into();

        Ok(TranscriptMetadata {
            meeting_id: meeting.id.clone(),
            name: meeting.name.clone(),
            start_time: start_datetime.to_rfc3339(),
            end_time: end_datetime.to_rfc3339(),
            duration_seconds: duration.as_secs(),
            participants: meeting.participants.clone(),
        })
    }

    /// Generate a Markdown transcript
    fn generate_markdown(&self, meeting: &MeetingSession) -> Result<String> {
        let start_datetime: DateTime<Local> = meeting.start_time.into();
        let duration = if let Some(end_time) = meeting.end_time {
            end_time
                .duration_since(meeting.start_time)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs()
                / 60
        } else {
            0
        };

        let mut markdown = String::new();

        // Header
        markdown.push_str(&format!("# {}\n\n", meeting.name));
        markdown.push_str(&format!(
            "**Date**: {}\n",
            start_datetime.format("%B %d, %Y")
        ));
        markdown.push_str(&format!("**Duration**: {} minutes\n", duration));
        markdown.push_str(&format!(
            "**Participants**: {}\n\n",
            meeting.participants.join(", ")
        ));
        markdown.push_str("---\n\n");

        // Transcript segments
        for segment in &meeting.transcript_segments {
            let timestamp = self.format_timestamp(segment.start_time);
            markdown.push_str(&format!("**[{}] {}:**\n", timestamp, segment.speaker));
            markdown.push_str(&format!("{}\n\n", segment.text));
        }

        Ok(markdown)
    }

    /// Format a timestamp in seconds to HH:MM:SS
    fn format_timestamp(&self, seconds: f64) -> String {
        let total_seconds = seconds as u64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let secs = total_seconds % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    }

    /// Load a transcript from disk
    pub fn load_transcript(
        &self,
        meeting_dir_name: &str,
    ) -> Result<(TranscriptMetadata, TranscriptData)> {
        let meeting_dir = self.base_path.join(meeting_dir_name);

        if !meeting_dir.exists() {
            return Err(anyhow::anyhow!(
                "Meeting directory not found: {}",
                meeting_dir.display()
            ));
        }

        // Load metadata
        let metadata_path = meeting_dir.join("metadata.json");
        let metadata_content = fs::read_to_string(&metadata_path)?;
        let metadata: TranscriptMetadata = serde_json::from_str(&metadata_content)?;

        // Load transcript
        let transcript_path = meeting_dir.join("transcript.json");
        let transcript_content = fs::read_to_string(&transcript_path)?;
        let transcript: TranscriptData = serde_json::from_str(&transcript_content)?;

        Ok((metadata, transcript))
    }

    /// List all saved meeting directories
    pub fn list_meetings(&self) -> Result<Vec<String>> {
        if !self.base_path.exists() {
            return Ok(Vec::new());
        }

        let mut meetings = Vec::new();

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    meetings.push(name.to_string_lossy().to_string());
                }
            }
        }

        meetings.sort();
        meetings.reverse(); // Most recent first

        Ok(meetings)
    }

    /// Delete a meeting transcript
    pub fn delete_transcript(&self, meeting_dir_name: &str) -> Result<()> {
        let meeting_dir = self.base_path.join(meeting_dir_name);

        if meeting_dir.exists() {
            fs::remove_dir_all(&meeting_dir)?;
            log::info!("Deleted transcript: {}", meeting_dir.display());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::managers::meeting::MeetingStatus;
    use std::time::{Duration, SystemTime};
    use tempfile::TempDir;

    fn create_test_meeting() -> MeetingSession {
        let start = SystemTime::now();
        let end = start + Duration::from_secs(300); // 5 minutes

        MeetingSession {
            id: "test-123".to_string(),
            name: "Test Meeting".to_string(),
            start_time: start,
            end_time: Some(end),
            status: MeetingStatus::Completed,
            participants: vec!["Speaker 1".to_string(), "Speaker 2".to_string()],
            project_path: None,
            transcript_segments: vec![
                TranscriptSegment {
                    speaker: "Speaker 1".to_string(),
                    start_time: 0.0,
                    end_time: 3.5,
                    text: "Hello, welcome to the meeting.".to_string(),
                    confidence: 0.95,
                    timestamp: start,
                },
                TranscriptSegment {
                    speaker: "Speaker 2".to_string(),
                    start_time: 3.5,
                    end_time: 7.0,
                    text: "Thanks, glad to be here.".to_string(),
                    confidence: 0.92,
                    timestamp: start + Duration::from_secs(3),
                },
            ],
        }
    }

    #[test]
    fn test_save_and_load_transcript() {
        let temp_dir = TempDir::new().unwrap();
        let storage = TranscriptStorage::new(temp_dir.path().to_path_buf()).unwrap();

        let meeting = create_test_meeting();
        let saved_path = storage.save_transcript(&meeting).unwrap();

        // Verify files exist
        assert!(saved_path.join("metadata.json").exists());
        assert!(saved_path.join("transcript.json").exists());
        assert!(saved_path.join("transcript.md").exists());

        // Load back and verify
        let dir_name = saved_path.file_name().unwrap().to_str().unwrap();
        let (metadata, transcript) = storage.load_transcript(dir_name).unwrap();

        assert_eq!(metadata.meeting_id, meeting.id);
        assert_eq!(metadata.name, meeting.name);
        assert_eq!(transcript.segments.len(), 2);
    }

    #[test]
    fn test_list_meetings() {
        let temp_dir = TempDir::new().unwrap();
        let storage = TranscriptStorage::new(temp_dir.path().to_path_buf()).unwrap();

        // Save two meetings
        let meeting1 = create_test_meeting();
        storage.save_transcript(&meeting1).unwrap();

        let mut meeting2 = create_test_meeting();
        meeting2.name = "Another Meeting".to_string();
        storage.save_transcript(&meeting2).unwrap();

        // List meetings
        let meetings = storage.list_meetings().unwrap();
        assert_eq!(meetings.len(), 2);
    }

    #[test]
    fn test_markdown_generation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = TranscriptStorage::new(temp_dir.path().to_path_buf()).unwrap();

        let meeting = create_test_meeting();
        let saved_path = storage.save_transcript(&meeting).unwrap();

        let markdown_content = fs::read_to_string(saved_path.join("transcript.md")).unwrap();

        assert!(markdown_content.contains("# Test Meeting"));
        assert!(markdown_content.contains("**Participants**: Speaker 1, Speaker 2"));
        assert!(markdown_content.contains("Hello, welcome to the meeting."));
        assert!(markdown_content.contains("[00:00:00] Speaker 1:"));
    }

    #[test]
    fn test_delete_transcript() {
        let temp_dir = TempDir::new().unwrap();
        let storage = TranscriptStorage::new(temp_dir.path().to_path_buf()).unwrap();

        let meeting = create_test_meeting();
        let saved_path = storage.save_transcript(&meeting).unwrap();

        assert!(saved_path.exists());

        let dir_name = saved_path.file_name().unwrap().to_str().unwrap();
        storage.delete_transcript(dir_name).unwrap();

        assert!(!saved_path.exists());
    }
}
