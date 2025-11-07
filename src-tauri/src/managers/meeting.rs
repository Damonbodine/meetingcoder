use crate::managers::audio::AudioRecordingManager;
use crate::managers::transcription::TranscriptionManager;
use crate::storage::transcript::TranscriptStorage;
use crate::integrations::github;
use crate::settings;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Represents the current status of a meeting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MeetingStatus {
    Recording,
    Paused,
    Completed,
}

/// A single transcript segment with speaker identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    /// Speaker identifier (e.g., "Speaker 1", "Speaker 2", or custom name)
    pub speaker: String,
    /// Start time in seconds from meeting start
    pub start_time: f64,
    /// End time in seconds from meeting start
    pub end_time: f64,
    /// Transcribed text
    pub text: String,
    /// Confidence score from transcription model (0.0 to 1.0)
    pub confidence: f32,
    /// Absolute timestamp when this segment was created
    pub timestamp: SystemTime,
}

/// A complete meeting session with all metadata and transcript segments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSession {
    /// Unique identifier for this meeting
    pub id: String,
    /// User-provided or auto-generated meeting name
    pub name: String,
    /// When the meeting started
    pub start_time: SystemTime,
    /// When the meeting ended (None if still in progress)
    pub end_time: Option<SystemTime>,
    /// All transcript segments accumulated during the meeting
    pub transcript_segments: Vec<TranscriptSegment>,
    /// Current status of the meeting
    pub status: MeetingStatus,
    /// List of participant speaker labels
    pub participants: Vec<String>,
    /// Optional project path for context updates
    pub project_path: Option<String>,
}

/// Summary information returned when a meeting ends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSummary {
    pub meeting_id: String,
    pub name: String,
    pub duration_seconds: u64,
    pub total_segments: usize,
    pub participants: Vec<String>,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
}

/// Manages the lifecycle of meeting sessions, coordinating between
/// audio recording and transcription managers
pub struct MeetingManager {
    /// Current active meetings (by meeting_id)
    active_meetings: Arc<Mutex<HashMap<String, MeetingSession>>>,
    /// Background task handles for chunking loops
    task_handles: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    /// Storage for saving transcripts
    transcript_storage: Arc<TranscriptStorage>,
    /// Audio recording manager for capturing system audio
    audio_manager: Arc<AudioRecordingManager>,
    /// Transcription manager for converting audio to text
    transcription_manager: Arc<TranscriptionManager>,
    /// App handle for emitting events
    app_handle: AppHandle,
}

impl MeetingManager {
    /// Create a new MeetingManager instance
    pub fn new(
        app_handle: &AppHandle,
        audio_manager: Arc<AudioRecordingManager>,
        transcription_manager: Arc<TranscriptionManager>,
    ) -> Result<Self> {
        let transcript_storage = TranscriptStorage::with_default_path()?;

        Ok(Self {
            active_meetings: Arc::new(Mutex::new(HashMap::new())),
            task_handles: Arc::new(Mutex::new(HashMap::new())),
            transcript_storage: Arc::new(transcript_storage),
            audio_manager,
            transcription_manager,
            app_handle: app_handle.clone(),
        })
    }

    /// Start a new meeting session
    ///
    /// # Arguments
    /// * `name` - User-provided name for the meeting
    ///
    /// # Returns
    /// The unique meeting_id for this session
    pub async fn start_meeting(&self, name: String) -> Result<String> {
        let meeting_id = Uuid::new_v4().to_string();
        // Initialize meeting in selected GitHub repo when enabled, else fallback to MeetingCoder workspace
        let settings = settings::get_settings(&self.app_handle);
        let project_path = if settings.github_enabled
            && settings.github_repo_owner.is_some()
            && settings.github_repo_name.is_some()
        {
            let owner = settings.github_repo_owner.clone().unwrap();
            let repo = settings.github_repo_name.clone().unwrap();
            match github::get_github_token()
                .and_then(|token| github::ensure_local_repo_clone(&owner, &repo, &token))
            {
                Ok(repo_root) => {
                    // Seed meeting scaffolding inside the repo root
                    let _ = crate::project::initializer::ProjectInitializer::seed_in_existing_dir_with_app(&std::path::PathBuf::from(&repo_root), &self.app_handle);
                    Some(repo_root)
                }
                Err(e) => {
                    log::warn!("Falling back to MeetingCoder workspace (GitHub clone failed): {}", e);
                    match crate::project::initializer::ProjectInitializer::with_default_path()
                        .and_then(|init| init.init_for_meeting_with_app(&name, &self.app_handle))
                    {
                        Ok(path) => Some(path),
                        Err(e) => { log::warn!("Project initialization failed: {}", e); None }
                    }
                }
            }
        } else {
            // Fallback when GitHub integration not configured
            match crate::project::initializer::ProjectInitializer::with_default_path()
                .and_then(|init| init.init_for_meeting_with_app(&name, &self.app_handle))
            {
                Ok(path) => Some(path),
                Err(e) => { log::warn!("Project initialization failed: {}", e); None }
            }
        };
        let meeting = MeetingSession {
            id: meeting_id.clone(),
            name: name.clone(),
            start_time: SystemTime::now(),
            end_time: None,
            transcript_segments: Vec::new(),
            status: MeetingStatus::Recording,
            participants: Vec::new(),
            project_path,
        };

        // Insert meeting into active meetings
        {
            let mut meetings = self.active_meetings.lock().await;
            meetings.insert(meeting_id.clone(), meeting);
        }

        log::info!("Started meeting: {} (ID: {})", name, meeting_id);

        // Load the transcription model before starting transcription
        log::info!("Loading transcription model...");
        self.transcription_manager.initiate_model_load();

        // Wait for model to load in background task
        // This ensures the model is ready before first transcription
        let transcription_manager = self.transcription_manager.clone();
        tokio::spawn(async move {
            // Wait up to 30 seconds for model to load
            let mut waited = 0;
            while !transcription_manager.is_model_loaded() && waited < 30 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                waited += 1;
            }

            if transcription_manager.is_model_loaded() {
                log::info!("Transcription model loaded successfully");
            } else {
                log::error!("Transcription model failed to load within 30 seconds");
            }
        });

        // Spawn transcription loop task
        let task_handle = tokio::spawn(Self::transcription_loop(
            meeting_id.clone(),
            self.active_meetings.clone(),
            self.audio_manager.clone(),
            self.transcription_manager.clone(),
            self.app_handle.clone(),
        ));

        // Store task handle for cleanup
        {
            let mut handles = self.task_handles.lock().await;
            handles.insert(meeting_id.clone(), task_handle);
        }

        log::info!("Transcription task spawned for meeting: {}", meeting_id);

        Ok(meeting_id)
    }

    /// Start a new meeting session without spawning the live transcription loop.
    /// This is used for offline imports of existing audio.
    pub async fn start_offline_meeting(&self, name: String) -> Result<String> {
        let meeting_id = uuid::Uuid::new_v4().to_string();
        // Initialize meeting in selected GitHub repo when enabled, else fallback to MeetingCoder workspace
        let settings = settings::get_settings(&self.app_handle);
        let project_path = if settings.github_enabled
            && settings.github_repo_owner.is_some()
            && settings.github_repo_name.is_some()
        {
            let owner = settings.github_repo_owner.clone().unwrap();
            let repo = settings.github_repo_name.clone().unwrap();
            match crate::integrations::github::get_github_token()
                .and_then(|token| crate::integrations::github::ensure_local_repo_clone(&owner, &repo, &token))
            {
                Ok(repo_root) => {
                    // Seed meeting scaffolding inside the repo root
                    let _ = crate::project::initializer::ProjectInitializer::seed_in_existing_dir_with_app(&std::path::PathBuf::from(&repo_root), &self.app_handle);
                    Some(repo_root)
                }
                Err(e) => {
                    log::warn!("Falling back to MeetingCoder workspace (GitHub clone failed): {}", e);
                    match crate::project::initializer::ProjectInitializer::with_default_path()
                        .and_then(|init| init.init_for_meeting_with_app(&name, &self.app_handle))
                    {
                        Ok(path) => Some(path),
                        Err(e) => { log::warn!("Project initialization failed: {}", e); None }
                    }
                }
            }
        } else {
            // Fallback when GitHub integration not configured
            match crate::project::initializer::ProjectInitializer::with_default_path()
                .and_then(|init| init.init_for_meeting_with_app(&name, &self.app_handle))
            {
                Ok(path) => Some(path),
                Err(e) => { log::warn!("Project initialization failed: {}", e); None }
            }
        };

        let meeting = MeetingSession {
            id: meeting_id.clone(),
            name: name.clone(),
            start_time: std::time::SystemTime::now(),
            end_time: None,
            transcript_segments: Vec::new(),
            status: MeetingStatus::Recording,
            participants: Vec::new(),
            project_path,
        };

        // Insert into active meetings
        {
            let mut meetings = self.active_meetings.lock().await;
            meetings.insert(meeting_id.clone(), meeting);
        }

        // Note: Do not auto-load a model here for offline imports.
        // The import flow selects and loads the appropriate engine (e.g., Whisper preference).
        Ok(meeting_id)
    }

    /// Pause an active meeting
    pub async fn pause_meeting(&self, meeting_id: &str) -> Result<()> {
        let mut meetings = self.active_meetings.lock().await;

        if let Some(meeting) = meetings.get_mut(meeting_id) {
            if meeting.status == MeetingStatus::Recording {
                meeting.status = MeetingStatus::Paused;
                log::info!("Paused meeting: {}", meeting_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Meeting is not in recording state"))
            }
        } else {
            Err(anyhow::anyhow!("Meeting not found: {}", meeting_id))
        }
    }

    /// Resume a paused meeting
    pub async fn resume_meeting(&self, meeting_id: &str) -> Result<()> {
        let mut meetings = self.active_meetings.lock().await;

        if let Some(meeting) = meetings.get_mut(meeting_id) {
            if meeting.status == MeetingStatus::Paused {
                meeting.status = MeetingStatus::Recording;
                log::info!("Resumed meeting: {}", meeting_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Meeting is not paused"))
            }
        } else {
            Err(anyhow::anyhow!("Meeting not found: {}", meeting_id))
        }
    }

    /// End a meeting and return summary information
    pub async fn end_meeting(&self, meeting_id: &str) -> Result<MeetingSummary> {
        let mut meetings = self.active_meetings.lock().await;

        if let Some(mut meeting) = meetings.remove(meeting_id) {
            meeting.end_time = Some(SystemTime::now());
            meeting.status = MeetingStatus::Completed;

            let duration = meeting.end_time.unwrap()
                .duration_since(meeting.start_time)
                .unwrap_or(Duration::from_secs(0));

            let summary = MeetingSummary {
                meeting_id: meeting.id.clone(),
                name: meeting.name.clone(),
                duration_seconds: duration.as_secs(),
                total_segments: meeting.transcript_segments.len(),
                participants: meeting.participants.clone(),
                start_time: meeting.start_time,
                end_time: meeting.end_time.unwrap(),
            };

            // Cancel any background tasks for this meeting
            let mut handles = self.task_handles.lock().await;
            if let Some(handle) = handles.remove(meeting_id) {
                handle.abort();
            }

            // Save transcript to disk
            match self.transcript_storage.save_transcript(&meeting) {
                Err(e) => {
                    log::error!("Failed to save transcript for meeting {}: {}", meeting.name, e);
                }
                Ok(meeting_dir) => {
                    log::info!("Transcript saved for meeting: {}", meeting.name);
                    // Generate a lightweight summary.md similar to Zoom meeting summary
                    if !meeting.transcript_segments.is_empty() {
                        let start_idx = 0usize;
                        let end_idx = meeting.transcript_segments.len().saturating_sub(1);
                        let summary = crate::summarization::agent::summarize_segments_with_context(
                            meeting.project_path.as_deref(),
                            &meeting.transcript_segments,
                            start_idx,
                            end_idx,
                        );
                        let mut md = String::new();
                        use std::fmt::Write as _;
                        let _ = writeln!(md, "# Meeting Summary\n");
                        let minutes = duration.as_secs() / 60;
                        let _ = writeln!(md, "**Title**: {}", meeting.name);
                        let _ = writeln!(md, "**Duration**: {} minutes\n", minutes);
                        if !summary.new_features.is_empty() || !summary.new_features_structured.is_empty() {
                            let _ = writeln!(md, "## Key Points / Features");
                            if !summary.new_features_structured.is_empty() {
                                for f in &summary.new_features_structured {
                                    let _ = writeln!(md, "- {}", f.title);
                                }
                            } else {
                                for s in &summary.new_features { let _ = writeln!(md, "- {}", s); }
                            }
                            let _ = writeln!(md);
                        }
                        if !summary.technical_decisions.is_empty() {
                            let _ = writeln!(md, "## Decisions");
                            for s in &summary.technical_decisions { let _ = writeln!(md, "- {}", s); }
                            let _ = writeln!(md);
                        }
                        if !summary.questions.is_empty() {
                            let _ = writeln!(md, "## Open Questions");
                            for s in &summary.questions { let _ = writeln!(md, "- {}", s); }
                            let _ = writeln!(md);
                        }
                        // Save summary.md alongside transcript
                        let summary_path = meeting_dir.join("summary.md");
                        if let Err(e) = std::fs::write(&summary_path, md) {
                            log::warn!("Failed to write summary.md: {}", e);
                        }
                    }
                }
            }

            log::info!(
                "Ended meeting: {} - Duration: {}s, Segments: {}",
                meeting.name,
                duration.as_secs(),
                meeting.transcript_segments.len()
            );

            Ok(summary)
        } else {
            Err(anyhow::anyhow!("Meeting not found: {}", meeting_id))
        }
    }

    /// Add a transcript segment to a meeting
    pub async fn add_segment(&self, meeting_id: &str, segment: TranscriptSegment) -> Result<()> {
        let mut meetings = self.active_meetings.lock().await;

        if let Some(meeting) = meetings.get_mut(meeting_id) {
            // Track new speakers
            if !meeting.participants.contains(&segment.speaker) {
                meeting.participants.push(segment.speaker.clone());
            }

            meeting.transcript_segments.push(segment);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Meeting not found: {}", meeting_id))
        }
    }

    /// Get the live transcript for an active meeting
    pub async fn get_live_transcript(&self, meeting_id: &str) -> Result<Vec<TranscriptSegment>> {
        let meetings = self.active_meetings.lock().await;

        if let Some(meeting) = meetings.get(meeting_id) {
            Ok(meeting.transcript_segments.clone())
        } else {
            Err(anyhow::anyhow!("Meeting not found: {}", meeting_id))
        }
    }

    /// Get the complete meeting session data
    pub async fn get_meeting(&self, meeting_id: &str) -> Result<MeetingSession> {
        let meetings = self.active_meetings.lock().await;

        if let Some(meeting) = meetings.get(meeting_id) {
            Ok(meeting.clone())
        } else {
            Err(anyhow::anyhow!("Meeting not found: {}", meeting_id))
        }
    }

    /// Update speaker labels in a meeting (e.g., rename "Speaker 1" to "John")
    pub async fn update_speaker_labels(
        &self,
        meeting_id: &str,
        mapping: HashMap<String, String>,
    ) -> Result<()> {
        let mut meetings = self.active_meetings.lock().await;

        if let Some(meeting) = meetings.get_mut(meeting_id) {
            // Update all segments
            for segment in &mut meeting.transcript_segments {
                if let Some(new_label) = mapping.get(&segment.speaker) {
                    segment.speaker = new_label.clone();
                }
            }

            // Update participants list
            meeting.participants = meeting.participants.iter()
                .map(|p| mapping.get(p).cloned().unwrap_or_else(|| p.clone()))
                .collect();

            log::info!("Updated speaker labels for meeting: {}", meeting_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Meeting not found: {}", meeting_id))
        }
    }

    /// Get list of all active meeting IDs
    pub async fn get_active_meetings(&self) -> Vec<String> {
        let meetings = self.active_meetings.lock().await;
        meetings.keys().cloned().collect()
    }

    /// Store a task handle for cleanup
    pub async fn register_task_handle(&self, meeting_id: String, handle: JoinHandle<()>) {
        let mut handles = self.task_handles.lock().await;
        handles.insert(meeting_id, handle);
    }

    /// Background transcription loop for a meeting
    ///
    /// Runs continuously, capturing 30-second audio chunks and transcribing them
    async fn transcription_loop(
        meeting_id: String,
        active_meetings: Arc<Mutex<HashMap<String, MeetingSession>>>,
        audio_manager: Arc<AudioRecordingManager>,
        transcription_manager: Arc<TranscriptionManager>,
        app_handle: AppHandle,
    ) {
        log::info!("Starting transcription loop for meeting: {}", meeting_id);

        let mut segment_index = 0;
        let mut last_sent_index: usize = 0;
        let mut last_update_instant = std::time::Instant::now();
        let mut accumulated_time: f64 = 0.0;
        // Append stats for SOAK instrumentation
        #[derive(Default, Clone)]
        struct AppendStats { updates_written: u64, max_append_ms: u128 }
        let stats = std::sync::Arc::new(tokio::sync::Mutex::new(AppendStats::default()));
        let mut last_soak_log = std::time::Instant::now();

        loop {
            // Re-read chunk duration each iteration for live setting updates
            let settings = settings::get_settings(&app_handle);
            let mut chunk_secs = settings.transcription_chunk_seconds as f32;
            if chunk_secs < 2.0 { chunk_secs = 2.0; }
            if chunk_secs > 60.0 { chunk_secs = 60.0; }
            let chunk_duration = chunk_secs; // seconds

            // Sleep for the chunk duration
            tokio::time::sleep(Duration::from_secs_f32(chunk_duration)).await;

            // Check if meeting still exists and is recording
            let should_continue = {
                let meetings = active_meetings.lock().await;
                if let Some(meeting) = meetings.get(&meeting_id) {
                    meeting.status == MeetingStatus::Recording
                } else {
                    false
                }
            };

            if !should_continue {
                log::info!("Transcription loop ending for meeting: {}", meeting_id);
                break;
            }

            // Get audio chunk from buffer
            let buffer_size = audio_manager.get_system_audio_buffer_size();
            log::info!("System audio buffer size before read: {} samples", buffer_size);

            let audio_chunk = audio_manager.get_system_audio_buffer(chunk_duration);

            if audio_chunk.is_empty() {
                log::warn!("No audio captured in this chunk (buffer was: {} samples), skipping transcription", buffer_size);
                continue;
            }

            // Check audio levels (RMS and peak)
            let sum_squares: f32 = audio_chunk.iter().map(|&x| x * x).sum();
            let rms = (sum_squares / audio_chunk.len() as f32).sqrt();
            let peak = audio_chunk.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);

            log::info!("Audio chunk stats - samples: {}, RMS: {:.6}, Peak: {:.6}", audio_chunk.len(), rms, peak);

            if rms < 0.0001 {
                log::warn!("Audio RMS is very low ({:.6}), audio may be silent", rms);
            }

            // Transcribe the audio chunk (blocking operation)
            let transcription_result = tokio::task::spawn_blocking({
                let transcription_manager = transcription_manager.clone();
                let audio_chunk = audio_chunk.clone();
                move || transcription_manager.transcribe(audio_chunk)
            }).await;

            let text = match transcription_result {
                Ok(Ok(transcribed_text)) => transcribed_text,
                Ok(Err(e)) => {
                    log::error!("Transcription error: {}", e);
                    continue;
                }
                Err(e) => {
                    log::error!("Task join error: {}", e);
                    continue;
                }
            };

            log::info!("Transcription result: '{}' (length: {} chars)", text, text.len());

            // Skip empty transcriptions
            if text.trim().is_empty() {
                log::warn!("Empty transcription returned from model, skipping segment {}", segment_index);
                continue;
            }

            // Calculate segment timing
            let start_time = accumulated_time;
            let end_time = start_time + chunk_duration as f64;

            // Create transcript segment
            let speaker_label = format!("Speaker {}", (segment_index % 2) + 1); // Placeholder speaker detection
            let segment = TranscriptSegment {
                speaker: speaker_label.clone(),
                start_time,
                end_time,
                text: text.clone(),
                confidence: 0.95, // Placeholder confidence
                timestamp: SystemTime::now(),
            };

            // Add segment to meeting and capture project path for transcript write
            let mut project_path_for_segment: Option<String> = None;
            {
                let mut meetings = active_meetings.lock().await;
                if let Some(meeting) = meetings.get_mut(&meeting_id) {
                    // Track new speakers
                    if !meeting.participants.contains(&segment.speaker) {
                        meeting.participants.push(segment.speaker.clone());
                    }
                    meeting.transcript_segments.push(segment.clone());
                    project_path_for_segment = meeting.project_path.clone();

                    log::info!("Added segment {} to meeting: {}", segment_index, meeting_id);
                } else {
                    log::warn!("Meeting not found while adding segment: {}", meeting_id);
                    break;
                }
            }

            // Emit event to frontend
            #[derive(Clone, Serialize)]
            struct SegmentAddedPayload {
                meeting_id: String,
                segment: TranscriptSegment,
            }

            let _ = app_handle.emit("transcript-segment-added", SegmentAddedPayload {
                meeting_id: meeting_id.clone(),
                segment,
            });

            // Append rolling transcript line in project folder (non-blocking)
            if let Some(pp) = project_path_for_segment.clone() {
                // Build a fresh segment (avoid borrowing moved values)
                let seg_clone = TranscriptSegment {
                    speaker: speaker_label,
                    start_time,
                    end_time,
                    text: text.clone(),
                    confidence: 0.95,
                    timestamp: SystemTime::now(),
                };
                let meeting_id_clone = meeting_id.clone();
                let idx = segment_index;
                tokio::task::spawn_blocking(move || {
                    if let Err(e) = crate::meeting::transcript_writer::append_segment(&pp, &meeting_id_clone, idx, &seg_clone) {
                        log::warn!("Failed to append transcript segment: {}", e);
                    }
                });
            }

            segment_index += 1;
            accumulated_time = end_time;

            // Append meeting update on configured interval
            let settings_now = settings::get_settings(&app_handle);
            let interval_secs = settings_now.meeting_update_interval_seconds.clamp(5, 300);
            let should_append_update = last_update_instant.elapsed() >= Duration::from_secs(interval_secs as u64);
            if should_append_update {
                let (project_path, segments_snapshot) = {
                    let meetings = active_meetings.lock().await;
                    if let Some(m) = meetings.get(&meeting_id) {
                        (m.project_path.clone(), m.transcript_segments.clone())
                    } else {
                        (None, Vec::new())
                    }
                };

                if let Some(project_path) = project_path {
                    if segments_snapshot.len() > last_sent_index {
                        let new_segments = &segments_snapshot[last_sent_index..];
                        let start_idx = last_sent_index;
                        let end_idx = segments_snapshot.len().saturating_sub(1);

                        let project_path_clone = project_path.clone();
                        let summary = crate::summarization::agent::summarize_segments_with_context(
                            Some(project_path_clone.as_str()),
                            new_segments,
                            start_idx,
                            end_idx,
                        );

                        // Determine source label
                        let source_label = match audio_manager.get_audio_source() {
                            crate::managers::audio::AudioSource::Microphone => "microphone".to_string(),
                            crate::managers::audio::AudioSource::SystemAudio(_) => "system_audio".to_string(),
                        };
                        let meeting_name = {
                            let meetings = active_meetings.lock().await;
                            meetings.get(&meeting_id).map(|m| m.name.clone()).unwrap_or_default()
                        };
                        let current_model = settings_now.selected_model.clone();

                        // Offload append to a spawned task with retries and latency tracking
                        let app_handle_clone = app_handle.clone();
                        let meeting_id_clone = meeting_id.clone();
                        let stats_clone = stats.clone();
                        let project_path_owned = project_path.clone();
                        let meeting_name_owned = meeting_name.clone();
                        let current_model_owned = current_model.clone();
                        let source_label_owned = source_label.clone();
                        let summary_owned = summary.clone();
                        tokio::spawn(async move {
                            let started = std::time::Instant::now();
                            let mut attempt: u32 = 0;
                            let mut last_err: Option<anyhow::Error> = None;
                            let update_id_opt: Option<u32>;
                            loop {
                                attempt += 1;
                                match crate::meeting::context_writer::append_update(
                                    &project_path_owned,
                                    &meeting_id_clone,
                                    &meeting_name_owned,
                                    &current_model_owned,
                                    &source_label_owned,
                                    &summary_owned,
                                ) {
                                    Ok(update_id) => {
                                        update_id_opt = Some(update_id);
                                        break;
                                    }
                                    Err(e) => {
                                        last_err = Some(e);
                                        if attempt >= 3 { update_id_opt = None; break; }
                                        let backoff_ms = match attempt { 1 => 120, 2 => 360, _ => 750 };
                                        log::warn!(
                                            "Append update failed (attempt {}), retrying in {}ms",
                                            attempt, backoff_ms
                                        );
                                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                                    }
                                }
                            }

                            let elapsed_ms = started.elapsed().as_millis();
                            {
                                let mut s = stats_clone.lock().await;
                                if let Some(_id) = update_id_opt {
                                    s.updates_written += 1;
                                }
                                if elapsed_ms > s.max_append_ms { s.max_append_ms = elapsed_ms; }
                            }

                            if let Some(update_id) = update_id_opt {
                                #[derive(Clone, Serialize)]
                                struct UpdatePayload { update_id: u32, meeting_id: String }
                                let _ = app_handle_clone.emit(
                                    "meeting-update-appended",
                                    UpdatePayload { update_id, meeting_id: meeting_id_clone.clone() },
                                );
                                log::info!(
                                    "Appended meeting update {} for segments [{}..={}] in {}ms",
                                    update_id, start_idx, end_idx, elapsed_ms
                                );
                                // Attempt auto-trigger if enabled
                                if let Some(pp) = Some(project_path_owned.clone()) {
                                    let _ = crate::automation::claude_trigger::trigger_meeting_update(
                                        &app_handle_clone,
                                        &pp,
                                        &meeting_id_clone,
                                        update_id,
                                    );
                                }
                            } else if let Some(err) = last_err {
                                log::error!("Failed to append meeting update after retries: {}", err);
                            }
                        });

                        // Locally advance pointers without blocking on append
                        last_sent_index = segments_snapshot.len();
                        last_update_instant = std::time::Instant::now();
                    }
                }
            }

            // Periodic SOAK instrumentation logging (every ~5 minutes)
            if last_soak_log.elapsed() >= Duration::from_secs(300) {
                let (project_path_opt, s) = {
                    let s = stats.lock().await.clone();
                    let meetings = active_meetings.lock().await;
                    (meetings.get(&meeting_id).and_then(|m| m.project_path.clone()), s)
                };
                let mut size_bytes = 0u64;
                if let Some(pp) = project_path_opt {
                    let path = std::path::Path::new(&pp).join(".meeting-updates.jsonl");
                    if let Ok(md) = std::fs::metadata(path) { size_bytes = md.len(); }
                }
                log::info!(
                    "SOAK updates_written={} jsonl_size_bytes={} max_append_latency_ms={}",
                    s.updates_written,
                    size_bytes,
                    s.max_append_ms
                );
                last_soak_log = std::time::Instant::now();
            }
        }

        log::info!("Transcription loop ended for meeting: {}", meeting_id);
    }

    /// Shutdown manager and cancel all active tasks
    pub async fn shutdown(&self) -> Result<()> {
        let mut handles = self.task_handles.lock().await;
        for (_, handle) in handles.drain() {
            handle.abort();
        }
        log::info!("MeetingManager shutdown complete");
        Ok(())
    }
}

// Note: Default implementation removed as MeetingManager requires
// audio_manager, transcription_manager, and app_handle parameters

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests temporarily disabled pending integration test setup
    // that provides audio_manager, transcription_manager, and app_handle

    #[tokio::test]
    #[ignore]
    async fn test_meeting_lifecycle() {
        // TODO: Set up test harness with mock managers
        // let manager = MeetingManager::new(app_handle, audio_manager, transcription_manager).unwrap();

        // Start meeting
        let meeting_id = manager.start_meeting("Test Meeting".to_string()).await.unwrap();
        assert!(!meeting_id.is_empty());

        // Get meeting
        let meeting = manager.get_meeting(&meeting_id).await.unwrap();
        assert_eq!(meeting.name, "Test Meeting");
        assert_eq!(meeting.status, MeetingStatus::Recording);

        // Add segment
        manager.add_segment(&meeting_id, TranscriptSegment {
            speaker: "Speaker 1".to_string(),
            start_time: 0.0,
            end_time: 3.5,
            text: "Hello world".to_string(),
            confidence: 0.95,
            timestamp: SystemTime::now(),
        }).await.unwrap();

        // Get transcript
        let transcript = manager.get_live_transcript(&meeting_id).await.unwrap();
        assert_eq!(transcript.len(), 1);
        assert_eq!(transcript[0].text, "Hello world");

        // End meeting
        let summary = manager.end_meeting(&meeting_id).await.unwrap();
        assert_eq!(summary.total_segments, 1);
        assert_eq!(summary.participants.len(), 1);
    }

    #[tokio::test]
    #[ignore]
    async fn test_pause_resume() {
        // TODO: Set up test harness with mock managers
        return; // Temporarily disabled
        // let manager = MeetingManager::new(app_handle, audio_manager, transcription_manager).unwrap();
        let meeting_id = manager.start_meeting("Test".to_string()).await.unwrap();

        // Pause
        manager.pause_meeting(&meeting_id).await.unwrap();
        let meeting = manager.get_meeting(&meeting_id).await.unwrap();
        assert_eq!(meeting.status, MeetingStatus::Paused);

        // Resume
        manager.resume_meeting(&meeting_id).await.unwrap();
        let meeting = manager.get_meeting(&meeting_id).await.unwrap();
        assert_eq!(meeting.status, MeetingStatus::Recording);

        manager.end_meeting(&meeting_id).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_speaker_label_update() {
        // TODO: Set up test harness with mock managers
        return; // Temporarily disabled
        // let manager = MeetingManager::new(app_handle, audio_manager, transcription_manager).unwrap();
        let meeting_id = manager.start_meeting("Test".to_string()).await.unwrap();

        // Add segments with generic labels
        manager.add_segment(&meeting_id, TranscriptSegment {
            speaker: "Speaker 1".to_string(),
            start_time: 0.0,
            end_time: 2.0,
            text: "First".to_string(),
            confidence: 0.9,
            timestamp: SystemTime::now(),
        }).await.unwrap();

        manager.add_segment(&meeting_id, TranscriptSegment {
            speaker: "Speaker 2".to_string(),
            start_time: 2.0,
            end_time: 4.0,
            text: "Second".to_string(),
            confidence: 0.9,
            timestamp: SystemTime::now(),
        }).await.unwrap();

        // Update labels
        let mut mapping = HashMap::new();
        mapping.insert("Speaker 1".to_string(), "Alice".to_string());
        mapping.insert("Speaker 2".to_string(), "Bob".to_string());

        manager.update_speaker_labels(&meeting_id, mapping).await.unwrap();

        // Verify updates
        let transcript = manager.get_live_transcript(&meeting_id).await.unwrap();
        assert_eq!(transcript[0].speaker, "Alice");
        assert_eq!(transcript[1].speaker, "Bob");

        let meeting = manager.get_meeting(&meeting_id).await.unwrap();
        assert!(meeting.participants.contains(&"Alice".to_string()));
        assert!(meeting.participants.contains(&"Bob".to_string()));

        manager.end_meeting(&meeting_id).await.unwrap();
    }
}
