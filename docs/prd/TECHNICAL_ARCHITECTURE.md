# Technical Architecture

## System Overview

MeetingCoder is built as a desktop application using Tauri, combining a Rust backend for performance-critical operations with a React/TypeScript frontend for the user interface. The architecture inherits core components from the Handy speech-to-text application while adding LLM integration and code generation capabilities.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (React)                        │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐   │
│  │   Meeting    │  │     Code     │  │   Settings &       │   │
│  │     UI       │  │   Preview    │  │   Configuration    │   │
│  └──────────────┘  └──────────────┘  └────────────────────┘   │
└────────────────────────────┬────────────────────────────────────┘
                             │ Tauri Commands / Events
┌────────────────────────────┴────────────────────────────────────┐
│                      Backend (Rust)                             │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                     Manager Layer                         │  │
│  │  ┌────────────┐  ┌──────────────┐  ┌────────────────┐   │  │
│  │  │   Audio    │  │ Transcription│  │   Meeting      │   │  │
│  │  │  Manager   │  │   Manager    │  │   Manager      │   │  │
│  │  └────────────┘  └──────────────┘  └────────────────┘   │  │
│  │                                                            │  │
│  │  ┌────────────┐  ┌──────────────┐  ┌────────────────┐   │  │
│  │  │    LLM     │  │   CodeGen    │  │   Project      │   │  │
│  │  │  Manager   │  │   Manager    │  │   Manager      │   │  │
│  │  └────────────┘  └──────────────┘  └────────────────┘   │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Core Services                          │  │
│  │  ┌────────────┐  ┌──────────────┐  ┌────────────────┐   │  │
│  │  │  System    │  │     VAD      │  │   Streaming    │   │  │
│  │  │   Audio    │  │  (Silero)    │  │ Orchestrator   │   │  │
│  │  └────────────┘  └──────────────┘  └────────────────┘   │  │
│  │                                                            │  │
│  │  ┌────────────┐  ┌──────────────┐  ┌────────────────┐   │  │
│  │  │  Whisper   │  │   File I/O   │  │    Preview     │   │  │
│  │  │  Engine    │  │   Service    │  │    Server      │   │  │
│  │  └────────────┘  └──────────────┘  └────────────────┘   │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
    ┌────▼─────┐      ┌──────▼──────┐    ┌──────▼──────┐
    │  Claude  │      │   OpenAI    │    │   Ollama    │
    │   API    │      │     API     │    │  (Local)    │
    └──────────┘      └─────────────┘    └─────────────┘
```

## Component Breakdown

### Frontend Components (React/TypeScript)

#### 1. Meeting UI (`src/components/meeting/`)
- **MeetingControls.tsx**: Start/stop meeting, status indicators
- **LiveTranscript.tsx**: Real-time transcript display with speaker labels
- **InsightsPanel.tsx**: AI-generated suggestions and warnings
- **MetadataForm.tsx**: Meeting name, participants, project type selection

#### 2. Code Preview (`src/components/preview/`)
- **FileTree.tsx**: Navigable tree of generated files
- **CodeEditor.tsx**: Monaco editor integration for inline editing
- **LivePreview.tsx**: Embedded browser for web app preview
- **DiffView.tsx**: Side-by-side comparison for updates

#### 3. Requirements Review (`src/components/requirements/`)
- **RequirementsList.tsx**: Extracted features with priorities
- **EditRequirement.tsx**: Inline editing of requirements
- **ApprovalFlow.tsx**: Review and approve before generation

#### 4. Settings (`src/components/settings/`)
- **LLMConfig.tsx**: Provider selection, API key management
- **AudioSettings.tsx**: System audio source selection
- **GenerationSettings.tsx**: Language, framework, model preferences

### Backend Managers (Rust)

#### 1. AudioRecordingManager (`src-tauri/src/managers/audio.rs`)
**Responsibilities**:
- Continuous audio capture from system audio
- Voice Activity Detection (VAD) to filter silence
- Audio buffering and chunking (30-60s segments)
- Device management and hot-swapping

**Key APIs**:
```rust
pub struct AudioRecordingManager {
    recorder: Arc<Mutex<Option<AudioRecorder>>>,
    mode: Arc<Mutex<MicrophoneMode>>, // AlwaysOn for meetings
    vad: Box<dyn VoiceActivityDetector>,
}

impl AudioRecordingManager {
    pub fn start_system_audio_capture(&self) -> Result<()>;
    pub fn get_audio_chunk(&self, duration: Duration) -> Result<Vec<f32>>;
    pub fn on_audio_level_changed(&self, callback: impl Fn(f32));
}
```

**Changes from Handy**:
- Switch from microphone-only to system audio capture
- Support continuous mode (not just push-to-talk)
- Integrate platform-specific system audio APIs

#### 2. TranscriptionManager (`src-tauri/src/managers/transcription.rs`)
**Responsibilities**:
- Queue audio chunks for transcription
- Run Whisper/Parakeet models
- Handle speaker diarization
- Emit transcript segments to frontend

**Key APIs**:
```rust
pub struct TranscriptionManager {
    engine: Arc<Mutex<Option<LoadedEngine>>>,
    queue: Arc<Mutex<VecDeque<AudioChunk>>>,
    diarizer: Option<SpeakerDiarizer>,
}

impl TranscriptionManager {
    pub async fn transcribe_chunk(
        &self,
        audio: Vec<f32>,
    ) -> Result<TranscriptSegment>;

    pub async fn transcribe_with_speakers(
        &self,
        audio: Vec<f32>,
    ) -> Result<Vec<SpeakerSegment>>;
}
```

**Changes from Handy**:
- Add queuing system for continuous chunks
- Integrate speaker diarization
- Return structured segments (not just text)

#### 3. MeetingManager (`src-tauri/src/managers/meeting.rs`) [NEW]
**Responsibilities**:
- Orchestrate meeting lifecycle (start, pause, end)
- Maintain meeting state and metadata
- Coordinate between audio, transcription, and code generation
- Save meeting artifacts (transcript, requirements, code)

**Key APIs**:
```rust
pub struct MeetingManager {
    active_meeting: Arc<Mutex<Option<MeetingSession>>>,
    storage: TranscriptStorage,
}

pub struct MeetingSession {
    pub id: String,
    pub start_time: SystemTime,
    pub transcript: Vec<TranscriptSegment>,
    pub requirements: Option<Requirements>,
    pub project_path: Option<PathBuf>,
    pub status: MeetingStatus,
}

impl MeetingManager {
    pub fn start_meeting(&mut self, metadata: MeetingMetadata) -> Result<String>;
    pub fn add_transcript_segment(&mut self, segment: TranscriptSegment);
    pub fn end_meeting(&mut self) -> Result<MeetingSummary>;
    pub fn get_current_transcript(&self) -> Vec<TranscriptSegment>;
}
```

#### 4. LLMManager (`src-tauri/src/managers/llm.rs`) [NEW]
**Responsibilities**:
- Abstract LLM provider differences
- Handle API authentication and rate limiting
- Manage prompt templates
- Stream responses for real-time updates

**Key APIs**:
```rust
pub trait LLMProvider: Send + Sync {
    async fn generate(&self, request: GenerationRequest) -> Result<String>;
    async fn generate_streaming(
        &self,
        request: GenerationRequest,
    ) -> Result<impl Stream<Item = String>>;
}

pub struct LLMManager {
    providers: HashMap<ProviderType, Box<dyn LLMProvider>>,
    active_provider: ProviderType,
}

impl LLMManager {
    pub async fn generate_requirements(
        &self,
        transcript: &str,
    ) -> Result<Requirements>;

    pub async fn generate_code(
        &self,
        requirements: &Requirements,
    ) -> Result<GeneratedProject>;

    pub async fn generate_incremental_update(
        &self,
        delta: RequirementsDelta,
        context: &ProjectContext,
    ) -> Result<Vec<FileUpdate>>;
}
```

#### 5. CodeGenManager (`src-tauri/src/managers/codegen.rs`) [NEW]
**Responsibilities**:
- Orchestrate code generation pipeline
- Validate generated code (syntax, security)
- Apply templates and best practices
- Handle incremental updates

**Key APIs**:
```rust
pub struct CodeGenManager {
    llm: Arc<LLMManager>,
    validator: CodeValidator,
    templates: TemplateRegistry,
}

impl CodeGenManager {
    pub async fn generate_initial_project(
        &self,
        requirements: &Requirements,
    ) -> Result<GeneratedProject>;

    pub async fn apply_incremental_update(
        &self,
        project: &mut ProjectContext,
        delta: RequirementsDelta,
    ) -> Result<UpdateResult>;

    pub fn validate_code(&self, code: &str, language: Language) -> ValidationResult;
}
```

#### 6. ProjectManager (`src-tauri/src/managers/project.rs`) [NEW]
**Responsibilities**:
- Manage project file structure
- Write/update files on disk
- Track project context and history
- Integration with preview server

**Key APIs**:
```rust
pub struct ProjectManager {
    projects_root: PathBuf,
    active_projects: HashMap<String, ProjectContext>,
}

pub struct ProjectContext {
    pub path: PathBuf,
    pub manifest: FileManifest,
    pub requirements: Requirements,
    pub generation_history: Vec<GenerationEvent>,
}

impl ProjectManager {
    pub fn create_project(
        &mut self,
        name: &str,
        files: Vec<GeneratedFile>,
    ) -> Result<PathBuf>;

    pub fn update_project(
        &mut self,
        project_id: &str,
        updates: Vec<FileUpdate>,
    ) -> Result<()>;

    pub fn get_project_context(&self, project_id: &str) -> Option<&ProjectContext>;
}
```

### Core Services (Rust)

#### 1. System Audio Capture (`src-tauri/src/system_audio/`)

**Platform-Specific Implementations**:

**macOS** (`system_audio/macos.rs`):
```rust
use coreaudio::audio_unit::{AudioUnit, Scope, Element};

pub struct MacOSSystemAudio {
    audio_unit: AudioUnit,
    buffer: Arc<Mutex<RingBuffer<f32>>>,
}

impl MacOSSystemAudio {
    pub fn new(device_name: &str) -> Result<Self>;
    pub fn start_capture(&mut self) -> Result<()>;
    pub fn read_samples(&self, count: usize) -> Vec<f32>;
}
```

**Windows** (`system_audio/windows.rs`):
```rust
use windows::Win32::Media::Audio::{IAudioClient, IAudioCaptureClient};

pub struct WindowsSystemAudio {
    audio_client: IAudioClient,
    capture_client: IAudioCaptureClient,
    buffer: Arc<Mutex<RingBuffer<f32>>>,
}

impl WindowsSystemAudio {
    pub fn new_loopback() -> Result<Self>;
    pub fn start_capture(&mut self) -> Result<()>;
    pub fn read_samples(&self, count: usize) -> Vec<f32>;
}
```

**Linux** (`system_audio/linux.rs`):
```rust
use libpulse_binding::stream::Stream;

pub struct LinuxSystemAudio {
    stream: Stream,
    buffer: Arc<Mutex<RingBuffer<f32>>>,
}

impl LinuxSystemAudio {
    pub fn new_monitor() -> Result<Self>;
    pub fn start_capture(&mut self) -> Result<()>;
    pub fn read_samples(&self, count: usize) -> Vec<f32>;
}
```

**Unified Interface**:
```rust
pub trait SystemAudioCapture: Send + Sync {
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn read_samples(&self, count: usize) -> Vec<f32>;
    fn get_device_list() -> Vec<AudioDevice>;
}

// Platform-specific constructor
pub fn create_system_audio_capture() -> Result<Box<dyn SystemAudioCapture>> {
    #[cfg(target_os = "macos")]
    return Ok(Box::new(MacOSSystemAudio::new()?));

    #[cfg(target_os = "windows")]
    return Ok(Box::new(WindowsSystemAudio::new_loopback()?));

    #[cfg(target_os = "linux")]
    return Ok(Box::new(LinuxSystemAudio::new_monitor()?));
}
```

#### 2. Speaker Diarization (`src-tauri/src/diarization.rs`)

**Implementation Options**:

**Option A: Whisper Built-in** (Simplest)
```rust
// Whisper v3+ includes diarization
pub struct WhisperDiarizer {
    whisper: WhisperEngine,
}

impl WhisperDiarizer {
    pub fn transcribe_with_speakers(
        &self,
        audio: &[f32],
    ) -> Result<Vec<SpeakerSegment>> {
        // Use Whisper's built-in diarization
        let result = self.whisper.transcribe_with_diarization(audio)?;
        Ok(result)
    }
}
```

**Option B: Pyannote Audio** (Most Accurate)
```rust
// Call Python pyannote-audio via FFI or subprocess
pub struct PyannoteDiarizer {
    python_path: PathBuf,
}

impl PyannoteDiarizer {
    pub fn identify_speakers(
        &self,
        audio_path: &Path,
    ) -> Result<Vec<SpeakerTimestamp>> {
        let output = Command::new(&self.python_path)
            .arg("-m")
            .arg("pyannote.audio")
            .arg("diarize")
            .arg(audio_path)
            .output()?;

        parse_diarization_output(&output.stdout)
    }
}
```

**Option C: Simple Heuristics** (Fallback)
```rust
pub struct HeuristicDiarizer {
    threshold: f32,
}

impl HeuristicDiarizer {
    pub fn identify_speakers(&self, audio: &[f32]) -> Vec<SpeakerSegment> {
        // Use audio level + turn-taking patterns
        // - High energy = likely active speaker
        // - Silence gap = speaker change
        let segments = split_by_silence(audio);
        assign_speaker_labels(segments)
    }
}
```

#### 3. Streaming Orchestrator (`src-tauri/src/streaming/orchestrator.rs`)

**Core Logic**:
```rust
pub struct StreamingOrchestrator {
    transcript_buffer: Vec<TranscriptSegment>,
    generation_state: GenerationState,
    llm_manager: Arc<LLMManager>,
    codegen_manager: Arc<CodeGenManager>,
    project_manager: Arc<ProjectManager>,
    last_update_time: Instant,
    project_id: Option<String>,
}

impl StreamingOrchestrator {
    pub async fn process_new_segment(&mut self, segment: TranscriptSegment) {
        self.transcript_buffer.push(segment);

        match &self.generation_state {
            GenerationState::Idle => {
                if self.should_start_generation() {
                    self.start_initial_generation().await;
                }
            }
            GenerationState::IterativeUpdating => {
                if self.should_trigger_update() {
                    self.apply_update().await;
                }
            }
            _ => {} // Wait for current operation
        }
    }

    fn should_start_generation(&self) -> bool {
        // Heuristics to detect sufficient context:
        // 1. At least 2 minutes of conversation
        // 2. Keywords indicate requirements (e.g., "should", "need", "feature")
        // 3. Project type mentioned (e.g., "website", "app", "API")

        let duration_seconds = self.get_transcript_duration();
        let has_keywords = self.contains_requirement_keywords();
        let has_project_type = self.detect_project_type().is_some();

        duration_seconds > 120 && has_keywords && has_project_type
    }

    fn should_trigger_update(&self) -> bool {
        // Trigger update if:
        // 1. New content since last update (30+ seconds)
        // 2. New feature mentioned
        // 3. Clarification provided

        let time_since_update = self.last_update_time.elapsed();
        time_since_update > Duration::from_secs(30) &&
        self.has_new_content()
    }

    async fn start_initial_generation(&mut self) {
        self.generation_state = GenerationState::GeneratingInitial;

        // Extract requirements
        let transcript_text = self.get_full_transcript();
        let requirements = self.llm_manager
            .generate_requirements(&transcript_text)
            .await
            .expect("Failed to extract requirements");

        // Emit to frontend for review
        emit_requirements_for_review(&requirements);

        // Wait for user approval (or timeout after 2 mins)
        // ...

        // Generate code
        let project = self.codegen_manager
            .generate_initial_project(&requirements)
            .await
            .expect("Failed to generate code");

        // Write to disk
        let project_path = self.project_manager
            .create_project(&requirements.project_name, project.files)
            .expect("Failed to write project");

        self.project_id = Some(project_path.to_string_lossy().to_string());
        self.generation_state = GenerationState::IterativeUpdating;
        self.last_update_time = Instant::now();
    }

    async fn apply_update(&mut self) {
        let new_segments = self.get_new_segments();
        let project_id = self.project_id.as_ref().unwrap();
        let context = self.project_manager
            .get_project_context(project_id)
            .unwrap();

        // Calculate what changed
        let delta = calculate_requirements_delta(
            &context.requirements,
            &new_segments,
        );

        if delta.is_empty() {
            return; // No meaningful changes
        }

        // Generate updates
        let updates = self.codegen_manager
            .apply_incremental_update(context, delta)
            .await
            .expect("Failed to generate updates");

        // Apply to disk
        self.project_manager
            .update_project(project_id, updates)
            .expect("Failed to apply updates");

        self.last_update_time = Instant::now();
    }
}
```

#### 4. Preview Server (`src-tauri/src/preview/server.rs`)

**Implementation**:
```rust
use std::process::{Child, Command};
use std::net::TcpListener;

pub struct PreviewServer {
    port: u16,
    project_path: PathBuf,
    process: Option<Child>,
    server_type: ServerType,
}

enum ServerType {
    Vite,         // For React/Vue projects
    WebpackDev,   // Alternative bundler
    NodeWatch,    // For Node.js APIs
    StaticHTTP,   // For plain HTML
}

impl PreviewServer {
    pub async fn start(&mut self) -> Result<String> {
        // Find available port
        self.port = self.find_available_port()?;

        // Detect project type
        let project_type = self.detect_project_type()?;

        // Start appropriate server
        self.server_type = match project_type {
            ProjectType::React | ProjectType::Vue => ServerType::Vite,
            ProjectType::NodeAPI => ServerType::NodeWatch,
            ProjectType::StaticHTML => ServerType::StaticHTTP,
            _ => return Err("Unsupported project type for preview"),
        };

        let process = self.start_dev_server()?;
        self.process = Some(process);

        // Wait for server to be ready
        self.wait_for_ready().await?;

        Ok(format!("http://localhost:{}", self.port))
    }

    fn start_dev_server(&self) -> Result<Child> {
        match self.server_type {
            ServerType::Vite => {
                Command::new("npm")
                    .args(&["run", "dev", "--", "--port", &self.port.to_string()])
                    .current_dir(&self.project_path)
                    .spawn()
            }
            ServerType::NodeWatch => {
                Command::new("node")
                    .args(&["--watch", "src/index.js"])
                    .env("PORT", self.port.to_string())
                    .current_dir(&self.project_path)
                    .spawn()
            }
            ServerType::StaticHTTP => {
                // Use simple HTTP server
                Command::new("python3")
                    .args(&["-m", "http.server", &self.port.to_string()])
                    .current_dir(&self.project_path)
                    .spawn()
            }
            _ => Err(anyhow::anyhow!("Unsupported server type")),
        }
    }

    async fn wait_for_ready(&self) -> Result<()> {
        let url = format!("http://localhost:{}", self.port);
        let client = reqwest::Client::new();

        for _ in 0..30 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if client.get(&url).send().await.is_ok() {
                return Ok(());
            }
        }

        Err(anyhow::anyhow!("Server failed to start"))
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            process.kill()?;
        }
        Ok(())
    }
}
```

## Data Models

### Transcript Segment
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub speaker: SpeakerId,
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
    pub confidence: f32,
    pub timestamp: SystemTime,
}

pub type SpeakerId = String; // "Speaker 1", "Speaker 2", etc.
```

### Requirements
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Requirements {
    pub project_name: String,
    pub project_type: ProjectType,
    pub tech_stack: Vec<String>,
    pub features: Vec<Feature>,
    pub constraints: Vec<String>,
    pub questions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub mentioned_by: SpeakerId,
    pub timestamp: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectType {
    ReactWebApp,
    NodeAPI,
    PythonCLI,
    FullStackApp,
    StaticWebsite,
    Other(String),
}
```

### Generated Project
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedProject {
    pub files: Vec<GeneratedFile>,
    pub metadata: ProjectMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub content: String,
    pub language: Language,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub description: String,
    pub dependencies: Vec<Dependency>,
    pub setup_instructions: String,
}
```

## Communication Patterns

### Tauri Commands (Frontend → Backend)
```rust
#[tauri::command]
async fn start_meeting(metadata: MeetingMetadata) -> Result<String, String>;

#[tauri::command]
async fn end_meeting(meeting_id: String) -> Result<MeetingSummary, String>;

#[tauri::command]
async fn get_live_transcript(meeting_id: String) -> Result<Vec<TranscriptSegment>, String>;

#[tauri::command]
async fn approve_requirements(
    meeting_id: String,
    requirements: Requirements,
) -> Result<(), String>;

#[tauri::command]
async fn start_code_preview(project_path: String) -> Result<String, String>;
```

### Tauri Events (Backend → Frontend)
```rust
// Emit transcript segment
app.emit("transcript-segment", segment)?;

// Emit requirements extracted
app.emit("requirements-ready", requirements)?;

// Emit code generation progress
app.emit("generation-progress", progress)?;

// Emit code updated
app.emit("code-updated", file_updates)?;

// Emit insight
app.emit("meeting-insight", insight)?;
```

## Performance Considerations

### Memory Management
- **Ring Buffers**: For audio streaming (prevent unlimited growth)
- **Transcript Pruning**: Keep last 2 hours in memory, archive older
- **LLM Response Streaming**: Process incrementally (don't load full response)
- **Lazy Loading**: Load project files on-demand, not all at once

### Concurrency
- **Async I/O**: All network and file operations use Tokio async
- **Thread Pools**: CPU-intensive tasks (transcription, validation) use Rayon
- **Lock-Free Queues**: Audio and transcript queues use crossbeam channels

### Caching
- **LLM Responses**: Cache by prompt hash (avoid duplicate calls)
- **Validation Results**: Cache by file hash
- **Transcription**: No caching (always fresh)

## Security Considerations

### API Key Storage
- Use system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Never store in plain text or log files
- Encrypt in memory when not actively in use

### Generated Code Validation
- **Syntax Check**: Parse before writing
- **Security Patterns**: Flag eval(), exec(), hardcoded secrets
- **Dependency Check**: Verify against known vulnerability databases

### Network Security
- Use HTTPS for all API calls
- Validate SSL certificates
- Timeout all requests (prevent hanging)

## Deployment & Distribution

### Build Process
```bash
# macOS (Intel)
cargo tauri build --target x86_64-apple-darwin

# macOS (Apple Silicon)
cargo tauri build --target aarch64-apple-darwin

# Windows
cargo tauri build --target x86_64-pc-windows-msvc

# Linux
cargo tauri build --target x86_64-unknown-linux-gnu
```

### Auto-Update
- Use Tauri's built-in updater
- Sign releases with code signing certificates
- Host updates on GitHub Releases or CDN

### Installers
- **macOS**: DMG with code signing and notarization
- **Windows**: MSI or NSIS installer with code signing
- **Linux**: AppImage, .deb, and .rpm packages

## Testing Infrastructure

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_requirement_extraction() {
        let transcript = "We need a login form with email and password";
        let llm = MockLLMProvider::new();
        let manager = LLMManager::new(llm);

        let requirements = manager.generate_requirements(transcript).await.unwrap();

        assert_eq!(requirements.features.len(), 1);
        assert_eq!(requirements.features[0].title, "Login form");
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_end_to_end_meeting() {
    // Start meeting
    let meeting_id = meeting_manager.start_meeting(metadata).await.unwrap();

    // Simulate transcript segments
    for segment in mock_transcript {
        meeting_manager.add_segment(segment);
    }

    // End meeting
    let summary = meeting_manager.end_meeting(meeting_id).await.unwrap();

    // Verify code was generated
    assert!(summary.project_path.exists());
}
```

### Performance Benchmarks
```rust
#[bench]
fn bench_transcription_latency(b: &mut Bencher) {
    let audio = load_test_audio();
    let transcriber = TranscriptionManager::new();

    b.iter(|| {
        transcriber.transcribe_chunk(&audio)
    });
}
```

## Monitoring & Observability

### Logging
```rust
use log::{debug, info, warn, error};

info!("Meeting started: {}", meeting_id);
debug!("Transcript segment received: {} chars", segment.text.len());
warn!("LLM request took longer than expected: {}ms", latency);
error!("Failed to generate code: {}", err);
```

### Metrics
- **Latency**: Track time from speech → transcription → code
- **Success Rate**: Percentage of successful generations
- **Resource Usage**: Memory, CPU, network
- **User Actions**: Button clicks, feature usage

### Error Reporting
- Integrate Sentry or similar (opt-in)
- Capture stack traces and context
- Privacy-conscious (no transcript content in reports)

## Extensibility Points

### Plugin System (Future)
```rust
pub trait MeetingCoderPlugin {
    fn on_transcript_segment(&self, segment: &TranscriptSegment);
    fn on_requirements_extracted(&self, requirements: &Requirements);
    fn on_code_generated(&self, project: &GeneratedProject);
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn MeetingCoderPlugin>>,
}
```

### Custom Prompts
- Allow users to customize prompt templates
- Store in `~/.meetingcoder/prompts/`
- Hot-reload on change

### Custom Templates
- Project structure templates
- Code style preferences
- File naming conventions

## Dependencies Summary

### Rust Crates
```toml
[dependencies]
# Tauri framework
tauri = "2.0"
tauri-plugin-store = "2.0"

# Async runtime
tokio = { version = "1", features = ["full"] }

# Audio
cpal = "0.15"
hound = "3.5"
rubato = "0.15"

# Transcription
whisper-rs = "0.10"
transcribe-rs = "0.3"
vad-rs = "0.1"

# LLM integration
reqwest = { version = "0.11", features = ["json"] }
tokio-tungstenite = "0.21"  # WebSocket for streaming

# Security
keyring = "2.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Validation
tree-sitter = "0.20"
syntect = "5.0"

# Logging
log = "0.4"
env_logger = "0.11"

# Platform-specific
[target.'cfg(target_os = "macos")'.dependencies]
coreaudio-sys = "0.2"

[target.'cfg(target_os = "windows")'.dependencies]
windows = "0.52"

[target.'cfg(target_os = "linux")'.dependencies]
libpulse-binding = "2.28"
```

### Frontend Dependencies
```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "@tauri-apps/api": "^2.0.0",
    "@monaco-editor/react": "^4.6.0",
    "react-syntax-highlighter": "^15.5.0"
  },
  "devDependencies": {
    "typescript": "^5.2.0",
    "vite": "^5.0.0",
    "@vitejs/plugin-react": "^4.2.0"
  }
}
```
