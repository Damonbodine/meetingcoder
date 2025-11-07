import { z } from "zod";

export const ShortcutBindingSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string(),
  default_binding: z.string(),
  current_binding: z.string(),
});

export const ShortcutBindingsMapSchema = z.record(
  z.string(),
  ShortcutBindingSchema,
);

export const AudioDeviceSchema = z.object({
  index: z.string(),
  name: z.string(),
  is_default: z.boolean(),
});

export const OverlayPositionSchema = z.enum(["none", "top", "bottom"]);
export type OverlayPosition = z.infer<typeof OverlayPositionSchema>;

export const ModelUnloadTimeoutSchema = z.enum([
  "never",
  "immediately",
  "min2",
  "min5",
  "min10",
  "min15",
  "hour1",
  "sec5",
]);
export type ModelUnloadTimeout = z.infer<typeof ModelUnloadTimeoutSchema>;

export const PasteMethodSchema = z.enum(["ctrl_v", "direct"]);
export type PasteMethod = z.infer<typeof PasteMethodSchema>;

export const ClipboardHandlingSchema = z.enum(["dont_modify", "copy_to_clipboard"]);
export type ClipboardHandling = z.infer<typeof ClipboardHandlingSchema>;

export const SettingsSchema = z.object({
  bindings: ShortcutBindingsMapSchema,
  push_to_talk: z.boolean(),
  audio_feedback: z.boolean(),
  audio_feedback_volume: z.number().optional().default(1.0),
  sound_theme: z
    .enum(["marimba", "pop", "custom"])
    .optional()
    .default("marimba"),
  start_hidden: z.boolean().optional().default(false),
  autostart_enabled: z.boolean().optional().default(false),
  selected_model: z.string(),
  always_on_microphone: z.boolean(),
  selected_microphone: z.string().nullable().optional(),
  selected_output_device: z.string().nullable().optional(),
  translate_to_english: z.boolean(),
  selected_language: z.string(),
  overlay_position: OverlayPositionSchema,
  debug_mode: z.boolean(),
  custom_words: z.array(z.string()).optional().default([]),
  model_unload_timeout: ModelUnloadTimeoutSchema.optional().default("never"),
  word_correction_threshold: z.number().optional().default(0.18),
  history_limit: z.number().optional().default(5),
  paste_method: PasteMethodSchema.optional().default("ctrl_v"),
  clipboard_handling: ClipboardHandlingSchema.optional().default("dont_modify"),
  mute_while_recording: z.boolean().optional().default(false),
  transcription_chunk_seconds: z.number().optional().default(10),
  meeting_update_interval_seconds: z.number().optional().default(20),
  auto_trigger_meeting_command: z.boolean().optional().default(false),
  auto_accept_changes: z.boolean().optional().default(false),
  auto_trigger_min_interval_seconds: z.number().optional().default(75),
  github_repo_owner: z.string().nullable().optional(),
  github_repo_name: z.string().nullable().optional(),
  github_default_branch: z.string().optional().default("main"),
  github_branch_pattern: z.string().optional().default("meeting/{meeting_id}"),
  github_enabled: z.boolean().optional().default(false),
  prefer_whisper_for_imports: z.boolean().optional().default(false),
  fast_import_mode_for_imports: z.boolean().optional().default(true),
  use_fixed_windows_for_imports: z.boolean().optional().default(false),
  min_segment_duration_for_imports: z.number().optional().default(10),
  ffmpeg_fallback_for_imports: z.boolean().optional().default(true),
});

export const BindingResponseSchema = z.object({
  success: z.boolean(),
  binding: ShortcutBindingSchema.nullable(),
  error: z.string().nullable(),
});

export type AudioDevice = z.infer<typeof AudioDeviceSchema>;
export type BindingResponse = z.infer<typeof BindingResponseSchema>;
export type ShortcutBinding = z.infer<typeof ShortcutBindingSchema>;
export type ShortcutBindingsMap = z.infer<typeof ShortcutBindingsMapSchema>;
export type Settings = z.infer<typeof SettingsSchema>;

export const ModelInfoSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string(),
  filename: z.string(),
  url: z.string().optional(),
  size_mb: z.number(),
  is_downloaded: z.boolean(),
  is_downloading: z.boolean(),
  partial_size: z.number(),
  is_directory: z.boolean(),
  accuracy_score: z.number(),
  speed_score: z.number(),
});

export type ModelInfo = z.infer<typeof ModelInfoSchema>;

// Meeting types
export const MeetingStatusSchema = z.enum(["recording", "paused", "completed"]);
export type MeetingStatus = z.infer<typeof MeetingStatusSchema>;

export const TranscriptSegmentSchema = z.object({
  speaker: z.string(),
  start_time: z.number(),
  end_time: z.number(),
  text: z.string(),
  confidence: z.number(),
  timestamp: z.number(), // Unix timestamp in milliseconds
});

export type TranscriptSegment = z.infer<typeof TranscriptSegmentSchema>;

export const MeetingSessionSchema = z.object({
  id: z.string(),
  name: z.string(),
  start_time: z.number(), // Unix timestamp
  end_time: z.number().optional().nullable(),
  transcript_segments: z.array(TranscriptSegmentSchema),
  status: MeetingStatusSchema,
  participants: z.array(z.string()),
});

export type MeetingSession = z.infer<typeof MeetingSessionSchema>;

export const MeetingSummarySchema = z.object({
  meeting_id: z.string(),
  name: z.string(),
  duration_seconds: z.number(),
  total_segments: z.number(),
  participants: z.array(z.string()),
  start_time: z.number(),
  end_time: z.number(),
});

export type MeetingSummary = z.infer<typeof MeetingSummarySchema>;

// Meeting history types
export const TranscriptMetadataSchema = z.object({
  meeting_id: z.string(),
  name: z.string(),
  start_time: z.string(), // ISO 8601 format
  end_time: z.string(),   // ISO 8601 format
  duration_seconds: z.number(),
  participants: z.array(z.string()),
});

export type TranscriptMetadata = z.infer<typeof TranscriptMetadataSchema>;

export const MeetingHistoryEntrySchema = z.object({
  dir_name: z.string(),
  dir_path: z.string(),
  metadata: TranscriptMetadataSchema,
});

export type MeetingHistoryEntry = z.infer<typeof MeetingHistoryEntrySchema>;

// GitHub types
export const GitHubRepoStatusSchema = z.object({
  repo_owner: z.string().nullable().optional(),
  repo_name: z.string().nullable().optional(),
  default_branch: z.string(),
  branch_pattern: z.string(),
  has_token: z.boolean(),
  current_branch: z.string().nullable().optional(),
  last_pr_url: z.string().nullable().optional(),
  last_pr_number: z.number().nullable().optional(),
  last_push_time: z.string().nullable().optional(),
});

export type GitHubRepoStatus = z.infer<typeof GitHubRepoStatusSchema>;

export const GitHubConnectionTestSchema = z.object({
  success: z.boolean(),
  username: z.string().nullable().optional(),
  error: z.string().nullable().optional(),
});

export type GitHubConnectionTest = z.infer<typeof GitHubConnectionTestSchema>;

export const PushResultSchema = z.object({
  success: z.boolean(),
  branch: z.string(),
  commit_message: z.string(),
  error: z.string().nullable().optional(),
});

export type PushResult = z.infer<typeof PushResultSchema>;

export const PRResultSchema = z.object({
  success: z.boolean(),
  pr_number: z.number().nullable().optional(),
  pr_url: z.string().nullable().optional(),
  error: z.string().nullable().optional(),
});

export type PRResult = z.infer<typeof PRResultSchema>;

export const RepoInfoSchema = z.object({
  id: z.number(),
  name: z.string(),
  full_name: z.string(),
  owner: z.object({
    login: z.string(),
  }),
  private: z.boolean(),
  description: z.string().nullable().optional(),
  html_url: z.string(),
  default_branch: z.string(),
});

export type RepoInfo = z.infer<typeof RepoInfoSchema>;
