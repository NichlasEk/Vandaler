use euther_oxide::audio::Audio;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_INSTRUMENT_BANK: &str =
    "audio/instruments/vand_furnace_core/bank.vand-instruments.json";
const DEFAULT_BASS_INSTRUMENT: &str = "growl_bass_wobbly";
const DEFAULT_LEAD_INSTRUMENT: &str = "fm_grinder";
const DEFAULT_NOISE_INSTRUMENT: &str = "psg_echo_warble";

#[derive(Clone, Deserialize, Serialize)]
struct AudioLabSettings {
    #[serde(default)]
    source: String,
    #[serde(default = "default_output_dir")]
    output_dir: String,
    #[serde(default)]
    instrument_bank: String,
    #[serde(default)]
    presets: HashMap<String, String>,
}

impl Default for AudioLabSettings {
    fn default() -> Self {
        Self {
            source: String::new(),
            output_dir: default_output_dir(),
            instrument_bank: String::new(),
            presets: HashMap::new(),
        }
    }
}

fn default_output_dir() -> String {
    "audio/converted".to_string()
}

#[derive(Serialize)]
struct AnalysisSummary {
    frames: u32,
    active_frames: u32,
    runtime_events: u32,
    dac_chunks: u32,
    arrangement: String,
    note_arrangement: String,
    preview_report: String,
    key: String,
    bpm: f32,
    bass_notes: u32,
    lead_notes: u32,
    drum_events: u32,
    bundle_dir: String,
    import_metadata: String,
    dac_preview: String,
}

#[derive(Default, Deserialize)]
struct PreviewReport {
    #[serde(default)]
    key: String,
    #[serde(default)]
    bpm: f32,
    #[serde(default)]
    bass_notes: u32,
    #[serde(default)]
    lead_notes: u32,
    #[serde(default)]
    drum_events: u32,
}

#[derive(Serialize)]
struct AnalysisResult {
    summary: AnalysisSummary,
    log: String,
}

#[derive(Deserialize, Serialize)]
struct InstrumentBankManifest {
    format: String,
    #[serde(default)]
    source_dir: String,
    #[serde(default)]
    source: String,
    license_review: String,
    instrument_count: u32,
    instruments: Vec<InstrumentBankEntry>,
}

#[derive(Deserialize, Serialize)]
struct InstrumentBankEntry {
    id: String,
    name: String,
    chip: String,
    category: String,
    source: String,
    file: String,
}

#[derive(Serialize)]
struct InstrumentBankResult {
    manifest: InstrumentBankManifest,
    manifest_path: String,
    log: String,
}

#[derive(Clone, Deserialize)]
struct VandInstrument {
    name: String,
    chip: String,
    category: String,
    fm: Option<VandFmPatch>,
}

#[derive(Clone, Deserialize)]
struct VandFmPatch {
    algorithm: u8,
    feedback: u8,
    operators: Vec<VandFmOperator>,
}

#[derive(Clone, Deserialize)]
struct VandFmOperator {
    mult: u8,
    tl: u8,
    ar: u8,
    dr: u8,
    sl: u8,
    rr: u8,
    dt: u8,
}

#[derive(Deserialize)]
struct VandArrangement {
    #[serde(default = "default_sample_rate")]
    sample_rate: u32,
    #[serde(default = "default_arrangement_hop")]
    hop: usize,
    #[serde(default)]
    instrument_bank: String,
    events: Vec<VandArrangementFrame>,
}

#[derive(Deserialize)]
struct VandArrangementFrame {
    #[serde(default, rename = "class")]
    class_name: String,
    #[serde(default)]
    bass_hz: f32,
    #[serde(default)]
    bass_amp: f32,
    #[serde(default)]
    bass_instrument_id: String,
    #[serde(default)]
    lead_hz: f32,
    #[serde(default)]
    lead_amp: f32,
    #[serde(default)]
    lead_instrument_id: String,
    #[serde(default)]
    noise_amp: f32,
    #[serde(default)]
    transient: f32,
    #[serde(default)]
    noise_instrument_id: String,
}

#[derive(Deserialize)]
struct VandNoteArrangement {
    #[serde(default = "default_sample_rate")]
    sample_rate: u32,
    #[serde(default = "default_arrangement_hop")]
    hop: usize,
    #[serde(default)]
    total_frames: usize,
    #[serde(default)]
    instrument_bank: String,
    notes: Vec<VandNoteEvent>,
    #[serde(default)]
    drums: Vec<VandDrumEvent>,
}

#[derive(Clone, Deserialize)]
struct VandNoteEvent {
    track: String,
    start_frame: usize,
    frames: usize,
    hz: f32,
    velocity: f32,
    instrument_id: String,
}

#[derive(Clone, Deserialize)]
struct VandDrumEvent {
    start_frame: usize,
    velocity: f32,
    noise_amp: f32,
    instrument_id: String,
    #[serde(default)]
    dac_chunk: Option<usize>,
    #[serde(default)]
    dac_path: String,
}

fn default_sample_rate() -> u32 {
    44_100
}

fn default_arrangement_hop() -> usize {
    512
}

fn repo_root() -> Result<PathBuf, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(4)
        .map(Path::to_path_buf)
        .ok_or_else(|| "could not locate repository root".to_string())
}

fn settings_path() -> Result<PathBuf, String> {
    Ok(repo_root()?.join("tools/audio/vand_ai_lism/settings.toml"))
}

fn bundle_path(input: &Path, output_dir: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("audio");
    output_dir
        .join(format!("{stem}.vand-audio"))
        .join("arrangement.vand-audio.json")
}

fn resolve_output_dir(repo: &Path, output_dir: &Path) -> PathBuf {
    if output_dir.is_absolute() {
        output_dir.to_path_buf()
    } else {
        repo.join(output_dir)
    }
}

fn mime_for_path(path: &Path) -> Result<&'static str, String> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("mp3") => Ok("audio/mpeg"),
        Some("ogg") => Ok("audio/ogg"),
        Some("wav") => Ok("audio/wav"),
        Some("flac") => Ok("audio/flac"),
        Some("aif") | Some("aiff") => Ok("audio/aiff"),
        _ => Err(format!("unsupported audio file: {}", path.display())),
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut index = 0;

    while index < bytes.len() {
        let b0 = bytes[index];
        let b1 = bytes.get(index + 1).copied().unwrap_or(0);
        let b2 = bytes.get(index + 2).copied().unwrap_or(0);
        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | b2 as u32;

        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if index + 1 < bytes.len() {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if index + 2 < bytes.len() {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }

        index += 3;
    }

    out
}

fn write_preview_wav(path: &Path, sample_rate: u32, samples: &[i16]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let mut file = fs::File::create(path)
        .map_err(|err| format!("failed to create {}: {err}", path.display()))?;
    let data_len = (samples.len() * 2) as u32;
    let byte_rate = sample_rate * 2;

    file.write_all(b"RIFF").map_err(|err| err.to_string())?;
    file.write_all(&(36 + data_len).to_le_bytes())
        .map_err(|err| err.to_string())?;
    file.write_all(b"WAVEfmt ").map_err(|err| err.to_string())?;
    file.write_all(&16u32.to_le_bytes())
        .map_err(|err| err.to_string())?;
    file.write_all(&1u16.to_le_bytes())
        .map_err(|err| err.to_string())?;
    file.write_all(&1u16.to_le_bytes())
        .map_err(|err| err.to_string())?;
    file.write_all(&sample_rate.to_le_bytes())
        .map_err(|err| err.to_string())?;
    file.write_all(&byte_rate.to_le_bytes())
        .map_err(|err| err.to_string())?;
    file.write_all(&2u16.to_le_bytes())
        .map_err(|err| err.to_string())?;
    file.write_all(&16u16.to_le_bytes())
        .map_err(|err| err.to_string())?;
    file.write_all(b"data").map_err(|err| err.to_string())?;
    file.write_all(&data_len.to_le_bytes())
        .map_err(|err| err.to_string())?;
    for sample in samples {
        file.write_all(&sample.to_le_bytes())
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn resolve_existing_path(repo: &Path, base: &Path, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        return path;
    }

    let repo_path = repo.join(&path);
    if repo_path.exists() {
        repo_path
    } else {
        base.join(path)
    }
}

fn read_instrument(path: &Path) -> Result<VandInstrument, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("invalid instrument JSON: {err}"))
}

fn write_fm_patch(audio: &mut Audio, instrument: &VandInstrument, channel: usize, attenuation: u8) {
    const OP_OFFSETS: [u8; 4] = [0x00, 0x04, 0x08, 0x0c];
    let Some(fm) = instrument.fm.as_ref() else {
        return;
    };

    audio.ym2612.write_register(
        0,
        0xb0 + channel as u8,
        (fm.algorithm & 0x07) | ((fm.feedback & 0x07) << 3),
        None,
        true,
    );
    audio
        .ym2612
        .write_register(0, 0xb4 + channel as u8, 0xc0, None, true);
    for (index, op) in fm.operators.iter().take(4).enumerate() {
        let reg = OP_OFFSETS[index] + channel as u8;
        audio.ym2612.write_register(
            0,
            0x30 + reg,
            ((op.dt & 0x07) << 4) | (op.mult & 0x0f),
            None,
            true,
        );
        audio.ym2612.write_register(
            0,
            0x40 + reg,
            op.tl.saturating_add(attenuation).min(127),
            None,
            true,
        );
        audio
            .ym2612
            .write_register(0, 0x50 + reg, op.ar.min(31), None, true);
        audio
            .ym2612
            .write_register(0, 0x60 + reg, op.dr.min(31), None, true);
        audio.ym2612.write_register(0, 0x70 + reg, 0x04, None, true);
        audio.ym2612.write_register(
            0,
            0x80 + reg,
            ((op.sl.min(15)) << 4) | op.rr.min(15),
            None,
            true,
        );
        audio.ym2612.write_register(0, 0x90 + reg, 0x00, None, true);
    }
}

fn write_fm_pitch(audio: &mut Audio, channel: usize, hz: f32) {
    let (fnum, block) = ym2612_pitch_for_hz(hz);
    audio.ym2612.write_register(
        0,
        0xa4 + channel as u8,
        ((block & 0x07) << 3) | ((fnum >> 8) as u8 & 0x07),
        None,
        true,
    );
    audio
        .ym2612
        .write_register(0, 0xa0 + channel as u8, (fnum & 0xff) as u8, None, true);
}

fn fm_key_on(audio: &mut Audio, channel: usize, on: bool) {
    let value = if on {
        0xf0 | channel as u8
    } else {
        channel as u8
    };
    audio.ym2612.write_register(0, 0x28, value, Some(0), true);
}

fn amp_to_psg_volume(amp: f32) -> u8 {
    let amp = amp.clamp(0.0, 1.0);
    (15.0 - amp * 14.0).round().clamp(0.0, 15.0) as u8
}

fn render_instrument_samples(instrument: &VandInstrument) -> Vec<i16> {
    const RATE: u32 = 44_100;
    const FRAME_SAMPLES: usize = 735;
    const FRAMES: usize = 96;
    const KEY_OFF_FRAME: usize = 62;
    const CHANNEL: usize = 0;
    let mut audio = Audio::new();
    audio.reset();
    let mut out = Vec::with_capacity(FRAMES * FRAME_SAMPLES);
    let name_lc = instrument.name.to_ascii_lowercase();
    let preview_hz = if instrument.category == "bass" || name_lc.contains("bass") {
        110.0
    } else {
        220.0
    };

    if instrument.fm.is_some() {
        audio.ym2612.write_register(0, 0x22, 0x00, None, true);
        audio.ym2612.write_register(0, 0x27, 0x00, None, true);
        write_fm_patch(&mut audio, instrument, CHANNEL, 0);
        write_fm_pitch(&mut audio, CHANNEL, preview_hz);
    } else if instrument.chip == "sn76489" {
        psg_set_tone(&mut audio, 0, 440.0, 3);
    }

    for frame in 0..FRAMES {
        audio.begin_frame();
        if instrument.fm.is_some() {
            if frame == 0 {
                audio
                    .ym2612
                    .write_register(0, 0x28, 0xf0 | CHANNEL as u8, Some(0), true);
            } else if frame == KEY_OFF_FRAME {
                audio
                    .ym2612
                    .write_register(0, 0x28, CHANNEL as u8, Some(0), true);
            }
        } else if instrument.chip == "sn76489" && frame == KEY_OFF_FRAME {
            psg_latch_volume(&mut audio, 0, 15);
        }
        out.extend(render_preview_frame_i16(
            &mut audio,
            FRAME_SAMPLES,
            RATE as usize,
        ));
    }
    out
}

fn normalize_preview_samples(samples: &mut [i16], target_peak: i16) {
    let peak = samples
        .iter()
        .map(|sample| i32::from(*sample).abs())
        .max()
        .unwrap_or(0);
    if peak < 1 {
        return;
    }

    let gain = f32::from(target_peak) / peak as f32;
    if gain <= 1.0 {
        return;
    }

    for sample in samples {
        let value = (*sample as f32 * gain).round() as i32;
        *sample = value.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    }
}

fn render_preview_frame_i16(
    audio: &mut Audio,
    frame_samples: usize,
    sample_rate: usize,
) -> Vec<i16> {
    audio.ym2612.tick(audio.ym_frame_cycles.round() as i64);
    audio
        .render_frame_samples(frame_samples, sample_rate)
        .into_iter()
        .map(|sample| (sample.clamp(-1.0, 1.0) * 32767.0) as i16)
        .collect()
}

fn ym2612_pitch_for_hz(hz: f32) -> (u16, u8) {
    const YM2612_CLOCK: f32 = 7_670_454.0;
    let base = YM2612_CLOCK / 144.0;
    let mut best = (0x400u16, 4u8, f32::MAX);
    for block in 1..=7u8 {
        let divider = 2.0f32.powi(block as i32 - 1);
        let fnum = (hz * (1u32 << 20) as f32 / base / divider).round();
        if (256.0..=2047.0).contains(&fnum) {
            let score = (fnum - 1024.0).abs();
            if score < best.2 {
                best = (fnum as u16, block, score);
            }
        }
    }
    (best.0, best.1)
}

fn psg_write_latch(audio: &mut Audio, channel: u8, volume: bool, data: u8) {
    audio.psg.write(
        0x80 | ((channel & 0x03) << 5) | if volume { 0x10 } else { 0x00 } | (data & 0x0f),
        None,
        None,
    );
}

fn psg_latch_volume(audio: &mut Audio, channel: u8, volume: u8) {
    psg_write_latch(audio, channel, true, volume & 0x0f);
}

fn psg_set_tone(audio: &mut Audio, channel: u8, hz: f64, volume: u8) {
    let period = (3_579_545.0 / (32.0 * hz)).round().clamp(1.0, 1023.0) as u16;
    psg_write_latch(audio, channel, false, (period & 0x0f) as u8);
    audio.psg.write(((period >> 4) & 0x3f) as u8, None, None);
    psg_latch_volume(audio, channel, volume);
}

fn psg_set_noise(audio: &mut Audio, amp: f32) {
    audio.psg.write(0xe7, None, None);
    psg_latch_volume(audio, 3, amp_to_psg_volume(amp));
}

fn load_instrument_bank_for_arrangement(
    repo: &Path,
    arrangement_path: &Path,
    arrangement: &VandArrangement,
    explicit_bank_path: &str,
) -> Result<HashMap<String, VandInstrument>, String> {
    let arrangement_dir = arrangement_path.parent().unwrap_or_else(|| Path::new("."));
    load_instrument_bank_from_reference(
        repo,
        arrangement_dir,
        &arrangement.instrument_bank,
        explicit_bank_path,
    )
}

fn load_instrument_bank_for_note_arrangement(
    repo: &Path,
    arrangement_path: &Path,
    arrangement: &VandNoteArrangement,
    explicit_bank_path: &str,
) -> Result<HashMap<String, VandInstrument>, String> {
    let arrangement_dir = arrangement_path.parent().unwrap_or_else(|| Path::new("."));
    load_instrument_bank_from_reference(
        repo,
        arrangement_dir,
        &arrangement.instrument_bank,
        explicit_bank_path,
    )
}

fn load_instrument_bank_from_reference(
    repo: &Path,
    arrangement_dir: &Path,
    instrument_bank: &str,
    explicit_bank_path: &str,
) -> Result<HashMap<String, VandInstrument>, String> {
    let bank_path = if !explicit_bank_path.is_empty() {
        PathBuf::from(explicit_bank_path)
    } else if !instrument_bank.is_empty() {
        resolve_existing_path(repo, arrangement_dir, instrument_bank)
    } else {
        repo.join(DEFAULT_INSTRUMENT_BANK)
    };
    let bank_dir = bank_path.parent().unwrap_or_else(|| Path::new("."));
    let manifest = read_instrument_bank(&bank_path)?;
    let mut instruments = HashMap::new();

    for entry in manifest.instruments {
        let instrument_path = resolve_existing_path(repo, bank_dir, &entry.file);
        let instrument = read_instrument(&instrument_path)?;
        instruments.insert(entry.id, instrument);
    }

    Ok(instruments)
}

fn frame_sample_count(source_rate: u32, hop: usize) -> usize {
    const RATE: u32 = 44_100;
    ((hop as u64 * RATE as u64 + (source_rate / 2) as u64) / source_rate.max(1) as u64).max(64)
        as usize
}

fn render_arrangement_samples(
    arrangement: &VandArrangement,
    instruments: &HashMap<String, VandInstrument>,
) -> Vec<i16> {
    const RATE: u32 = 44_100;
    let frame_samples = frame_sample_count(arrangement.sample_rate, arrangement.hop);
    let mut audio = Audio::new();
    audio.reset();
    audio.ym2612.write_register(0, 0x22, 0x00, None, true);
    audio.ym2612.write_register(0, 0x27, 0x00, None, true);
    psg_latch_volume(&mut audio, 3, 15);

    let mut out = Vec::with_capacity(arrangement.events.len() * frame_samples);
    let mut bass_id = String::new();
    let mut lead_id = String::new();
    let mut bass_on = false;
    let mut lead_on = false;

    for frame in &arrangement.events {
        audio.begin_frame();

        if frame.bass_hz > 0.0 && frame.bass_amp > 0.01 {
            let next_bass_id = if frame.bass_instrument_id.is_empty() {
                DEFAULT_BASS_INSTRUMENT
            } else {
                &frame.bass_instrument_id
            };
            if bass_id != next_bass_id {
                bass_id = next_bass_id.to_string();
                if let Some(instrument) = instruments.get(&bass_id) {
                    write_fm_patch(&mut audio, instrument, 0, 0);
                }
                bass_on = false;
            }
            write_fm_pitch(&mut audio, 0, frame.bass_hz);
            if !bass_on {
                fm_key_on(&mut audio, 0, true);
                bass_on = true;
            }
        } else if bass_on {
            fm_key_on(&mut audio, 0, false);
            bass_on = false;
        }

        if frame.lead_hz > 0.0 && frame.lead_amp > 0.01 {
            let next_lead_id = if frame.lead_instrument_id.is_empty() {
                DEFAULT_LEAD_INSTRUMENT
            } else {
                &frame.lead_instrument_id
            };
            if lead_id != next_lead_id {
                lead_id = next_lead_id.to_string();
                if let Some(instrument) = instruments.get(&lead_id) {
                    write_fm_patch(&mut audio, instrument, 1, 0);
                }
                lead_on = false;
            }
            write_fm_pitch(&mut audio, 1, frame.lead_hz);
            if !lead_on {
                fm_key_on(&mut audio, 1, true);
                lead_on = true;
            }
        } else if lead_on {
            fm_key_on(&mut audio, 1, false);
            lead_on = false;
        }

        let noise_amp = frame.noise_amp.max(frame.transient * 0.65);
        let noise_id = if frame.noise_instrument_id.is_empty() {
            DEFAULT_NOISE_INSTRUMENT
        } else {
            &frame.noise_instrument_id
        };
        let use_noise = matches!(frame.class_name.as_str(), "drum" | "sample");
        if use_noise && instruments.contains_key(noise_id) && noise_amp > 0.12 {
            psg_set_noise(&mut audio, noise_amp);
        } else {
            psg_latch_volume(&mut audio, 3, 15);
        }

        out.extend(render_preview_frame_i16(
            &mut audio,
            frame_samples,
            RATE as usize,
        ));
    }

    if bass_on {
        fm_key_on(&mut audio, 0, false);
    }
    if lead_on {
        fm_key_on(&mut audio, 1, false);
    }

    out
}

fn render_note_arrangement_samples(
    arrangement: &VandNoteArrangement,
    instruments: &HashMap<String, VandInstrument>,
    arrangement_dir: &Path,
) -> Vec<i16> {
    const RATE: u32 = 44_100;
    let frame_samples = frame_sample_count(arrangement.sample_rate, arrangement.hop);
    let mut audio = Audio::new();
    audio.reset();
    audio.ym2612.write_register(0, 0x22, 0x00, None, true);
    audio.ym2612.write_register(0, 0x27, 0x00, None, true);
    psg_latch_volume(&mut audio, 3, 15);

    let mut notes = arrangement.notes.clone();
    notes.sort_by_key(|note| note.start_frame);
    let mut drums = arrangement.drums.clone();
    drums.sort_by_key(|drum| drum.start_frame);

    let inferred_frames = notes
        .iter()
        .map(|note| note.start_frame.saturating_add(note.frames))
        .chain(drums.iter().map(|drum| drum.start_frame.saturating_add(3)))
        .max()
        .unwrap_or(0);
    let total_frames = arrangement.total_frames.max(inferred_frames).max(1);
    let mut out = Vec::with_capacity(total_frames * frame_samples);
    let mut note_index = 0usize;
    let mut drum_index = 0usize;
    let mut bass_id = String::new();
    let mut lead_id = String::new();
    let mut bass_end = 0usize;
    let mut lead_end = 0usize;
    let mut bass_on = false;
    let mut lead_on = false;
    let mut noise_until = 0usize;
    let mut noise_amp = 0.0f32;

    for frame in 0..total_frames {
        audio.begin_frame();

        while note_index < notes.len() && notes[note_index].start_frame == frame {
            let note = &notes[note_index];
            let (channel, current_id, active, end_frame, fallback) = if note.track == "bass" {
                (
                    0usize,
                    &mut bass_id,
                    &mut bass_on,
                    &mut bass_end,
                    DEFAULT_BASS_INSTRUMENT,
                )
            } else {
                (
                    1usize,
                    &mut lead_id,
                    &mut lead_on,
                    &mut lead_end,
                    DEFAULT_LEAD_INSTRUMENT,
                )
            };
            let next_id = if note.instrument_id.is_empty() {
                fallback
            } else {
                &note.instrument_id
            };
            if current_id.as_str() != next_id {
                *current_id = next_id.to_string();
                if *active {
                    fm_key_on(&mut audio, channel, false);
                    *active = false;
                }
            }
            if *active {
                fm_key_on(&mut audio, channel, false);
                *active = false;
            }
            if let Some(instrument) = instruments.get(current_id.as_str()) {
                let attenuation = ((1.0 - note.velocity.clamp(0.0, 1.0)) * 22.0) as u8;
                write_fm_patch(&mut audio, instrument, channel, attenuation);
            }
            write_fm_pitch(&mut audio, channel, note.hz);
            fm_key_on(&mut audio, channel, true);
            *active = true;
            *end_frame = note.start_frame.saturating_add(note.frames);
            note_index += 1;
        }

        if bass_on && frame >= bass_end {
            fm_key_on(&mut audio, 0, false);
            bass_on = false;
        }
        if lead_on && frame >= lead_end {
            fm_key_on(&mut audio, 1, false);
            lead_on = false;
        }

        while drum_index < drums.len() && drums[drum_index].start_frame == frame {
            let drum = &drums[drum_index];
            let id = if drum.instrument_id.is_empty() {
                DEFAULT_NOISE_INSTRUMENT
            } else {
                &drum.instrument_id
            };
            if instruments.contains_key(id) {
                noise_amp = drum.noise_amp.max(drum.velocity * 0.65).clamp(0.0, 1.0);
                noise_until = frame.saturating_add(3);
            }
            drum_index += 1;
        }
        if frame < noise_until && noise_amp > 0.05 {
            psg_set_noise(&mut audio, noise_amp);
        } else {
            psg_latch_volume(&mut audio, 3, 15);
        }

        out.extend(render_preview_frame_i16(
            &mut audio,
            frame_samples,
            RATE as usize,
        ));
    }

    mix_dac_drums(&mut out, &drums, arrangement_dir, frame_samples);
    out
}

fn mix_dac_drums(
    out: &mut [i16],
    drums: &[VandDrumEvent],
    arrangement_dir: &Path,
    frame_samples: usize,
) {
    const PREVIEW_RATE: usize = 44_100;
    const DAC_RATE: usize = 13_320;
    let mut cache: HashMap<String, Vec<u8>> = HashMap::new();

    for drum in drums {
        if drum.dac_chunk.is_none() || drum.dac_path.is_empty() {
            continue;
        }
        let bytes = if let Some(bytes) = cache.get(&drum.dac_path) {
            bytes
        } else {
            let path = resolve_existing_path(Path::new(""), arrangement_dir, &drum.dac_path);
            let Ok(bytes) = fs::read(path) else {
                continue;
            };
            cache.insert(drum.dac_path.clone(), bytes);
            cache.get(&drum.dac_path).unwrap()
        };
        if bytes.is_empty() {
            continue;
        }

        let start = drum.start_frame.saturating_mul(frame_samples);
        if start >= out.len() {
            continue;
        }
        let gain = drum.velocity.max(drum.noise_amp).clamp(0.0, 1.0) * 0.55;
        let preview_len = ((bytes.len() as u64 * PREVIEW_RATE as u64 + (DAC_RATE / 2) as u64)
            / DAC_RATE as u64)
            .max(1) as usize;
        for i in 0..preview_len {
            let out_index = start + i;
            if out_index >= out.len() {
                break;
            }
            let src_index = (i * DAC_RATE / PREVIEW_RATE).min(bytes.len() - 1);
            let centered = bytes[src_index] as f32 - 128.0;
            let sample = (centered / 128.0 * 32767.0 * gain).round() as i32;
            let mixed = i32::from(out[out_index]) + sample;
            out[out_index] = mixed.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        }
    }
}

fn parse_metric(log: &str, label: &str) -> u32 {
    let Some(index) = log.find(label) else {
        return 0;
    };
    log[..index]
        .split_whitespace()
        .last()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0)
}

async fn run_blocking<T, F>(work: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|err| format!("background task failed: {err}"))?
}

#[tauri::command]
fn pick_audio_file() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Open audio")
        .add_filter("Audio", &["mp3", "ogg", "wav", "flac", "aiff", "aif"])
        .pick_file()
        .map(|path| path.display().to_string())
}

#[tauri::command]
fn pick_output_dir() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Choose output folder")
        .pick_folder()
        .map(|path| path.display().to_string())
}

#[tauri::command]
fn pick_instrument_dir() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Choose Furnace instrument folder")
        .pick_folder()
        .map(|path| path.display().to_string())
}

#[tauri::command]
fn pick_instrument_bank() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Open Vand-AI-lism instrument bank")
        .add_filter("Vand instrument bank", &["json"])
        .pick_file()
        .map(|path| path.display().to_string())
}

fn read_instrument_bank(path: &Path) -> Result<InstrumentBankManifest, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("invalid bank manifest: {err}"))
}

#[tauri::command]
fn load_settings() -> Result<AudioLabSettings, String> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(AudioLabSettings::default());
    }
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    toml::from_str(&text).map_err(|err| format!("invalid settings TOML: {err}"))
}

#[tauri::command]
fn save_settings(settings: AudioLabSettings) -> Result<(), String> {
    let path = settings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let text = toml::to_string_pretty(&settings)
        .map_err(|err| format!("failed to encode settings TOML: {err}"))?;
    fs::write(&path, text).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

#[tauri::command]
fn load_instrument_bank(path: String) -> Result<InstrumentBankResult, String> {
    let path = PathBuf::from(path);
    let manifest = read_instrument_bank(&path)?;
    Ok(InstrumentBankResult {
        manifest,
        manifest_path: path.display().to_string(),
        log: "Loaded instrument bank.".to_string(),
    })
}

#[tauri::command]
async fn analyse_audio(input: String, output_dir: String) -> Result<AnalysisResult, String> {
    run_blocking(move || {
        let repo = repo_root()?;
        let input_path = PathBuf::from(&input);
        let output_dir = resolve_output_dir(&repo, &PathBuf::from(&output_dir));
        let output_path = bundle_path(&input_path, &output_dir);
        let lab_manifest = repo.join("tools/audio/audio_lab/Cargo.toml");

        let output = Command::new("cargo")
            .current_dir(&repo)
            .arg("run")
            .arg("--manifest-path")
            .arg(&lab_manifest)
            .arg("--")
            .arg("analyse-audio")
            .arg(&input_path)
            .arg("--out")
            .arg(&output_path)
            .output()
            .map_err(|err| format!("failed to start audio lab: {err}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let log = format!("{stdout}{stderr}");
        if !output.status.success() {
            return Err(log);
        }

        let bundle_dir = output_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| "analysis output path has no parent".to_string())?;
        let preview_report_path = bundle_dir.join("preview_report.json");
        let preview_report = fs::read_to_string(&preview_report_path)
            .ok()
            .and_then(|text| serde_json::from_str::<PreviewReport>(&text).ok())
            .unwrap_or_default();
        let summary = AnalysisSummary {
            frames: parse_metric(&log, "frames,"),
            active_frames: parse_metric(&log, "active,"),
            runtime_events: parse_metric(&log, "runtime events,"),
            dac_chunks: parse_metric(&log, "dac chunks"),
            arrangement: output_path.display().to_string(),
            note_arrangement: bundle_dir
                .join("note_arrangement.vand-audio.json")
                .display()
                .to_string(),
            preview_report: preview_report_path.display().to_string(),
            key: preview_report.key,
            bpm: preview_report.bpm,
            bass_notes: preview_report.bass_notes,
            lead_notes: preview_report.lead_notes,
            drum_events: preview_report.drum_events,
            import_metadata: bundle_dir.join("import.json").display().to_string(),
            dac_preview: bundle_dir.join("dac_preview.wav").display().to_string(),
            bundle_dir: bundle_dir.display().to_string(),
        };

        Ok(AnalysisResult { summary, log })
    })
    .await
}

#[tauri::command]
async fn import_instrument_dir(
    input_dir: String,
    output_dir: String,
) -> Result<InstrumentBankResult, String> {
    run_blocking(move || {
        let repo = repo_root()?;
        let input_path = PathBuf::from(&input_dir);
        let output_dir = resolve_output_dir(&repo, &PathBuf::from(&output_dir));
        let bank_name = input_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("bank");
        let output_path = output_dir.join(format!("{bank_name}.vand-instruments"));
        let lab_manifest = repo.join("tools/audio/audio_lab/Cargo.toml");

        let output = Command::new("cargo")
            .current_dir(&repo)
            .arg("run")
            .arg("--manifest-path")
            .arg(&lab_manifest)
            .arg("--")
            .arg("import-instrument-dir")
            .arg(&input_path)
            .arg("--out")
            .arg(&output_path)
            .output()
            .map_err(|err| format!("failed to start audio lab: {err}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let log = format!("{stdout}{stderr}");
        if !output.status.success() {
            return Err(log);
        }

        let manifest_path = output_path.join("bank.vand-instruments.json");
        let manifest = read_instrument_bank(&manifest_path)?;
        Ok(InstrumentBankResult {
            manifest,
            manifest_path: manifest_path.display().to_string(),
            log,
        })
    })
    .await
}

#[tauri::command]
async fn audio_data_url(path: String) -> Result<String, String> {
    run_blocking(move || {
        let path = PathBuf::from(path);
        let mime = mime_for_path(&path)?;
        let bytes =
            fs::read(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        Ok(format!("data:{mime};base64,{}", base64_encode(&bytes)))
    })
    .await
}

#[tauri::command]
async fn instrument_preview_data_url(path: String) -> Result<String, String> {
    run_blocking(move || {
        let path = PathBuf::from(path);
        let instrument = read_instrument(&path)?;
        let mut samples = render_instrument_samples(&instrument);
        normalize_preview_samples(&mut samples, 22_000);
        let preview = std::env::temp_dir().join("vand-ai-lism-instrument-preview.wav");
        write_preview_wav(&preview, 44_100, &samples)?;
        let bytes = fs::read(&preview)
            .map_err(|err| format!("failed to read {}: {err}", preview.display()))?;
        Ok(format!("data:audio/wav;base64,{}", base64_encode(&bytes)))
    })
    .await
}

#[tauri::command]
async fn arrangement_preview_data_url(path: String, bank_path: String) -> Result<String, String> {
    run_blocking(move || {
        let repo = repo_root()?;
        let arrangement_path = PathBuf::from(path);
        let text = fs::read_to_string(&arrangement_path)
            .map_err(|err| format!("failed to read {}: {err}", arrangement_path.display()))?;
        let arrangement: VandArrangement = serde_json::from_str(&text)
            .map_err(|err| format!("invalid arrangement JSON: {err}"))?;
        let instruments = load_instrument_bank_for_arrangement(
            &repo,
            &arrangement_path,
            &arrangement,
            &bank_path,
        )?;
        let mut samples = render_arrangement_samples(&arrangement, &instruments);
        normalize_preview_samples(&mut samples, 22_000);
        let preview = std::env::temp_dir().join("vand-ai-lism-arrangement-preview.wav");
        write_preview_wav(&preview, 44_100, &samples)?;
        let bytes = fs::read(&preview)
            .map_err(|err| format!("failed to read {}: {err}", preview.display()))?;
        Ok(format!("data:audio/wav;base64,{}", base64_encode(&bytes)))
    })
    .await
}

#[tauri::command]
async fn note_preview_data_url(path: String, bank_path: String) -> Result<String, String> {
    run_blocking(move || {
        let repo = repo_root()?;
        let arrangement_path = PathBuf::from(path);
        let text = fs::read_to_string(&arrangement_path)
            .map_err(|err| format!("failed to read {}: {err}", arrangement_path.display()))?;
        let arrangement: VandNoteArrangement = serde_json::from_str(&text)
            .map_err(|err| format!("invalid note arrangement JSON: {err}"))?;
        let instruments = load_instrument_bank_for_note_arrangement(
            &repo,
            &arrangement_path,
            &arrangement,
            &bank_path,
        )?;
        let arrangement_dir = arrangement_path.parent().unwrap_or_else(|| Path::new("."));
        let mut samples =
            render_note_arrangement_samples(&arrangement, &instruments, arrangement_dir);
        normalize_preview_samples(&mut samples, 22_000);
        let preview = std::env::temp_dir().join("vand-ai-lism-note-preview.wav");
        write_preview_wav(&preview, 44_100, &samples)?;
        let bytes = fs::read(&preview)
            .map_err(|err| format!("failed to read {}: {err}", preview.display()))?;
        Ok(format!("data:audio/wav;base64,{}", base64_encode(&bytes)))
    })
    .await
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            pick_audio_file,
            pick_output_dir,
            pick_instrument_dir,
            pick_instrument_bank,
            load_settings,
            save_settings,
            load_instrument_bank,
            analyse_audio,
            import_instrument_dir,
            audio_data_url,
            instrument_preview_data_url,
            arrangement_preview_data_url,
            note_preview_data_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vand-AI-lism");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_arrangement_preview_with_core_bank() {
        let repo = repo_root().expect("repo root");
        let arrangement_path = repo.join("audio/converted/test-arrangement.vand-audio.json");
        let arrangement = VandArrangement {
            sample_rate: 44_100,
            hop: 512,
            instrument_bank: DEFAULT_INSTRUMENT_BANK.to_string(),
            events: vec![
                VandArrangementFrame {
                    class_name: "bass".to_string(),
                    bass_hz: 110.0,
                    bass_amp: 0.8,
                    bass_instrument_id: DEFAULT_BASS_INSTRUMENT.to_string(),
                    lead_hz: 220.0,
                    lead_amp: 0.5,
                    lead_instrument_id: DEFAULT_LEAD_INSTRUMENT.to_string(),
                    noise_amp: 0.0,
                    transient: 0.0,
                    noise_instrument_id: String::new(),
                },
                VandArrangementFrame {
                    class_name: "drum".to_string(),
                    bass_hz: 123.47,
                    bass_amp: 0.7,
                    bass_instrument_id: DEFAULT_BASS_INSTRUMENT.to_string(),
                    lead_hz: 246.94,
                    lead_amp: 0.4,
                    lead_instrument_id: DEFAULT_LEAD_INSTRUMENT.to_string(),
                    noise_amp: 0.2,
                    transient: 0.1,
                    noise_instrument_id: DEFAULT_NOISE_INSTRUMENT.to_string(),
                },
            ],
        };
        let instruments =
            load_instrument_bank_for_arrangement(&repo, &arrangement_path, &arrangement, "")
                .expect("core bank");
        let mut samples = render_arrangement_samples(&arrangement, &instruments);
        assert!(!samples.is_empty());
        assert!(samples.iter().any(|sample| *sample != 0));
        normalize_preview_samples(&mut samples, 22_000);
        let peak = samples
            .iter()
            .map(|sample| i32::from(*sample).abs())
            .max()
            .unwrap_or(0);
        assert!(peak >= 18_000, "preview peak too low: {peak}");
    }

    #[test]
    fn renders_note_preview_with_core_bank() {
        let repo = repo_root().expect("repo root");
        let arrangement_path = repo.join("audio/converted/test-note-arrangement.vand-audio.json");
        let arrangement = VandNoteArrangement {
            sample_rate: 44_100,
            hop: 512,
            total_frames: 96,
            instrument_bank: DEFAULT_INSTRUMENT_BANK.to_string(),
            notes: vec![
                VandNoteEvent {
                    track: "bass".to_string(),
                    start_frame: 0,
                    frames: 48,
                    hz: 110.0,
                    velocity: 0.8,
                    instrument_id: DEFAULT_BASS_INSTRUMENT.to_string(),
                },
                VandNoteEvent {
                    track: "lead".to_string(),
                    start_frame: 16,
                    frames: 32,
                    hz: 220.0,
                    velocity: 0.7,
                    instrument_id: DEFAULT_LEAD_INSTRUMENT.to_string(),
                },
            ],
            drums: vec![VandDrumEvent {
                start_frame: 24,
                velocity: 0.8,
                noise_amp: 0.5,
                instrument_id: DEFAULT_NOISE_INSTRUMENT.to_string(),
                dac_chunk: None,
                dac_path: String::new(),
            }],
        };
        let instruments =
            load_instrument_bank_for_note_arrangement(&repo, &arrangement_path, &arrangement, "")
                .expect("core bank");
        let mut samples = render_note_arrangement_samples(
            &arrangement,
            &instruments,
            arrangement_path.parent().unwrap(),
        );
        assert!(!samples.is_empty());
        assert!(samples.iter().any(|sample| *sample != 0));
        normalize_preview_samples(&mut samples, 22_000);
        let peak = samples
            .iter()
            .map(|sample| i32::from(*sample).abs())
            .max()
            .unwrap_or(0);
        assert!(peak >= 18_000, "note preview peak too low: {peak}");
    }

    #[test]
    fn serializes_audio_lab_settings_as_toml() {
        let mut presets = HashMap::new();
        presets.insert("preview".to_string(), "note".to_string());
        let settings = AudioLabSettings {
            source: "/tmp/song.ogg".to_string(),
            output_dir: "audio/converted".to_string(),
            instrument_bank: "audio/instruments/core.json".to_string(),
            presets,
        };
        let text = toml::to_string_pretty(&settings).expect("settings toml");
        assert!(text.contains("source = \"/tmp/song.ogg\""));
        assert!(text.contains("[presets]"));
        let decoded: AudioLabSettings = toml::from_str(&text).expect("decode settings");
        assert_eq!(decoded.source, settings.source);
        assert_eq!(decoded.output_dir, settings.output_dir);
        assert_eq!(decoded.instrument_bank, settings.instrument_bank);
        assert_eq!(
            decoded.presets.get("preview").map(String::as_str),
            Some("note")
        );
    }

    #[test]
    fn mixes_dac_drum_chunks_into_preview_buffer() {
        let dir = std::env::temp_dir().join("vand-ai-lism-dac-mix-test");
        fs::create_dir_all(dir.join("dac_chunks")).expect("temp chunk dir");
        fs::write(
            dir.join("dac_chunks/chunk_00.u8"),
            [128u8, 255, 0, 220, 128],
        )
        .expect("chunk bytes");
        let drums = vec![VandDrumEvent {
            start_frame: 1,
            velocity: 1.0,
            noise_amp: 0.0,
            instrument_id: DEFAULT_NOISE_INSTRUMENT.to_string(),
            dac_chunk: Some(0),
            dac_path: "dac_chunks/chunk_00.u8".to_string(),
        }];
        let mut out = vec![0i16; 256];
        mix_dac_drums(&mut out, &drums, &dir, 32);
        assert!(out.iter().any(|sample| *sample != 0));
    }
}
