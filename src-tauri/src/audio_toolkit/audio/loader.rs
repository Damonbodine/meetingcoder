use anyhow::{anyhow, Result};
use rodio::Source;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::super::audio::resampler::FrameResampler;

/// Load an audio file of common formats and convert to mono 16kHz f32 samples.
/// Strategy:
/// - Prefer Symphonia for MP4/M4A/AAC where rodio may truncate
/// - If the first decoder yields < 1 second of audio (<16000 samples), try the alternate decoder
/// - If both are short, return a clear error instead of proceeding
pub fn load_audio_file_to_mono_16k<P: AsRef<Path>>(path: P) -> Result<Vec<f32>> {
    let path_ref = path.as_ref();
    let ext = path_ref
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let prefer_symphonia = matches!(ext.as_str(), "m4a" | "mp4" | "aac");

    let (primary, secondary): (fn(&Path) -> Result<Vec<f32>>, fn(&Path) -> Result<Vec<f32>>) =
        if prefer_symphonia {
            (decode_with_symphonia, decode_with_rodio)
        } else {
            (decode_with_rodio, decode_with_symphonia)
        };

    // Try primary
    let mut primary_out = match primary(path_ref) {
        Ok(v) => v,
        Err(e) => {
            log::warn!(
                "Primary decoder failed for {}: {} — trying alternate",
                path_ref.display(),
                e
            );
            Vec::new()
        }
    };

    // If clearly too short or failed, try alternate
    if primary_out.len() < 16_000 {
        let alt_out = match secondary(path_ref) {
            Ok(v) => v,
            Err(e) => {
                if primary_out.is_empty() {
                    // Both failed hard
                    return Err(anyhow!(
                        "Audio decode failed with both decoders for {}: {}",
                        path_ref.display(),
                        e
                    ));
                }
                log::warn!("Alternate decoder also errored for {}: {}", path_ref.display(), e);
                Vec::new()
            }
        };

        if alt_out.len() > primary_out.len() {
            log::info!(
                "Dual-decode heuristic: chose alternate decoder output ({} > {} samples)",
                alt_out.len(),
                primary_out.len()
            );
            primary_out = alt_out;
        }
    }

    if primary_out.len() < 16_000 {
        return Err(anyhow!(
            "Decoded audio too short (<1s). Got {} samples for {}",
            primary_out.len(),
            path_ref.display()
        ));
    }

    Ok(primary_out)
}

fn decode_with_rodio(path_ref: &Path) -> Result<Vec<f32>> {
    let file = File::open(path_ref).map_err(|e| anyhow!("Failed to open file {}: {}", path_ref.display(), e))?;
    let reader = BufReader::new(file);
    let decoder = rodio::Decoder::new(reader)
        .map_err(|e| anyhow!("Failed to decode audio {}: {}", path_ref.display(), e))?;
    let in_rate = decoder.sample_rate() as usize;
    let channels = decoder.channels() as usize;
    if channels == 0 { return Err(anyhow!("Audio file has 0 channels: {}", path_ref.display())); }
    let mut interleaved: Vec<f32> = Vec::new();
    if let Some(dur) = decoder.total_duration() {
        let est_samples = (dur.as_secs_f64() * in_rate as f64 * channels as f64) as usize;
        interleaved.reserve(est_samples.min(100_000_000));
    }
    for s in decoder { interleaved.push((s as f32) / (i16::MAX as f32)); if interleaved.len() > 300_000_000 { break; } }
    if interleaved.is_empty() { return Err(anyhow!("Decoded zero samples from {}", path_ref.display())); }
    log::info!(
        "Decoded with rodio: path={}, channels={}, in_rate={} Hz, interleaved_samples={}",
        path_ref.display(),
        channels,
        in_rate,
        interleaved.len()
    );
    let frames = interleaved.len() / channels;
    let mut mono: Vec<f32> = Vec::with_capacity(frames);
    for i in 0..frames {
        let mut acc = 0.0f32; for c in 0..channels { acc += interleaved[i * channels + c]; }
        mono.push(acc / channels as f32);
    }
    let out = resample_to_16k(in_rate, &mono)?;
    log::info!(
        "Rodio resample: in_rate={} -> out_rate=16000 Hz, mono_in_samples={}, mono_out_samples={}",
        in_rate,
        mono.len(),
        out.len()
    );
    Ok(out)
}

fn resample_to_16k(in_rate: usize, mono: &[f32]) -> Result<Vec<f32>> {
    if in_rate == 16_000 { return Ok(mono.to_vec()); }
    let mut resampler = FrameResampler::new(in_rate, 16_000, std::time::Duration::from_millis(30));
    let mut out: Vec<f32> = Vec::with_capacity((mono.len() as f64 * 16_000f64 / in_rate as f64).ceil() as usize + 1024);
    resampler.push(mono, |frame| out.extend_from_slice(frame));
    resampler.finish(|frame| out.extend_from_slice(frame));
    Ok(out)
}

fn decode_with_symphonia(path_ref: &Path) -> Result<Vec<f32>> {
    use symphonia::core::audio::{AudioBufferRef, Signal};
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::errors::Error as SymphoniaError;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = File::open(path_ref)
        .map_err(|e| anyhow!("Failed to open file {}: {}", path_ref.display(), e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path_ref.extension().and_then(|e| e.to_str()) { hint.with_extension(ext); }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| anyhow!("Symphonia probe failed: {}", e))?;
    let mut format = probed.format;

    // Select the first audio track.
    let track = format
        .default_track()
        .ok_or_else(|| anyhow!("No supported audio tracks"))?;

    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| anyhow!("Unknown sample rate"))? as usize;
    // Some Zoom M4A files report unknown channel layout. Default to 2 channels instead of failing.
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count())
        .unwrap_or(2);

    log::info!(
        "Symphonia probe: path={}, codec={:?}, channels={}, in_rate={} Hz",
        path_ref.display(),
        track.codec_params.codec,
        channels,
        sample_rate
    );

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| anyhow!("Symphonia decoder init failed: {}", e))?;

    let mut mono: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::ResetRequired) => { decoder.reset(); continue; }
            Err(SymphoniaError::IoError(err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(SymphoniaError::IoError(err)) if err.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(anyhow!("Symphonia read error: {}", e)),
        };

        match decoder.decode(&packet) {
            Ok(AudioBufferRef::F32(buf)) => {
                // Downmix to mono by averaging channels
                let ch = buf.spec().channels.count();
                if ch == 1 {
                    mono.extend_from_slice(buf.chan(0));
                } else {
                    for i in 0..buf.frames() {
                        let mut acc = 0.0f32;
                        for c in 0..ch { acc += buf.chan(c)[i]; }
                        mono.push(acc / ch as f32);
                    }
                }
            }
            Ok(AudioBufferRef::S16(buf)) => {
                let ch = buf.spec().channels.count();
                if ch == 1 {
                    mono.extend(buf.chan(0).iter().map(|&s| (s as f32) / (i16::MAX as f32)));
                } else {
                    for i in 0..buf.frames() {
                        let mut acc = 0.0f32;
                        for c in 0..ch { acc += (buf.chan(c)[i] as f32) / (i16::MAX as f32); }
                        mono.push(acc / ch as f32);
                    }
                }
            }
            Ok(AudioBufferRef::U8(buf)) => {
                let ch = buf.spec().channels.count();
                let to_f32 = |x: u8| ((x as f32) - 128.0) / 128.0;
                if ch == 1 {
                    mono.extend(buf.chan(0).iter().map(|&s| to_f32(s)));
                } else {
                    for i in 0..buf.frames() {
                        let mut acc = 0.0f32;
                        for c in 0..ch { acc += to_f32(buf.chan(c)[i]); }
                        mono.push(acc / ch as f32);
                    }
                }
            }
            Ok(other) => {
                // Convert other sample formats to f32 into a new buffer
                let spec = *other.spec();
                let frames = other.frames() as u64;
                let mut out_buf = symphonia::core::audio::AudioBuffer::<f32>::new(frames, spec);
                other.convert(&mut out_buf);
                let ch = out_buf.spec().channels.count();
                if ch == 1 {
                    mono.extend_from_slice(out_buf.chan(0));
                } else {
                    for i in 0..out_buf.frames() {
                        let mut acc = 0.0f32;
                        for c in 0..ch { acc += out_buf.chan(c)[i]; }
                        mono.push(acc / ch as f32);
                    }
                }
            }
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(anyhow!("Symphonia decode error: {}", e)),
        }

        if mono.len() > 100_000_000 { break; }
    }

    if mono.is_empty() { return Err(anyhow!("Decoded zero samples from {}", path_ref.display())); }
    let out = resample_to_16k(sample_rate, &mono)?;
    log::info!(
        "Symphonia resample: in_rate={} -> out_rate=16000 Hz, mono_in_samples={}, mono_out_samples={}",
        sample_rate,
        mono.len(),
        out.len()
    );
    Ok(out)
}
