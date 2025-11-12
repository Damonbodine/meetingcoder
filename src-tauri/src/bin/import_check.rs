use meetingcoder_app_lib::audio_toolkit::audio::load_audio_file_to_mono_16k;
use meetingcoder_app_lib::audio_toolkit::vad::{SileroVad, SmoothedVad, VoiceActivityDetector, VadFrame};
use std::env;
use std::time::Instant;

fn build_vad_segments(samples: &[f32], min_segment_seconds: u32) -> Result<Vec<(usize, usize)>, String> {
    // 16kHz, 30ms frames
    let mut inner = SileroVad::new("./resources/models/silero_vad_v4.onnx", 0.4).map_err(|e| e.to_string())?;
    // Smooth parameters: prefill=10 (~300ms), hangover=12 (~360ms), onset=3 (~90ms)
    let mut vad = SmoothedVad::new(Box::new(inner), 10, 12, 3);

    let frame_len = 480usize; // 30 ms @ 16k
    let mut segments: Vec<(usize, usize)> = Vec::new();
    let mut in_speech = false;
    let mut seg_start: usize = 0;
    let mut last_speech_idx = 0usize;
    let mut idx = 0usize;
    let max_seg_samples = 16_000usize * 60; // cap ~60s

    while idx + frame_len <= samples.len() {
        let frame = &samples[idx..idx + frame_len];
        let out = vad.push_frame(frame).map_err(|e| e.to_string())?;
        match out {
            VadFrame::Speech(_) => {
                last_speech_idx = idx + frame_len;
                if !in_speech {
                    seg_start = idx.saturating_sub(frame_len * 10);
                    in_speech = true;
                } else if last_speech_idx - seg_start > max_seg_samples {
                    segments.push((seg_start, last_speech_idx));
                    seg_start = last_speech_idx.saturating_sub(1600);
                }
            }
            VadFrame::Noise => {
                if in_speech {
                    segments.push((seg_start, last_speech_idx));
                    in_speech = false;
                }
            }
        }
        idx += frame_len;
    }
    if in_speech { segments.push((seg_start, samples.len())); }

    // Merge tiny gaps and clamp
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (s, e) in segments.into_iter() {
        if s >= e { continue; }
        if let Some((_, pe)) = merged.last_mut() {
            if s.saturating_sub(*pe) < 1600 { // <0.1s
                *pe = (*pe).max(e);
                continue;
            }
        }
        merged.push((s, e.min(samples.len())));
    }

    // Compact very short segments by merging forward until a minimum duration is reached
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

    Ok(compact)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: import_check <audio_path>");
        std::process::exit(2);
    }
    let path = &args[1];
    println!("Loading: {}", path);

    let t0 = Instant::now();
    let samples = load_audio_file_to_mono_16k(path)?;
    let t_decode = t0.elapsed();
    if samples.len() < 16_000 { eprintln!("Decoded audio too short: {} samples", samples.len()); std::process::exit(3); }
    let minutes = samples.len() as f64 / 16_000f64 / 60.0;
    println!(
        "Import decode: mono_samples={}, approx_minutes={:.2}, wall_time={:.2}s",
        samples.len(), minutes, t_decode.as_secs_f64()
    );

    let t1 = Instant::now();
    let segs = build_vad_segments(&samples, 10)?;
    let t_vad = t1.elapsed();
    let total_secs = samples.len() as f64 / 16_000f64;
    let mut last_end = 0usize;
    let mut lens: Vec<f64> = Vec::new();
    for (s, e) in &segs { last_end = last_end.max(*e); lens.push((*e as f64 - *s as f64) / 16_000f64); }
    lens.sort_by(|a,b| a.partial_cmp(b).unwrap());
    let mean = if !lens.is_empty() { lens.iter().sum::<f64>() / lens.len() as f64 } else { 0.0 };
    let median = if !lens.is_empty() { let mid = lens.len()/2; if lens.len()%2==0 {(lens[mid-1]+lens[mid])/2.0} else { lens[mid] } } else { 0.0 };
    println!(
        "VAD segments: count={}, mean_len={:.2}s, median_len={:.2}s, last_end={:.2}s, total={:.2}s, vad_time={:.2}s",
        segs.len(), mean, median, last_end as f64 / 16_000f64, total_secs, t_vad.as_secs_f64()
    );

    Ok(())
}
