use crate::managers::meeting::{MeetingManager, TranscriptSegment};
use crate::managers::transcription::TranscriptionManager;
use crate::queue::{Queue, QueueItem};
use anyhow::Result;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

fn load_wav_16k_mono_f32(path: &std::path::Path) -> Result<Vec<f32>> {
    let mut r = hound::WavReader::open(path)?;
    let spec = r.spec();
    if spec.channels != 1
        || spec.sample_rate != 16_000
        || spec.sample_format != hound::SampleFormat::Float
        || spec.bits_per_sample != 32
    {
        // Fallback: convert using audio loader pipeline if unexpected format
        return crate::audio_toolkit::audio::load_audio_file_to_mono_16k(path);
    }
    let mut out = Vec::with_capacity(r.len() as usize);
    for s in r.samples::<f32>() {
        out.push(s?);
    }
    Ok(out)
}

/// State for simple per-meeting speaker alternation in the worker
#[derive(Default, Clone, Copy)]
struct DiarState {
    /// Last assigned speaker index (1 or 2). 0 means uninitialized.
    last_speaker: u8,
}

fn next_speaker_label(state: &mut DiarState, turn_boundary: bool) -> String {
    // Start with Speaker 1 if uninitialized
    if state.last_speaker == 0 {
        state.last_speaker = 1;
        return "Speaker 1".to_string();
    }

    // Toggle only at a detected boundary; otherwise keep the same label
    if turn_boundary {
        state.last_speaker = if state.last_speaker == 1 { 2 } else { 1 };
    }
    format!("Speaker {}", state.last_speaker)
}

fn silence_fraction(samples: &[f32], thresh: f32) -> f32 {
    if samples.is_empty() {
        return 1.0;
    }
    let silent = samples.iter().filter(|&&s| s.abs() < thresh).count();
    silent as f32 / samples.len() as f32
}

fn process_item(
    item: QueueItem,
    meeting_manager: Arc<MeetingManager>,
    transcription_manager: Arc<TranscriptionManager>,
    app: &AppHandle,
    diar_map: &Arc<Mutex<HashMap<String, DiarState>>>,
) -> Result<()> {
    let p = std::path::Path::new(&item.file_path);
    let samples = load_wav_16k_mono_f32(p)?;
    if samples.is_empty() {
        return Err(anyhow::anyhow!("empty samples"));
    }

    let text = transcription_manager.transcribe(samples.clone())?;
    if text.trim().is_empty() {
        info!("ASR produced empty text for {:?}", p);
        return Ok(());
    }

    // Determine speaker label using a simple turn heuristic per meeting
    // Toggle speaker only if we observe notable silence inside the chunk
    let turn_boundary = silence_fraction(&samples, 1e-3) > 0.20; // coarse threshold
    let speaker_label = {
        let mut map = diar_map.lock().unwrap();
        let state = map.entry(item.meeting_id.clone()).or_default();
        next_speaker_label(state, turn_boundary)
    };

    let segment = TranscriptSegment {
        speaker: speaker_label,
        start_time: (item.start_ms as f64) / 1000.0,
        end_time: (item.end_ms as f64) / 1000.0,
        text: text.clone(),
        confidence: 0.95,
        timestamp: std::time::SystemTime::now(),
    };

    // Determine segment index as current length before appending
    let mut next_index: usize = 0;
    if let Ok(m) = tauri::async_runtime::block_on(meeting_manager.get_meeting(&item.meeting_id)) {
        next_index = m.transcript_segments.len();
    }

    // Try to append to active meeting
    if let Err(e) = tauri::async_runtime::block_on(
        meeting_manager.add_segment(&item.meeting_id, segment.clone()),
    ) {
        warn!("add_segment failed (meeting may have ended): {}", e);
    }

    // Emit UI event
    #[derive(Clone, serde::Serialize)]
    struct SegmentAddedPayload {
        meeting_id: String,
        segment: TranscriptSegment,
    }
    let _ = app.emit(
        "transcript-segment-added",
        SegmentAddedPayload {
            meeting_id: item.meeting_id.clone(),
            segment: segment.clone(),
        },
    );

    // Append rolling transcript on disk if we can get project path
    if let Ok(m) = tauri::async_runtime::block_on(meeting_manager.get_meeting(&item.meeting_id)) {
        if let Some(pp) = m.project_path.clone() {
            if let Err(e) = crate::meeting::transcript_writer::append_segment(
                &pp,
                &item.meeting_id,
                next_index,
                &segment,
            ) {
                warn!("append_segment failed: {}", e);
            }
        }
    }

    Ok(())
}

pub fn spawn(
    queue: Arc<Queue>,
    meeting_manager: Arc<MeetingManager>,
    transcription_manager: Arc<TranscriptionManager>,
    app: AppHandle,
) {
    // Map to track simple per-meeting diarization state for alternation
    let diar_map: Arc<Mutex<HashMap<String, DiarState>>> = Arc::new(Mutex::new(HashMap::new()));
    thread::spawn(move || loop {
        match queue.fetch_next() {
            Ok(Some(item)) => {
                info!("ASR worker picked item {} {:?}", item.id, item.file_path);
                let res = process_item(
                    item.clone(),
                    meeting_manager.clone(),
                    transcription_manager.clone(),
                    &app,
                    &diar_map,
                );
                match res {
                    Ok(()) => {
                        let _ = queue.mark_done(item.id);
                    }
                    Err(e) => {
                        error!("ASR worker failed: {}", e);
                        let _ = queue.mark_failed(item.id, &format!("{}", e));
                    }
                }
            }
            Ok(None) => {
                thread::sleep(Duration::from_millis(250));
            }
            Err(e) => {
                error!("Queue fetch error: {}", e);
                thread::sleep(Duration::from_millis(500));
            }
        }
    });
}
