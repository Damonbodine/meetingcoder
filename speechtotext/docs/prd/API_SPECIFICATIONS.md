# API Specifications

## Overview

This document defines all API contracts between components in MeetingCoder, including:
- Tauri command interfaces (Frontend ↔ Backend)
- Tauri event schemas (Backend → Frontend)
- Internal Rust API contracts
- LLM provider integrations (Backend → External APIs)

## Tauri Commands

Commands are invoked by the frontend to trigger backend operations.

### Meeting Management

#### `start_meeting`

Starts a new meeting session with metadata.

**Request**:
```typescript
interface MeetingMetadata {
  name?: string;           // Optional meeting name
  participants?: string[]; // Optional participant list
  projectType?: string;    // Optional hint for project type
}

// Invoke
const meetingId = await invoke<string>('start_meeting', {
  metadata: {
    name: "Stakeholder Call - Q1 Roadmap",
    participants: ["Me", "Sarah (PM)"],
    projectType: "web_app"
  }
});
```

**Response**:
```typescript
type Response = string; // Meeting ID (UUID)
```

**Errors**:
- `MeetingAlreadyActive`: Another meeting is in progress
- `AudioDeviceNotAvailable`: System audio not configured
- `TranscriptionModelNotLoaded`: Whisper model not ready

---

### Import

#### `import_audio_as_meeting`

Imports a local audio file as an offline meeting and returns a `MeetingSummary` once finished.

Request:
```typescript
const summary = await invoke<MeetingSummary>('import_audio_as_meeting', {
  meetingName: 'Customer Interview #3',
  filePath: '/path/to/audio.mp3',
});
```

Response:
```typescript
interface MeetingSummary {
  meeting_id: string;
  name: string;
  duration_seconds: number;
  total_segments: number;
  participants: string[];
  start_time: string; // ISO 8601
  end_time: string;   // ISO 8601
}
```

Errors:
- `File not found`
- `Unsupported file type`
- `Audio file is too large` (safety cap ~1.5GB)
- `Transcription model not loaded` (install/select a model)
- `Decode produced zero samples` (corrupt/unsupported content)

---

#### `import_youtube_as_meeting`

Downloads YouTube audio via `yt-dlp`, then imports it as an offline meeting.

Request:
```typescript
const summary = await invoke<MeetingSummary>('import_youtube_as_meeting', {
  meetingName: 'All Hands 2024-11-01',
  url: 'https://www.youtube.com/watch?v=...' ,
});
```

Response: `MeetingSummary` (same as above)

Errors:
- `yt-dlp not found` (provide install guidance)
- Network errors: `yt-dlp failed to download audio due to network issues`
- `Downloaded audio file not found`
- All errors from `import_audio_as_meeting` may also apply

Side effects:
- Appends `.meeting-updates.jsonl` using `source = "import:file"` or `"import:youtube"`
- May trigger `/meeting` automation if enabled in settings

---

#### `end_meeting`

Ends the active meeting and generates summary.

**Request**:
```typescript
const summary = await invoke<MeetingSummary>('end_meeting', {
  meetingId: "uuid-string"
});
```

**Response**:
```typescript
interface MeetingSummary {
  meeting_id: string;
  duration_seconds: number;
  transcript_path: string;           // Path to saved transcript
  requirements?: Requirements;        // Extracted requirements
  project_path?: string;             // Generated project path
  total_segments: number;
  summary_text: string;              // Executive summary
  next_steps: string[];              // Suggested action items
}
```

---

#### `pause_meeting`

Temporarily pauses transcription (stops audio capture).

**Request**:
```typescript
await invoke('pause_meeting', { meetingId: "uuid" });
```

**Response**: `void`

---

#### `resume_meeting`

Resumes a paused meeting.

**Request**:
```typescript
await invoke('resume_meeting', { meetingId: "uuid" });
```

**Response**: `void`

---

#### `get_meeting_status`

Gets current meeting status and metadata.

**Request**:
```typescript
const status = await invoke<MeetingStatus>('get_meeting_status', {
  meetingId: "uuid"
});
```

**Response**:
```typescript
interface MeetingStatus {
  id: string;
  state: "idle" | "recording" | "paused" | "generating" | "completed";
  start_time: string;        // ISO 8601
  elapsed_seconds: number;
  transcript_segments: number;
  last_activity: string;     // ISO 8601
}
```

---

### Transcription

#### `get_live_transcript`

Retrieves current transcript segments.

**Request**:
```typescript
const transcript = await invoke<TranscriptSegment[]>('get_live_transcript', {
  meetingId: "uuid",
  offset?: number,   // Optional: start from segment N
  limit?: number     // Optional: max segments to return
});
```

**Response**:
```typescript
interface TranscriptSegment {
  speaker: string;          // "Speaker 1", "Speaker 2", etc.
  start_time: number;       // Seconds from meeting start
  end_time: number;
  text: string;
  confidence: number;       // 0.0 - 1.0
  timestamp: string;        // ISO 8601
}
```

---

#### `update_speaker_labels`

Renames speaker labels (e.g., "Speaker 1" → "You").

**Request**:
```typescript
await invoke('update_speaker_labels', {
  meetingId: "uuid",
  mapping: {
    "Speaker 1": "Me",
    "Speaker 2": "Sarah"
  }
});
```

**Response**: `void`

---

### Requirements & Code Generation

#### `extract_requirements`

Manually trigger requirement extraction from transcript.

**Request**:
```typescript
const requirements = await invoke<Requirements>('extract_requirements', {
  meetingId: "uuid"
});
```

**Response**:
```typescript
interface Requirements {
  project_name: string;
  project_type: ProjectType;
  tech_stack: string[];
  features: Feature[];
  constraints: string[];
  questions: string[];
}

interface Feature {
  id: string;
  title: string;
  description: string;
  priority: "high" | "medium" | "low";
  mentioned_by: string;      // Speaker ID
  timestamp: number;         // Seconds from start
}

type ProjectType =
  | "react_web_app"
  | "node_api"
  | "python_cli"
  | "full_stack_app"
  | "static_website"
  | string;
```

---

#### `approve_requirements`

User approves/edits requirements before code generation.

**Request**:
```typescript
await invoke('approve_requirements', {
  meetingId: "uuid",
  requirements: {
    // Modified Requirements object
    project_name: "customer-feedback-app",
    features: [...]
  }
});
```

**Response**: `void`

**Side Effects**: Triggers code generation

---

#### `generate_code`

Manually trigger code generation (if not auto-triggered).

**Request**:
```typescript
const result = await invoke<CodeGenerationResult>('generate_code', {
  meetingId: "uuid",
  requirements?: Requirements  // Optional: use specific requirements
});
```

**Response**:
```typescript
interface CodeGenerationResult {
  project_path: string;
  files_generated: number;
  validation_passed: boolean;
  errors: string[];
  warnings: string[];
}
```

---

#### `get_generated_code`

Retrieves generated code files for preview.

**Request**:
```typescript
const project = await invoke<GeneratedProject>('get_generated_code', {
  projectId: "uuid"
});
```

**Response**:
```typescript
interface GeneratedProject {
  files: GeneratedFile[];
  metadata: ProjectMetadata;
}

interface GeneratedFile {
  path: string;              // Relative path
  content: string;           // File contents
  language: string;          // "typescript", "python", etc.
  validation_status: "valid" | "warning" | "error";
  validation_messages: string[];
}

interface ProjectMetadata {
  name: string;
  description: string;
  dependencies: Dependency[];
  setup_instructions: string;
}

interface Dependency {
  name: string;
  version: string;
  type: "runtime" | "dev";
}
```

---

#### `apply_code_updates`

Applies incremental updates to existing project.

**Request**:
```typescript
await invoke('apply_code_updates', {
  projectId: "uuid",
  updates: [
    {
      type: "create",
      path: "src/components/NewComponent.tsx",
      content: "..."
    },
    {
      type: "update",
      path: "src/App.tsx",
      content: "..."
    },
    {
      type: "delete",
      path: "src/OldComponent.tsx"
    }
  ]
});
```

**Response**: `void`

---

### Preview Server

#### `start_preview`

Starts development server for generated project.

**Request**:
```typescript
const previewUrl = await invoke<string>('start_preview', {
  projectPath: "/path/to/project"
});
```

**Response**:
```typescript
type Response = string; // "http://localhost:5173"
```

**Errors**:
- `ProjectNotFound`: Path doesn't exist
- `PreviewNotSupported`: Project type can't be previewed
- `PortInUse`: All ports occupied
- `DependenciesNotInstalled`: Need to run npm install

---

#### `stop_preview`

Stops running preview server.

**Request**:
```typescript
await invoke('stop_preview', {
  projectPath: "/path/to/project"
});
```

**Response**: `void`

---

### Settings & Configuration

#### `get_settings`

Retrieves current application settings.

**Request**:
```typescript
const settings = await invoke<AppSettings>('get_settings');
```

**Response**:
```typescript
interface AppSettings {
  // LLM Configuration
  llm_provider: "claude" | "openai" | "ollama";
  llm_model: string;                    // Provider-specific model name
  api_key_configured: boolean;          // Don't return actual key

  // Audio
  audio_device: string;
  always_on_microphone: boolean;

  // Transcription
  transcription_model: "whisper_small" | "whisper_medium" | "whisper_large" | "parakeet";
  enable_diarization: boolean;
  language: string;                     // ISO 639-1 code

  // Code Generation
  preferred_languages: string[];
  auto_generate: boolean;               // Auto-generate on sufficient context
  auto_apply_updates: boolean;          // Auto-apply incremental updates

  // Paths
  projects_directory: string;
  transcripts_directory: string;
}
```

---

#### `update_settings`

Updates application settings.

**Request**:
```typescript
await invoke('update_settings', {
  settings: {
    llm_provider: "claude",
    llm_model: "claude-3-5-sonnet-20250219",
    // ... partial settings update
  }
});
```

**Response**: `void`

---

#### `set_api_key`

Securely stores LLM API key.

**Request**:
```typescript
await invoke('set_api_key', {
  provider: "claude",
  apiKey: "sk-..."
});
```

**Response**: `void`

**Security**: Key stored in system keychain, never in plain text

---

#### `test_llm_connection`

Tests LLM provider connectivity.

**Request**:
```typescript
const result = await invoke<TestResult>('test_llm_connection', {
  provider: "claude"
});
```

**Response**:
```typescript
interface TestResult {
  success: boolean;
  error?: string;
  latency_ms?: number;
  model_available?: string;
}
```

---

## Tauri Events

Events are emitted by the backend to notify the frontend of state changes.

### `transcript-segment`

Emitted when a new transcript segment is available.

**Payload**:
```typescript
interface TranscriptSegmentEvent {
  meeting_id: string;
  segment: TranscriptSegment;
}
```

**Frontend Handler**:
```typescript
listen<TranscriptSegmentEvent>('transcript-segment', (event) => {
  console.log('New segment:', event.payload.segment.text);
});
```

---

### `import-progress`

Emitted during offline import to report progress stages.

Payload:
```typescript
type ImportStage =
  | 'starting'
  | 'downloading'        // YouTube only
  | 'loading-model'
  | 'decoding'
  | 'transcribing'
  | 'finalizing';

interface ImportProgressEvent {
  stage: ImportStage;
  percent?: number; // 0-100, provided for loading/transcribing where applicable
}
```

Frontend handler:
```typescript
listen<ImportProgressEvent>('import-progress', ({ payload }) => {
  console.log(payload.stage, payload.percent ?? 0);
});
```

---

### `requirements-extracted`

Emitted when requirements have been extracted and are ready for review.

**Payload**:
```typescript
interface RequirementsExtractedEvent {
  meeting_id: string;
  requirements: Requirements;
  auto_approved: boolean;  // If true, generation already started
}
```

---

### `generation-started`

Emitted when code generation begins.

**Payload**:
```typescript
interface GenerationStartedEvent {
  meeting_id: string;
  project_id: string;
  estimated_duration_seconds: number;
}
```

---

### `generation-progress`

Emitted periodically during code generation.

**Payload**:
```typescript
interface GenerationProgressEvent {
  project_id: string;
  progress: number;         // 0.0 - 1.0
  current_stage: string;    // "Generating components", "Writing files", etc.
  files_completed: number;
  files_total: number;
}
```

---

### `generation-completed`

Emitted when code generation finishes.

**Payload**:
```typescript
interface GenerationCompletedEvent {
  project_id: string;
  result: CodeGenerationResult;
}
```

---

### `code-updated`

Emitted when incremental updates are applied.

**Payload**:
```typescript
interface CodeUpdatedEvent {
  project_id: string;
  updates: FileUpdate[];
  changelog: string;  // Human-readable description
}

interface FileUpdate {
  path: string;
  change_type: "created" | "modified" | "deleted";
  reason: string;
}
```

---

### `meeting-insight`

Emitted when AI generates a suggestion or identifies an issue.

**Payload**:
```typescript
interface MeetingInsightEvent {
  meeting_id: string;
  insight: Insight;
}

interface Insight {
  type: "suggestion" | "warning" | "question" | "missing_info";
  title: string;
  description: string;
  timestamp: number;           // Seconds from meeting start
  actionable: boolean;
  suggested_action?: string;
}
```

---

### `validation-result`

Emitted after code validation completes.

**Payload**:
```typescript
interface ValidationResultEvent {
  project_id: string;
  file_path: string;
  status: "valid" | "warning" | "error";
  issues: ValidationIssue[];
}

interface ValidationIssue {
  severity: "error" | "warning" | "info";
  message: string;
  line?: number;
  column?: number;
  fix_suggestion?: string;
}
```

---

### `preview-server-ready`

Emitted when preview server has started and is accessible.

**Payload**:
```typescript
interface PreviewServerReadyEvent {
  project_id: string;
  url: string;              // "http://localhost:5173"
  server_type: string;      // "vite", "webpack", "node", etc.
}
```

---

## LLM Provider APIs

### Claude API (Anthropic)

**Endpoint**: `https://api.anthropic.com/v1/messages`

**Headers**:
```
x-api-key: <API_KEY>
anthropic-version: 2024-01-01
content-type: application/json
```

**Request Body**:
```json
{
  "model": "claude-3-5-sonnet-20250219",
  "max_tokens": 4096,
  "messages": [
    {
      "role": "user",
      "content": "<transcript>\n...\n</transcript>\n\nExtract requirements from this meeting transcript..."
    }
  ]
}
```

**Response**:
```json
{
  "id": "msg_...",
  "type": "message",
  "role": "assistant",
  "content": [
    {
      "type": "text",
      "text": "{\"project_name\": \"...\"}"
    }
  ]
}
```

**Rust Implementation**:
```rust
pub struct ClaudeProvider {
    api_key: String,
    client: reqwest::Client,
}

impl LLMProvider for ClaudeProvider {
    async fn generate(&self, request: GenerationRequest) -> Result<String> {
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2024-01-01")
            .json(&json!({
                "model": "claude-3-5-sonnet-20250219",
                "max_tokens": request.max_tokens,
                "messages": [{
                    "role": "user",
                    "content": request.prompt
                }]
            }))
            .send()
            .await?;

        let data: ClaudeResponse = response.json().await?;
        Ok(data.content[0].text.clone())
    }
}
```

---

### OpenAI API

**Endpoint**: `https://api.openai.com/v1/chat/completions`

**Headers**:
```
Authorization: Bearer <API_KEY>
Content-Type: application/json
```

**Request Body**:
```json
{
  "model": "gpt-4-turbo",
  "messages": [
    {
      "role": "system",
      "content": "You are an expert software requirements analyst."
    },
    {
      "role": "user",
      "content": "<transcript>...</transcript>\n\nExtract requirements..."
    }
  ],
  "max_tokens": 4096
}
```

**Response**:
```json
{
  "id": "chatcmpl-...",
  "choices": [
    {
      "message": {
        "role": "assistant",
        "content": "{\"project_name\": \"...\"}"
      },
      "finish_reason": "stop"
    }
  ]
}
```

**Rust Implementation**:
```rust
pub struct OpenAIProvider {
    api_key: String,
    client: reqwest::Client,
}

impl LLMProvider for OpenAIProvider {
    async fn generate(&self, request: GenerationRequest) -> Result<String> {
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&json!({
                "model": "gpt-4-turbo",
                "max_tokens": request.max_tokens,
                "messages": [
                    {
                        "role": "system",
                        "content": "You are an expert software engineer."
                    },
                    {
                        "role": "user",
                        "content": request.prompt
                    }
                ]
            }))
            .send()
            .await?;

        let data: OpenAIResponse = response.json().await?;
        Ok(data.choices[0].message.content.clone())
    }
}
```

---

### Ollama API (Local)

**Endpoint**: `http://localhost:11434/api/generate`

**Request Body**:
```json
{
  "model": "codellama",
  "prompt": "<transcript>...</transcript>\n\nExtract requirements...",
  "stream": false
}
```

**Response**:
```json
{
  "model": "codellama",
  "response": "{\"project_name\": \"...\"}",
  "done": true
}
```

**Rust Implementation**:
```rust
pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl LLMProvider for OllamaProvider {
    async fn generate(&self, request: GenerationRequest) -> Result<String> {
        let response = self.client
            .post(format!("{}/api/generate", self.base_url))
            .json(&json!({
                "model": request.model.unwrap_or("codellama".to_string()),
                "prompt": request.prompt,
                "stream": false
            }))
            .send()
            .await?;

        let data: OllamaResponse = response.json().await?;
        Ok(data.response)
    }
}
```

---

## Internal Rust APIs

### Manager Trait Pattern

All managers follow a consistent interface pattern:

```rust
pub trait Manager: Send + Sync {
    /// Initialize the manager with application handle
    fn new(app: &tauri::AppHandle) -> Result<Self> where Self: Sized;

    /// Cleanup resources on shutdown
    fn shutdown(&mut self) -> Result<()>;
}
```

---

### AudioRecordingManager API

```rust
pub trait AudioRecordingManager: Manager {
    /// Start capturing system audio
    fn start_system_capture(&mut self) -> Result<()>;

    /// Stop audio capture
    fn stop_capture(&mut self) -> Result<()>;

    /// Get next audio chunk (blocking)
    fn get_chunk(&self, duration: Duration) -> Result<Vec<f32>>;

    /// Get current audio level (0.0 - 1.0)
    fn get_audio_level(&self) -> f32;

    /// List available audio devices
    fn list_devices(&self) -> Result<Vec<AudioDevice>>;

    /// Switch to different audio device
    fn set_device(&mut self, device_id: &str) -> Result<()>;
}
```

---

### TranscriptionManager API

```rust
pub trait TranscriptionManager: Manager {
    /// Transcribe audio chunk
    async fn transcribe(
        &self,
        audio: Vec<f32>,
        sample_rate: usize,
    ) -> Result<TranscriptSegment>;

    /// Transcribe with speaker diarization
    async fn transcribe_with_speakers(
        &self,
        audio: Vec<f32>,
        sample_rate: usize,
    ) -> Result<Vec<SpeakerSegment>>;

    /// Load transcription model
    async fn load_model(&mut self, model: ModelType) -> Result<()>;

    /// Unload model from memory
    fn unload_model(&mut self) -> Result<()>;

    /// Check if model is loaded
    fn is_model_loaded(&self) -> bool;
}
```

---

### LLMManager API

```rust
pub trait LLMManager: Manager {
    /// Generate requirements from transcript
    async fn generate_requirements(
        &self,
        transcript: &str,
    ) -> Result<Requirements>;

    /// Generate code from requirements
    async fn generate_code(
        &self,
        requirements: &Requirements,
    ) -> Result<GeneratedProject>;

    /// Generate incremental update
    async fn generate_update(
        &self,
        context: &ProjectContext,
        new_transcript: &str,
    ) -> Result<Vec<FileUpdate>>;

    /// Generate meeting insight
    async fn generate_insight(
        &self,
        transcript: &str,
        context: &ProjectContext,
    ) -> Result<Insight>;

    /// Switch active provider
    fn set_provider(&mut self, provider: ProviderType) -> Result<()>;

    /// Test connection to provider
    async fn test_connection(&self) -> Result<TestResult>;
}
```

---

### ProjectManager API

```rust
pub trait ProjectManager: Manager {
    /// Create new project from generated files
    fn create_project(
        &mut self,
        name: &str,
        files: Vec<GeneratedFile>,
    ) -> Result<ProjectContext>;

    /// Update existing project
    fn update_project(
        &mut self,
        project_id: &str,
        updates: Vec<FileUpdate>,
    ) -> Result<()>;

    /// Get project context
    fn get_project(&self, project_id: &str) -> Option<&ProjectContext>;

    /// Delete project
    fn delete_project(&mut self, project_id: &str) -> Result<()>;

    /// List all projects
    fn list_projects(&self) -> Vec<ProjectSummary>;

    /// Install dependencies for project
    async fn install_dependencies(&self, project_id: &str) -> Result<()>;
}
```

---

## Error Types

All APIs use a consistent error handling pattern:

```rust
#[derive(Debug, thiserror::Error)]
pub enum MeetingCoderError {
    #[error("Audio device error: {0}")]
    AudioDevice(String),

    #[error("Transcription failed: {0}")]
    Transcription(String),

    #[error("LLM API error: {0}")]
    LLMApi(String),

    #[error("Code generation failed: {0}")]
    CodeGeneration(String),

    #[error("File I/O error: {0}")]
    FileIO(#[from] std::io::Error),

    #[error("Meeting not found: {0}")]
    MeetingNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, MeetingCoderError>;
```

---

## Versioning

All APIs follow semantic versioning. Breaking changes require major version bump.

**Current Version**: `v1.0.0`

**Compatibility Promise**:
- Tauri commands: Stable from v1.0.0
- Tauri events: Backward compatible (new fields added, old fields never removed)
- Internal Rust APIs: No stability guarantee (internal use only)
- LLM prompts: May change to improve quality (version field in response)

## Rate Limiting

### LLM APIs

Implement exponential backoff for rate limit errors:

```rust
async fn call_with_retry<F, T>(
    f: F,
    max_retries: u32,
) -> Result<T>
where
    F: Fn() -> Future<Output = Result<T>>,
{
    let mut delay = Duration::from_secs(1);

    for attempt in 0..max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_rate_limit() => {
                if attempt == max_retries - 1 {
                    return Err(e);
                }
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }

    unreachable!()
}
```

### Internal APIs

No rate limiting on internal APIs. Callers responsible for throttling.

## Testing Utilities

### Mock Providers

```rust
pub struct MockLLMProvider {
    responses: VecDeque<String>,
}

impl MockLLMProvider {
    pub fn with_responses(responses: Vec<String>) -> Self {
        Self {
            responses: responses.into(),
        }
    }
}

impl LLMProvider for MockLLMProvider {
    async fn generate(&self, _request: GenerationRequest) -> Result<String> {
        Ok(self.responses.pop_front().unwrap_or_default())
    }
}
```

### Test Fixtures

```rust
pub mod fixtures {
    pub fn sample_transcript() -> Vec<TranscriptSegment> {
        vec![
            TranscriptSegment {
                speaker: "Speaker 1".to_string(),
                start_time: 0.0,
                end_time: 3.5,
                text: "We need a login form with email and password".to_string(),
                confidence: 0.95,
                timestamp: SystemTime::now(),
            },
            // ...
        ]
    }

    pub fn sample_requirements() -> Requirements {
        // ...
    }
}
```
