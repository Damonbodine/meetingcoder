use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::{AppHandle, Manager, State, Emitter};

use crate::audio_toolkit::audio::load_audio_file_to_mono_16k;
use crate::automation::claude_trigger::trigger_meeting_update;
use crate::managers::meeting::{MeetingManager, TranscriptSegment};
use crate::managers::transcription::TranscriptionManager;
use crate::meeting::context_writer::append_update;
use crate::settings;
use crate::summarization::agent::summarize_segments_with_context;
use crate::audio_toolkit::vad::{SileroVad, SmoothedVad, VoiceActivityDetector, VadFrame};
use crate::managers::model::{ModelManager, EngineType};
use tauri::path::BaseDirectory;

#[derive(serde::Serialize, Clone)]
struct ImportProgress<'a> {
    stage: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    percent: Option<u8>,
}

fn emit_progress(app: &AppHandle, stage: &str, percent: Option<u8>) {
    let _ = app.emit(
        "import-progress",
        ImportProgress { stage, percent },
    );
}

/// Native file picker for audio files via Rust dialog plugin.
/// Returns an optional absolute path as String.
#[tauri::command]
pub async fn pick_audio_file(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .add_filter(
            "Audio",
            &[
                "wav", "mp3", "m4a", "ogg", "flac", "aac", "mp4", "m4b", "webm", "opus", "oga", "weba", "aiff", "aif", "caf"
            ],
        )
        .blocking_pick_file();
    Ok(picked.map(|p| p.to_string()))
}

fn is_supported_audio_extension(path: &PathBuf) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext = ext.to_ascii_lowercase();
        matches!(ext.as_str(),
            "wav" | "mp3" | "m4a" | "ogg" | "flac" | "aac" | "mp4" | "m4b" | "webm" | "opus" | "oga" | "weba" | "aiff" | "aif" | "caf"
        )
    } else {
        false
    }
}

const MAX_IMPORT_FILE_BYTES: u64 = 1_500_000_000; // ~1.5GB safety cap

async fn import_audio_from_path_as_meeting(
    app: AppHandle,
    meeting_name: String,
    file_path: String,
    source_label: &str,
    meeting_manager: State<'_, Arc<MeetingManager>>,
    transcription_manager: State<'_, Arc<TranscriptionManager>>,
    model_manager: State<'_, Arc<ModelManager>>,
) -> Result<crate::managers::meeting::MeetingSummary, String> {
    emit_progress(&app, "starting", Some(0));

    // Validate and normalize path
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    if !is_supported_audio_extension(&path) {
        return Err(format!(
            "Unsupported file type: {} (supported: wav, mp3, m4a, ogg, flac)",
            path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("<none>")
        ));
    }
    if let Ok(meta) = fs::metadata(&path) {
        if meta.len() > MAX_IMPORT_FILE_BYTES {
            return Err(format!(
                "Audio file is too large (>{} MB). Please trim or convert.",
                MAX_IMPORT_FILE_BYTES / 1_000_000
            ));
        }
    }

    // Start an offline meeting
    let meeting_id = meeting_manager
        .start_offline_meeting(meeting_name.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Load model for import; prefer Whisper if enabled and available
    emit_progress(&app, "loading-model", Some(0));
    let settings_now = settings::get_settings(&app);
    let mut loaded_any = false;
    if settings_now.prefer_whisper_for_imports {
        let mut whisper_models: Vec<_> = model_manager
            .get_available_models()
            .into_iter()
            .filter(|m| m.engine_type == EngineType::Whisper && m.is_downloaded)
            .collect();
        whisper_models.sort_by(|a, b| b.accuracy_score.total_cmp(&a.accuracy_score));
        if let Some(best) = whisper_models.first() {
            match transcription_manager.load_model(&best.id) {
                Ok(_) => {
                    log::info!("Import: using Whisper model '{}' due to preference.", best.id);
                    loaded_any = true;
                }
                Err(e) => log::warn!("Failed to load preferred Whisper model {}: {}. Falling back to selected model.", best.id, e),
            }
        } else {
            log::info!("Import: Whisper preference enabled but no downloaded Whisper models; using selected model.");
        }
    }
    if !loaded_any {
        // Fall back: wait briefly for selected model to load (up to 30s)
        transcription_manager.initiate_model_load();
        let mut waited = 0u32;
        while !transcription_manager.is_model_loaded() && waited < 30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            waited += 1;
            emit_progress(&app, "loading-model", Some(((waited * 100) / 30) as u8));
        }
        if !transcription_manager.is_model_loaded() {
            return Err("Transcription model not loaded. Open Model Selector and download/select a model.".to_string());
        }
    }

    // Decode and convert audio
    emit_progress(&app, "decoding", Some(0));
    let decode_wall = std::time::Instant::now();
    let samples = load_audio_file_to_mono_16k(path).map_err(|e| e.to_string())?;
    if samples.is_empty() {
        return Err("Audio decode produced zero samples. The file may be corrupt or unsupported.".to_string());
    }
    // Opportunistic ffmpeg fallback: if container is MP4/M4A/AAC/WEBM and length < 2 minutes, and setting enabled
    let mut samples = samples; // make mutable
    if settings_now.ffmpeg_fallback_for_imports {
        if let Some(ext) = Path::new(&file_path).extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()) {
            let is_problematic_container = matches!(ext.as_str(), "mp4" | "m4a" | "aac" | "webm");
            let short_threshold = 16_000usize * 60 * 2; // 2 minutes
            if is_problematic_container && samples.len() < short_threshold {
                log::warn!(
                    "Import decode seems short for container '{}': {} samples (~{:.2} sec). Attempting ffmpeg fallback...",
                    ext,
                    samples.len(),
                    samples.len() as f64 / 16_000f64
                );
                if let Ok(v) = ffmpeg_decode_to_mono_16k_pcm(&PathBuf::from(&file_path)) {
                    if v.len() > samples.len() {
                        log::info!(
                            "FFmpeg fallback succeeded: mono_samples={}, approx_minutes={:.2}",
                            v.len(),
                            v.len() as f64 / 16_000f64 / 60.0
                        );
                        samples = v;
                    } else {
                        log::warn!("FFmpeg fallback produced no improvement ({} <= {}). Keeping original decode.", v.len(), samples.len());
                    }
                } else {
                    log::warn!("FFmpeg fallback failed or ffmpeg not found. Proceeding with original decode.");
                }
            }
        }
    }
    // Instrumentation: mono sample count and approx minutes
    let minutes = (samples.len() as f64) / 16_000f64 / 60.0;
    let decode_secs = decode_wall.elapsed().as_secs_f64();
    log::info!(
        "Import decode: mono_samples={}, approx_minutes={:.2}, wall_time={:.2}s",
        samples.len(),
        minutes,
        decode_secs
    );

    // Segment and transcribe
    let settings = settings::get_settings(&app);
    let total = samples.len();
    // Select segmentation strategy
    let segments_to_process = if settings.use_fixed_windows_for_imports {
        log::info!("Segmentation: using fixed 45s windows (imports override)");
        build_fixed_segments_with_overlap(total, 45, 0.9)
    } else {
        // Try VAD segmentation for more natural boundaries; fallback to fixed windows
        match build_vad_segments(&app, &samples, settings.min_segment_duration_for_imports) {
            Ok(v) if !v.is_empty() => v,
            _ => build_fixed_segments_with_overlap(total, 45, 0.9),
        }
    };
    // Instrumentation: segment coverage + stats
    {
        let total_secs = (total as f64) / 16_000f64;
        let mut n = 0usize;
        let mut last_end = 0usize;
        let mut lens: Vec<f64> = Vec::new();
        for (s, e) in &segments_to_process {
            n += 1;
            last_end = last_end.max(*e);
            let dur = (*e as f64 - *s as f64) / 16_000f64;
            lens.push(dur);
            log::debug!("segment {}: start={:.2}s end={:.2}s dur={:.2}s", n, (*s as f64)/16_000f64, (*e as f64)/16_000f64, dur);
        }
        lens.sort_by(|a,b| a.partial_cmp(b).unwrap());
        let mean = if !lens.is_empty() { lens.iter().sum::<f64>() / lens.len() as f64 } else { 0.0 };
        let median = if !lens.is_empty() { let mid = lens.len()/2; if lens.len()%2==0 {(lens[mid-1]+lens[mid])/2.0} else { lens[mid] } } else { 0.0 };
        log::info!("Import segments: count={}, mean_len={:.2}s, median_len={:.2}s, last_end={:.2}s, total={:.2}s",
            n, mean, median, (last_end as f64)/16_000f64, total_secs);
    }
    let mut processed_until = 0usize;
    let mut segments_accum: Vec<TranscriptSegment> = Vec::new();
    let import_start = std::time::Instant::now();
    let mut total_audio_sec_processed = 0.0f64;
    let mut total_wall_sec_transcribing = 0.0f64;

    // Fetch project_path for updates
    let project_path = meeting_manager
        .get_meeting(&meeting_id)
        .await
        .ok()
        .and_then(|m| m.project_path);

    let mut sent_last_update_idx: usize = 0;

    // Helper to trim overlapping text at segment joins (UTF-8 safe)
    fn trim_overlap(prev: &str, cur: &str) -> String {
        if prev.is_empty() || cur.is_empty() { return cur.to_string(); }
        let prev_chars: Vec<char> = prev.chars().collect();
        let tail_start = prev_chars.len().saturating_sub(200);
        let prev_tail: String = prev_chars[tail_start..].iter().collect();

        let cur_chars: Vec<char> = cur.chars().collect();
        let max_check = cur_chars.len().min(120);
        let mut best = 0usize;
        for k in (10..=max_check).rev() {
            let prefix: String = cur_chars[..k].iter().collect();
            if prev_tail.ends_with(&prefix) { best = k; break; }
        }
        cur_chars[best..].iter().collect()
    }

    for (start_idx_global, end_idx) in segments_to_process.into_iter() {
        let chunk = samples[start_idx_global..end_idx].to_vec();

        let start_time = (start_idx_global as f64) / 16_000f64;
        let end_time = (end_idx as f64) / 16_000f64;

        // Progress based on furthest processed point
        processed_until = processed_until.max(end_idx);
        let pct = ((processed_until as f64 / total as f64) * 100.0).round() as u8;
        emit_progress(&app, "transcribing", Some(pct.min(100)));

        // Transcribe chunk (blocking)
        let chunk_audio_sec = (end_idx - start_idx_global) as f64 / 16_000f64;
        let chunk_wall_start = std::time::Instant::now();
        let text = {
            let tm = transcription_manager.inner().clone();
            tauri::async_runtime::spawn_blocking(move || tm.transcribe(chunk))
                .await
                .map_err(|e| e.to_string())
                .and_then(|r| r.map_err(|e| e.to_string()))?
        };
        let wall = chunk_wall_start.elapsed().as_secs_f64();
        total_audio_sec_processed += chunk_audio_sec;
        total_wall_sec_transcribing += wall;
        if wall > 0.0 {
            log::info!(
                "Transcribe chunk: dur={:.2}s, wall={:.2}s, throughput={:.2}x",
                chunk_audio_sec,
                wall,
                chunk_audio_sec / wall
            );
        }

        if !text.trim().is_empty() {
            let final_text = if let Some(prev) = segments_accum.last() {
                let trimmed = trim_overlap(&prev.text, &text);
                if trimmed.len() < text.len() {
                    log::debug!("Trimmed {} overlapping chars at segment join", text.len() - trimmed.len());
                }
                trimmed
            } else { text.clone() };
            let seg = TranscriptSegment {
                speaker: "Speaker 1".to_string(),
                start_time,
                end_time,
                text: final_text.clone(),
                confidence: 0.95,
                timestamp: std::time::SystemTime::now(),
            };
            meeting_manager
                .add_segment(&meeting_id, seg.clone())
                .await
                .map_err(|e| e.to_string())?;
            segments_accum.push(seg);

            // Append a summary update periodically or after each chunk
            if let Some(pp) = &project_path {
                let full_transcript = meeting_manager
                    .get_live_transcript(&meeting_id)
                    .await
                    .map_err(|e| e.to_string())?;
                let start = sent_last_update_idx;
                let end = full_transcript.len().saturating_sub(1);
                if end >= start {
                    let summary = summarize_segments_with_context(
                        Some(pp),
                        &full_transcript,
                        start,
                        end,
                    );
                    if let Ok(update_id) = append_update(
                        pp,
                        &meeting_id,
                        &meeting_name,
                        &settings.selected_model,
                        source_label,
                        &summary,
                    ) {
                        // Notify frontend and attempt automation
                        let _ = app.emit(
                            "meeting-update-appended",
                            serde_json::json!({"update_id": update_id, "meeting_id": meeting_id }),
                        );
                        let _ = trigger_meeting_update(&app, pp, &meeting_id, update_id);
                        sent_last_update_idx = end + 1;
                    }
                }
            }
        }

    }

    emit_progress(&app, "finalizing", Some(100));
    let total_wall = import_start.elapsed().as_secs_f64();
    let total_minutes = (total as f64) / 16_000f64 / 60.0;
    if total_wall > 0.0 {
        log::info!(
            "Import performance: audio_total={:.2} min, wall_total={:.2} min, RTF={:.2}x (audio_sec/wall_sec)",
            total_minutes,
            total_wall / 60.0,
            (total as f64 / 16_000f64) / total_wall
        );
    }

    // End meeting and persist transcript
    meeting_manager
        .end_meeting(&meeting_id)
        .await
        .map_err(|e| e.to_string())
}

fn build_fixed_segments_with_overlap(total: usize, chunk_seconds: u32, overlap_seconds: f64) -> Vec<(usize, usize)> {
    // Favor longer chunks for better context.
    let chunk_seconds = (chunk_seconds.max(20).min(60)) as usize;
    let chunk_len = 16_000usize * chunk_seconds;
    // Add overlap to avoid cutting words at boundaries
    let overlap_samples: usize = (16_000f64 * overlap_seconds).round() as usize;

    let mut v = Vec::new();
    let mut start_idx_global = 0usize;
    while start_idx_global < total {
        let end_idx = (start_idx_global + chunk_len).min(total);
        v.push((start_idx_global, end_idx));
        // If we've reached the end, stop. Avoid creating a trailing
        // tiny window consisting only of overlap.
        if end_idx == total {
            break;
        }
        if end_idx >= overlap_samples {
            let next = end_idx - overlap_samples;
            start_idx_global = if next > start_idx_global { next } else { end_idx };
        } else {
            start_idx_global = end_idx;
        }
    }
    v
}

fn build_vad_segments(app: &AppHandle, samples: &[f32], min_segment_seconds: u32) -> Result<Vec<(usize, usize)>, String> {
    let vad_path = app
        .path()
        .resolve("resources/models/silero_vad_v4.onnx", BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve VAD path: {}", e))?;
    let inner = SileroVad::new(&vad_path, 0.4).map_err(|e| e.to_string())?; // slightly more permissive
    // Smooth parameters: prefill=10 (~300ms), hangover=12 (~360ms), onset=3 (~90ms)
    let mut vad = SmoothedVad::new(Box::new(inner), 10, 12, 3);

    let frame_len = 480usize; // 30 ms @ 16k
    let mut segments: Vec<(usize, usize)> = Vec::new();
    let mut in_speech = false;
    let mut seg_start: usize = 0;
    let mut last_speech_idx = 0usize;
    let mut idx = 0usize;
    let max_seg_samples = 16_000usize * 60; // cap ~60s per segment to improve context

    while idx + frame_len <= samples.len() {
        let frame = &samples[idx..idx + frame_len];
        let out = vad.push_frame(frame).map_err(|e| e.to_string())?;
        match out {
            VadFrame::Speech(_) => {
                // update last speech
                last_speech_idx = idx + frame_len;
                if !in_speech {
                    // New speech block. Backoff a bit to include pre-roll
                    seg_start = idx.saturating_sub(frame_len * 10);
                    in_speech = true;
                } else if last_speech_idx - seg_start > max_seg_samples {
                    // Split long segments
                    segments.push((seg_start, last_speech_idx));
                    seg_start = last_speech_idx.saturating_sub(1600); // 0.1s overlap
                }
            }
            VadFrame::Noise => {
                if in_speech {
                    // End of speech block
                    segments.push((seg_start, last_speech_idx));
                    in_speech = false;
                }
            }
        }
        idx += frame_len;
    }
    if in_speech {
        segments.push((seg_start, samples.len()));
    }

    // Merge tiny gaps and clamp
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (s, e) in segments.into_iter() {
        if s >= e { continue; }
        if let Some((ps, pe)) = merged.last_mut() {
            if s.saturating_sub(*pe) < 1600 { // merge small gaps (<0.1s)
                *pe = (*pe).max(e);
                continue;
            }
        }
        merged.push((s, e.min(samples.len())));
    }
    // Compact very short segments by merging forward until a minimum duration is reached (configurable, default 8-10s)
    let min_seg_samples = 16_000usize * (min_segment_seconds.max(5).min(15) as usize);
    let mut compact: Vec<(usize, usize)> = Vec::new();
    let mut acc_start: Option<usize> = None;
    let mut acc_end: usize = 0;
    for (s, e) in merged.into_iter() {
        if acc_start.is_none() {
            acc_start = Some(s);
            acc_end = e;
            continue;
        }
        let cur_len = acc_end.saturating_sub(acc_start.unwrap());
        if cur_len < min_seg_samples {
            acc_end = e.min(acc_start.unwrap() + max_seg_samples);
        } else {
            compact.push((acc_start.unwrap(), acc_end));
            acc_start = Some(s);
            acc_end = e;
        }
    }
    if let Some(st) = acc_start { compact.push((st, acc_end)); }

    // Summary log for VAD segmentation (post-compaction)
    let mut lens: Vec<f64> = compact.iter().map(|(s,e)| ((*e as f64 - *s as f64) / 16_000f64)).collect();
    lens.sort_by(|a,b| a.partial_cmp(b).unwrap());
    let mean = if !lens.is_empty() { lens.iter().sum::<f64>() / lens.len() as f64 } else { 0.0 };
    let median = if !lens.is_empty() { let mid = lens.len()/2; if lens.len()%2==0 {(lens[mid-1]+lens[mid])/2.0} else { lens[mid] } } else { 0.0 };
    log::info!("VAD segments: count={}, mean_len={:.2}s, median_len={:.2}s", compact.len(), mean, median);
    Ok(compact)
}

fn ffmpeg_decode_to_mono_16k_pcm(src: &Path) -> Result<Vec<f32>, String> {
    // Create temp WAV path
    let mut tmp = std::env::temp_dir();
    let fname = format!(
        "import_tmp_{}.wav",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    tmp.push(fname);

    // ffmpeg -v error -y -i <in> -f wav -ac 1 -ar 16000 -acodec pcm_s16le <tmp.wav>
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-v",
            "error",
            "-y",
            "-i",
            src.to_string_lossy().as_ref(),
            "-f",
            "wav",
            "-ac",
            "1",
            "-ar",
            "16000",
            "-acodec",
            "pcm_s16le",
            tmp.to_string_lossy().as_ref(),
        ])
        .output();

    let mut out_samples: Vec<f32> = Vec::new();
    match status {
        Ok(output) => {
            if !output.status.success() {
                return Err(format!("ffmpeg returned error code: {:?}", output.status.code()));
            }
            // Read WAV via hound
            match hound::WavReader::open(&tmp) {
                Ok(mut reader) => {
                    let spec = reader.spec();
                    if spec.sample_rate != 16_000 {
                        log::warn!("ffmpeg wav sample_rate={} (expected 16000)", spec.sample_rate);
                    }
                    // Support only PCM 16 here (we forced pcm_s16le above)
                    for s in reader.samples::<i16>() {
                        let v = s.map_err(|e| e.to_string())? as f32 / i16::MAX as f32;
                        out_samples.push(v);
                    }
                }
                Err(e) => return Err(format!("Failed to open ffmpeg wav: {}", e)),
            }
        }
        Err(e) => return Err(format!("Failed to execute ffmpeg: {}", e)),
    }

    // Cleanup temp file
    let _ = std::fs::remove_file(&tmp);
    Ok(out_samples)
}

/// Import a local audio file as a new offline meeting.
#[tauri::command]
pub async fn import_audio_as_meeting(
    app: AppHandle,
    meeting_name: String,
    file_path: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
    transcription_manager: State<'_, Arc<TranscriptionManager>>,
    model_manager: State<'_, Arc<ModelManager>>,
) -> Result<crate::managers::meeting::MeetingSummary, String> {
    import_audio_from_path_as_meeting(
        app,
        meeting_name,
        file_path,
        "import:file",
        meeting_manager,
        transcription_manager,
        model_manager,
    )
    .await
}

/// Import a YouTube URL as a new offline meeting. Requires `yt-dlp` in PATH.
#[tauri::command]
pub async fn import_youtube_as_meeting(
    app: AppHandle,
    meeting_name: String,
    url: String,
    meeting_manager: State<'_, Arc<MeetingManager>>,
    transcription_manager: State<'_, Arc<TranscriptionManager>>,
    model_manager: State<'_, Arc<ModelManager>>,
) -> Result<crate::managers::meeting::MeetingSummary, String> {
    // Indicate start of YouTube flow
    emit_progress(&app, "downloading", Some(0));

    // Check yt-dlp availability explicitly to return a clean error if missing
    match std::process::Command::new("yt-dlp").arg("--version").output() {
        Ok(out) => {
            if !out.status.success() {
                return Err("yt-dlp not found. Please install yt-dlp and ensure it is on your PATH.".to_string());
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err("yt-dlp not found. Please install yt-dlp and ensure it is on your PATH.".to_string());
            }
            return Err(format!("Failed to check yt-dlp availability: {}", e));
        }
    }

    // Download best audio to a temp file using yt-dlp
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let target = app_dir.join("downloads");
    std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;

    let output = target.join("yt_audio.%(ext)s");
    let output_str = output.to_string_lossy().to_string();

    let output_exec = std::process::Command::new("yt-dlp")
        .arg("-f")
        .arg("bestaudio/best")
        .arg("-o")
        .arg(&output_str)
        .arg(&url)
        .output()
        .map_err(|e| format!("Failed to spawn yt-dlp: {}", e))?;
    if !output_exec.status.success() {
        let stderr = String::from_utf8_lossy(&output_exec.stderr);
        let msg = if stderr.to_lowercase().contains("network")
            || stderr.to_lowercase().contains("unable to download data")
        {
            "yt-dlp failed to download audio due to network issues. Check your connection and try again.".to_string()
        } else {
            format!(
                "yt-dlp failed to download audio. Details: {}",
                stderr.trim()
            )
        };
        return Err(msg);
    }

    // Find the downloaded file (first matching yt_audio.*)
    let mut found: Option<PathBuf> = None;
    if let Ok(entries) = std::fs::read_dir(&target) {
        for e in entries.flatten() {
            let p = e.path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("yt_audio.") {
                    found = Some(p);
                    break;
                }
            }
        }
    }
    let audio_file = found.ok_or_else(|| "Downloaded audio file not found".to_string())?;

    // Delegate to file importer with source label
    import_audio_from_path_as_meeting(
        app,
        meeting_name,
        audio_file.to_string_lossy().to_string(),
        "import:youtube",
        meeting_manager,
        transcription_manager,
        model_manager,
    )
    .await
}
