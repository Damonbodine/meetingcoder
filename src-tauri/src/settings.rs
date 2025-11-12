use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShortcutBinding {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_binding: String,
    pub current_binding: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OverlayPosition {
    None,
    Top,
    Bottom,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelUnloadTimeout {
    Never,
    Immediately,
    Min2,
    Min5,
    Min10,
    Min15,
    Hour1,
    Sec5, // Debug mode only
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PasteMethod {
    CtrlV,
    Direct,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardHandling {
    DontModify,
    CopyToClipboard,
}

impl Default for ModelUnloadTimeout {
    fn default() -> Self {
        ModelUnloadTimeout::Never
    }
}

impl Default for PasteMethod {
    fn default() -> Self {
        // Default to CtrlV for macOS and Windows, Direct for Linux
        #[cfg(target_os = "linux")]
        return PasteMethod::Direct;
        #[cfg(not(target_os = "linux"))]
        return PasteMethod::CtrlV;
    }
}

impl Default for ClipboardHandling {
    fn default() -> Self {
        ClipboardHandling::DontModify
    }
}

impl ModelUnloadTimeout {
    pub fn to_minutes(self) -> Option<u64> {
        match self {
            ModelUnloadTimeout::Never => None,
            ModelUnloadTimeout::Immediately => Some(0), // Special case for immediate unloading
            ModelUnloadTimeout::Min2 => Some(2),
            ModelUnloadTimeout::Min5 => Some(5),
            ModelUnloadTimeout::Min10 => Some(10),
            ModelUnloadTimeout::Min15 => Some(15),
            ModelUnloadTimeout::Hour1 => Some(60),
            ModelUnloadTimeout::Sec5 => Some(0), // Special case for debug - handled separately
        }
    }

    pub fn to_seconds(self) -> Option<u64> {
        match self {
            ModelUnloadTimeout::Never => None,
            ModelUnloadTimeout::Immediately => Some(0), // Special case for immediate unloading
            ModelUnloadTimeout::Sec5 => Some(5),
            _ => self.to_minutes().map(|m| m * 60),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SoundTheme {
    Marimba,
    Pop,
    Custom,
}

impl SoundTheme {
    fn as_str(&self) -> &'static str {
        match self {
            SoundTheme::Marimba => "marimba",
            SoundTheme::Pop => "pop",
            SoundTheme::Custom => "custom",
        }
    }

    pub fn to_start_path(&self) -> String {
        format!("resources/{}_start.wav", self.as_str())
    }

    pub fn to_stop_path(&self) -> String {
        format!("resources/{}_stop.wav", self.as_str())
    }
}

/* still handy for composing the initial JSON in the store ------------- */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSettings {
    pub bindings: HashMap<String, ShortcutBinding>,
    pub push_to_talk: bool,
    pub audio_feedback: bool,
    #[serde(default = "default_audio_feedback_volume")]
    pub audio_feedback_volume: f32,
    #[serde(default = "default_sound_theme")]
    pub sound_theme: SoundTheme,
    #[serde(default = "default_start_hidden")]
    pub start_hidden: bool,
    #[serde(default = "default_autostart_enabled")]
    pub autostart_enabled: bool,
    #[serde(default = "default_model")]
    pub selected_model: String,
    #[serde(default = "default_always_on_microphone")]
    pub always_on_microphone: bool,
    #[serde(default)]
    pub selected_microphone: Option<String>,
    #[serde(default)]
    pub selected_output_device: Option<String>,
    #[serde(default = "default_translate_to_english")]
    pub translate_to_english: bool,
    #[serde(default = "default_selected_language")]
    pub selected_language: String,
    #[serde(default = "default_overlay_position")]
    pub overlay_position: OverlayPosition,
    #[serde(default = "default_debug_mode")]
    pub debug_mode: bool,
    #[serde(default)]
    pub custom_words: Vec<String>,
    #[serde(default)]
    pub model_unload_timeout: ModelUnloadTimeout,
    #[serde(default = "default_word_correction_threshold")]
    pub word_correction_threshold: f64,
    #[serde(default = "default_history_limit")]
    pub history_limit: usize,
    #[serde(default)]
    pub paste_method: PasteMethod,
    #[serde(default)]
    pub clipboard_handling: ClipboardHandling,
    #[serde(default)]
    pub mute_while_recording: bool,
    #[serde(default = "default_transcription_chunk_seconds")]
    pub transcription_chunk_seconds: u32,
    #[serde(default = "default_system_audio_buffer_seconds")]
    pub system_audio_buffer_seconds: u32,
    #[serde(default = "default_system_audio_silence_threshold")]
    pub system_audio_silence_threshold: f32, // dBFS
    #[serde(default = "default_meeting_update_interval_seconds")]
    pub meeting_update_interval_seconds: u32,
    #[serde(default = "default_auto_trigger_meeting_command")]
    pub auto_trigger_meeting_command: bool,
    #[serde(default = "default_auto_accept_changes")]
    pub auto_accept_changes: bool,
    #[serde(default = "default_auto_trigger_min_interval_seconds")]
    pub auto_trigger_min_interval_seconds: u32,
    #[serde(default)]
    pub github_repo_owner: Option<String>,
    #[serde(default)]
    pub github_repo_name: Option<String>,
    #[serde(default = "default_github_default_branch")]
    pub github_default_branch: String,
    #[serde(default = "default_github_branch_pattern")]
    pub github_branch_pattern: String,
    #[serde(default = "default_github_enabled")]
    pub github_enabled: bool,
    #[serde(default = "default_github_auto_commit_push")]
    pub github_auto_commit_push: bool,
    #[serde(default = "default_github_auto_create_pr")]
    pub github_auto_create_pr: bool,
    #[serde(default = "default_github_auto_update_pr")]
    pub github_auto_update_pr: bool,
    #[serde(default = "default_prefer_whisper_for_imports")]
    pub prefer_whisper_for_imports: bool,
    #[serde(default = "default_fast_import_mode_for_imports")]
    pub fast_import_mode_for_imports: bool,
    #[serde(default = "default_use_fixed_windows_for_imports")]
    pub use_fixed_windows_for_imports: bool,
    #[serde(default = "default_min_segment_duration_for_imports")]
    pub min_segment_duration_for_imports: u32,
    #[serde(default = "default_ffmpeg_fallback_for_imports")]
    pub ffmpeg_fallback_for_imports: bool,
    #[serde(default = "default_use_llm_summarization")]
    pub use_llm_summarization: bool,
    #[serde(default = "default_llm_model")]
    pub llm_model: String,
    #[serde(default = "default_use_queue_transcription")]
    pub use_queue_transcription: bool,
    #[serde(default = "default_queue_worker_count")]
    pub queue_worker_count: u32,
    // PRD generation settings
    #[serde(default = "default_enable_prd_generation")]
    pub enable_prd_generation: bool,
    #[serde(default = "default_prd_initial_min_segments")]
    pub prd_initial_min_segments: usize,
    #[serde(default = "default_prd_update_interval_minutes")]
    pub prd_update_interval_minutes: u64,
}

fn default_model() -> String {
    "".to_string()
}

fn default_always_on_microphone() -> bool {
    false
}

fn default_translate_to_english() -> bool {
    false
}

fn default_start_hidden() -> bool {
    false
}

fn default_autostart_enabled() -> bool {
    false
}

fn default_selected_language() -> String {
    "auto".to_string()
}

fn default_overlay_position() -> OverlayPosition {
    #[cfg(target_os = "linux")]
    return OverlayPosition::None;
    #[cfg(not(target_os = "linux"))]
    return OverlayPosition::Bottom;
}

fn default_debug_mode() -> bool {
    false
}

fn default_word_correction_threshold() -> f64 {
    0.18
}

fn default_history_limit() -> usize {
    5
}

fn default_audio_feedback_volume() -> f32 {
    1.0
}

fn default_sound_theme() -> SoundTheme {
    SoundTheme::Marimba
}

fn default_transcription_chunk_seconds() -> u32 {
    10
}

fn default_system_audio_silence_threshold() -> f32 { -50.0 }

// Lower default buffer size to reduce RAM footprint and backlog risk.
// 90s @ 16kHz mono float32 ≈ 5.8 MB
fn default_system_audio_buffer_seconds() -> u32 { 90 }

fn default_meeting_update_interval_seconds() -> u32 {
    20
}

fn default_auto_trigger_meeting_command() -> bool { false }

fn default_auto_accept_changes() -> bool { false }

fn default_auto_trigger_min_interval_seconds() -> u32 { 75 }

fn default_github_default_branch() -> String { "main".to_string() }

fn default_github_branch_pattern() -> String { "meeting/{meeting_id}".to_string() }

fn default_github_enabled() -> bool { false }
fn default_github_auto_commit_push() -> bool { true }
fn default_github_auto_create_pr() -> bool { true }
fn default_github_auto_update_pr() -> bool { true }
fn default_prefer_whisper_for_imports() -> bool { false }
fn default_fast_import_mode_for_imports() -> bool { true }
fn default_use_fixed_windows_for_imports() -> bool { false }
fn default_min_segment_duration_for_imports() -> u32 { 10 }
fn default_ffmpeg_fallback_for_imports() -> bool { true }
fn default_use_llm_summarization() -> bool { false }
fn default_llm_model() -> String { "claude-sonnet-4-5-20250929".to_string() }
fn default_use_queue_transcription() -> bool { true }
fn default_queue_worker_count() -> u32 { 2 }
fn default_enable_prd_generation() -> bool { true }
fn default_prd_initial_min_segments() -> usize { 15 }
fn default_prd_update_interval_minutes() -> u64 { 15 }

pub const SETTINGS_STORE_PATH: &str = "settings_store.json";

pub fn get_default_settings() -> AppSettings {
    #[cfg(target_os = "windows")]
    let default_shortcut = "ctrl+space";
    #[cfg(target_os = "macos")]
    let default_shortcut = "option+space";
    #[cfg(target_os = "linux")]
    let default_shortcut = "ctrl+space";
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let default_shortcut = "alt+space";

    let mut bindings = HashMap::new();
    bindings.insert(
        "transcribe".to_string(),
        ShortcutBinding {
            id: "transcribe".to_string(),
            name: "Transcribe".to_string(),
            description: "Converts your speech into text.".to_string(),
            default_binding: default_shortcut.to_string(),
            current_binding: default_shortcut.to_string(),
        },
    );

    AppSettings {
        bindings,
        push_to_talk: true,
        audio_feedback: false,
        audio_feedback_volume: default_audio_feedback_volume(),
        sound_theme: default_sound_theme(),
        start_hidden: default_start_hidden(),
        autostart_enabled: default_autostart_enabled(),
        selected_model: "".to_string(),
        always_on_microphone: false,
        selected_microphone: None,
        selected_output_device: None,
        translate_to_english: false,
        selected_language: "auto".to_string(),
        overlay_position: OverlayPosition::Bottom,
        debug_mode: false,
        custom_words: Vec::new(),
        model_unload_timeout: ModelUnloadTimeout::Never,
        word_correction_threshold: default_word_correction_threshold(),
        history_limit: default_history_limit(),
        paste_method: PasteMethod::default(),
        clipboard_handling: ClipboardHandling::default(),
        mute_while_recording: false,
        transcription_chunk_seconds: default_transcription_chunk_seconds(),
        system_audio_buffer_seconds: default_system_audio_buffer_seconds(),
        system_audio_silence_threshold: default_system_audio_silence_threshold(),
        meeting_update_interval_seconds: default_meeting_update_interval_seconds(),
        auto_trigger_meeting_command: default_auto_trigger_meeting_command(),
        auto_accept_changes: default_auto_accept_changes(),
        auto_trigger_min_interval_seconds: default_auto_trigger_min_interval_seconds(),
        github_repo_owner: None,
        github_repo_name: None,
        github_default_branch: default_github_default_branch(),
        github_branch_pattern: default_github_branch_pattern(),
        github_enabled: default_github_enabled(),
        github_auto_commit_push: default_github_auto_commit_push(),
        github_auto_create_pr: default_github_auto_create_pr(),
        github_auto_update_pr: default_github_auto_update_pr(),
        prefer_whisper_for_imports: default_prefer_whisper_for_imports(),
        fast_import_mode_for_imports: default_fast_import_mode_for_imports(),
        use_fixed_windows_for_imports: default_use_fixed_windows_for_imports(),
        min_segment_duration_for_imports: default_min_segment_duration_for_imports(),
        ffmpeg_fallback_for_imports: default_ffmpeg_fallback_for_imports(),
        use_llm_summarization: default_use_llm_summarization(),
        llm_model: default_llm_model(),
        use_queue_transcription: default_use_queue_transcription(),
        queue_worker_count: default_queue_worker_count(),
        enable_prd_generation: default_enable_prd_generation(),
        prd_initial_min_segments: default_prd_initial_min_segments(),
        prd_update_interval_minutes: default_prd_update_interval_minutes(),
    }
}

pub fn load_or_create_app_settings(app: &AppHandle) -> AppSettings {
    // Initialize store
    let store = app
        .store(SETTINGS_STORE_PATH)
        .expect("Failed to initialize store");

    let settings = if let Some(settings_value) = store.get("settings") {
        // Parse the entire settings object
        match serde_json::from_value::<AppSettings>(settings_value) {
            Ok(settings) => {
                println!("Found existing settings: {:?}", settings);

                settings
            }
            Err(e) => {
                println!("Failed to parse settings: {}", e);
                // Fall back to default settings if parsing fails
                let default_settings = get_default_settings();

                // Store the default settings
                store.set("settings", serde_json::to_value(&default_settings).unwrap());

                default_settings
            }
        }
    } else {
        // Create default settings
        let default_settings = get_default_settings();

        // Store the settings
        store.set("settings", serde_json::to_value(&default_settings).unwrap());

        default_settings
    };

    settings
}

pub fn get_settings(app: &AppHandle) -> AppSettings {
    let store = app
        .store(SETTINGS_STORE_PATH)
        .expect("Failed to initialize store");

    if let Some(settings_value) = store.get("settings") {
        serde_json::from_value::<AppSettings>(settings_value)
            .unwrap_or_else(|_| get_default_settings())
    } else {
        get_default_settings()
    }
}

pub fn write_settings(app: &AppHandle, settings: AppSettings) {
    let store = app
        .store(SETTINGS_STORE_PATH)
        .expect("Failed to initialize store");

    store.set("settings", serde_json::to_value(&settings).unwrap());
}

pub fn get_bindings(app: &AppHandle) -> HashMap<String, ShortcutBinding> {
    let settings = get_settings(app);

    settings.bindings
}

pub fn get_stored_binding(app: &AppHandle, id: &str) -> ShortcutBinding {
    let bindings = get_bindings(app);

    let binding = bindings.get(id).unwrap().clone();

    binding
}

pub fn get_history_limit(app: &AppHandle) -> usize {
    let settings = get_settings(app);
    settings.history_limit
}
