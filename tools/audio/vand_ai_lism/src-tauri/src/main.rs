use euther_oxide::audio::Audio;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_INSTRUMENT_BANK: &str =
    "audio/instruments/vand_furnace_core/bank.vand-instruments.json";
const DEFAULT_BASS_INSTRUMENT: &str = "growl_bass_wobbly";
const DEFAULT_LEAD_INSTRUMENT: &str = "fm_grinder";
const DEFAULT_PAD_INSTRUMENT: &str = "growl";
const DEFAULT_NOISE_INSTRUMENT: &str = "psg_echo_warble";

#[derive(Clone, Copy)]
struct RenderMix {
    bass_gain: f32,
    lead_gain: f32,
    psg_gain: f32,
    dac_gain: f32,
}

impl Default for RenderMix {
    fn default() -> Self {
        Self {
            bass_gain: 1.0,
            lead_gain: 0.85,
            psg_gain: 0.85,
            dac_gain: 1.0,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct AudioLabSettings {
    #[serde(default)]
    source: String,
    #[serde(default = "default_output_dir")]
    output_dir: String,
    #[serde(default)]
    instrument_bank: String,
    #[serde(default = "default_ai_backend")]
    ai_backend: String,
    #[serde(default = "default_ai_home")]
    ai_home: String,
    #[serde(default)]
    ai_tool_dir: String,
    #[serde(default)]
    presets: HashMap<String, String>,
}

impl Default for AudioLabSettings {
    fn default() -> Self {
        Self {
            source: String::new(),
            output_dir: default_output_dir(),
            instrument_bank: String::new(),
            ai_backend: default_ai_backend(),
            ai_home: default_ai_home(),
            ai_tool_dir: String::new(),
            presets: HashMap::new(),
        }
    }
}

fn default_output_dir() -> String {
    "audio/converted".to_string()
}

fn default_ai_backend() -> String {
    "off".to_string()
}

fn default_ai_home() -> String {
    "/home/nichlas/ai".to_string()
}

fn set_default_env(key: &str, value: &str) {
    if env::var_os(key).is_none() {
        // This runs before Tauri/WebKitGTK starts worker threads.
        unsafe {
            env::set_var(key, value);
        }
    }
}

fn configure_native_wayland_environment() {
    set_default_env("GDK_BACKEND", "wayland");
    set_default_env("GTK_THEME", "Adwaita");
    set_default_env("GTK_IM_MODULE", "gtk-im-context-simple");
    set_default_env("QT_IM_MODULE", "simple");
    set_default_env("XMODIFIERS", "@im=none");
    set_default_env("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
}

#[derive(Serialize)]
struct AnalysisSummary {
    frames: u32,
    active_frames: u32,
    runtime_events: u32,
    dac_chunks: u32,
    arrangement: String,
    note_arrangement: String,
    render_plan: String,
    preview_report: String,
    key: String,
    bpm: f32,
    bass_notes: u32,
    lead_notes: u32,
    drum_events: u32,
    loop_start: f32,
    loop_end: f32,
    bundle_dir: String,
    import_metadata: String,
    dac_preview: String,
    runtime_preview: String,
    ai_manifest: String,
    ai_notes: String,
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
    #[serde(default)]
    loop_start: f32,
    #[serde(default)]
    loop_end: f32,
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
    #[serde(default)]
    role: String,
    #[serde(default)]
    tags: Vec<String>,
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
    #[serde(default)]
    role: String,
    #[serde(default)]
    tags: Vec<String>,
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

#[derive(Deserialize)]
struct VandRenderPlan {
    #[serde(default = "default_sample_rate")]
    sample_rate: u32,
    #[serde(default = "default_arrangement_hop")]
    hop: usize,
    #[serde(default)]
    total_frames: usize,
    #[serde(default)]
    instrument_bank: String,
    tracks: Vec<VandRenderTrack>,
    #[serde(default)]
    drums: Vec<VandRenderDrum>,
}

#[derive(Deserialize)]
struct VandRenderTrack {
    role: String,
    events: Vec<VandRenderNoteEvent>,
}

#[derive(Deserialize)]
struct VandRenderNoteEvent {
    start_frame: usize,
    frames: usize,
    hz: f32,
    velocity: f32,
    instrument_id: String,
}

#[derive(Deserialize)]
struct VandRenderDrum {
    start_frame: usize,
    velocity: f32,
    #[serde(default)]
    psg_noise_amp: f32,
    #[serde(default)]
    psg_instrument_id: String,
    #[serde(default)]
    dac_chunk: Option<usize>,
    #[serde(default)]
    dac_path: String,
}

#[derive(Deserialize)]
struct VandAiNotes {
    #[serde(default = "default_sample_rate")]
    sample_rate: u32,
    #[serde(default = "default_arrangement_hop")]
    hop: usize,
    #[serde(default)]
    total_seconds: f32,
    tracks: Vec<VandAiNoteTrack>,
}

#[derive(Deserialize)]
struct VandAiNoteTrack {
    role: String,
    events: Vec<VandAiNoteEvent>,
}

#[derive(Clone, Deserialize)]
struct VandAiNoteEvent {
    start_time: f32,
    duration: f32,
    hz: f32,
    midi: i32,
    velocity: f32,
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

fn role_matches(track: &str, role: &str) -> bool {
    match track {
        "bass" => matches!(role, "bass"),
        "pad" => matches!(role, "pad" | "chord"),
        "chord" => matches!(role, "chord" | "pad" | "keys"),
        "counter" | "hook" | "lead" => matches!(role, "lead" | "hook"),
        _ => false,
    }
}

fn choose_fm_instrument_id(
    instruments: &HashMap<String, VandInstrument>,
    requested: &str,
    track: &str,
    fallback: &str,
) -> String {
    if !requested.is_empty()
        && instruments
            .get(requested)
            .is_some_and(|instrument| instrument.fm.is_some())
    {
        return requested.to_string();
    }
    if instruments
        .get(fallback)
        .is_some_and(|instrument| instrument.fm.is_some())
    {
        return fallback.to_string();
    }

    let mut best: Option<(&str, i32)> = None;
    for (id, instrument) in instruments {
        if instrument.chip != "ym2612" || instrument.fm.is_none() {
            continue;
        }
        let role = if instrument.role.is_empty() {
            instrument.category.as_str()
        } else {
            instrument.role.as_str()
        };
        let mut score = 0;
        if role_matches(track, role) {
            score += 80;
        }
        if role_matches(track, &instrument.category) {
            score += 40;
        }
        for tag in &instrument.tags {
            if role_matches(track, tag) {
                score += 28;
            }
            if track == "bass" && matches!(tag.as_str(), "low" | "aggressive") {
                score += 12;
            }
            if matches!(track, "lead" | "hook" | "counter")
                && matches!(tag.as_str(), "melodic" | "bright" | "short")
            {
                score += 12;
            }
            if matches!(track, "pad" | "chord") && matches!(tag.as_str(), "sustained" | "chord") {
                score += 12;
            }
        }
        let name = instrument.name.to_ascii_lowercase();
        if track == "bass" && (name.contains("bass") || name.contains("sub")) {
            score += 14;
        }
        if matches!(track, "lead" | "hook") && (name.contains("lead") || name.contains("pluck")) {
            score += 14;
        }
        if matches!(track, "pad" | "chord") && (name.contains("pad") || name.contains("string")) {
            score += 14;
        }
        if score == 0 {
            score = 1;
        }
        if best.is_none_or(|(_, best_score)| score > best_score) {
            best = Some((id.as_str(), score));
        }
    }

    best.map(|(id, _)| id.to_string())
        .unwrap_or_else(|| fallback.to_string())
}

fn write_fm_patch(audio: &mut Audio, instrument: &VandInstrument, channel: usize, attenuation: u8) {
    const OP_OFFSETS: [u8; 4] = [0x00, 0x04, 0x08, 0x0c];
    let Some(fm) = instrument.fm.as_ref() else {
        return;
    };
    let part = if channel >= 3 { 1 } else { 0 };
    let slot = (channel % 3) as u8;

    audio.ym2612.write_register(
        part,
        0xb0 + slot,
        (fm.algorithm & 0x07) | ((fm.feedback & 0x07) << 3),
        None,
        true,
    );
    audio
        .ym2612
        .write_register(part, 0xb4 + slot, 0xc0, None, true);
    for (index, op) in fm.operators.iter().take(4).enumerate() {
        let reg = OP_OFFSETS[index] + slot;
        audio.ym2612.write_register(
            part,
            0x30 + reg,
            ((op.dt & 0x07) << 4) | (op.mult & 0x0f),
            None,
            true,
        );
        audio.ym2612.write_register(
            part,
            0x40 + reg,
            op.tl.saturating_add(attenuation).min(127),
            None,
            true,
        );
        audio
            .ym2612
            .write_register(part, 0x50 + reg, op.ar.min(31), None, true);
        audio
            .ym2612
            .write_register(part, 0x60 + reg, op.dr.min(31), None, true);
        audio
            .ym2612
            .write_register(part, 0x70 + reg, 0x04, None, true);
        audio.ym2612.write_register(
            part,
            0x80 + reg,
            ((op.sl.min(15)) << 4) | op.rr.min(15),
            None,
            true,
        );
        audio
            .ym2612
            .write_register(part, 0x90 + reg, 0x00, None, true);
    }
}

fn write_fm_pitch(audio: &mut Audio, channel: usize, hz: f32) {
    let (fnum, block) = ym2612_pitch_for_hz(hz);
    let part = if channel >= 3 { 1 } else { 0 };
    let slot = (channel % 3) as u8;
    audio.ym2612.write_register(
        part,
        0xa4 + slot,
        ((block & 0x07) << 3) | ((fnum >> 8) as u8 & 0x07),
        None,
        true,
    );
    audio
        .ym2612
        .write_register(part, 0xa0 + slot, (fnum & 0xff) as u8, None, true);
}

fn fm_key_on(audio: &mut Audio, channel: usize, on: bool) {
    let key_channel = if channel >= 3 { channel + 1 } else { channel };
    let value = if on {
        0xf0 | key_channel as u8
    } else {
        key_channel as u8
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

fn load_instrument_bank_for_render_plan(
    repo: &Path,
    plan_path: &Path,
    plan: &VandRenderPlan,
    explicit_bank_path: &str,
) -> Result<HashMap<String, VandInstrument>, String> {
    let plan_dir = plan_path.parent().unwrap_or_else(|| Path::new("."));
    load_instrument_bank_from_reference(repo, plan_dir, &plan.instrument_bank, explicit_bank_path)
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

fn render_plan_to_note_arrangement(plan: &VandRenderPlan) -> VandNoteArrangement {
    let mut notes = Vec::new();
    for track in &plan.tracks {
        for event in &track.events {
            notes.push(VandNoteEvent {
                track: track.role.clone(),
                start_frame: event.start_frame,
                frames: event.frames,
                hz: event.hz,
                velocity: event.velocity,
                instrument_id: event.instrument_id.clone(),
            });
        }
    }
    let drums = plan
        .drums
        .iter()
        .map(|drum| VandDrumEvent {
            start_frame: drum.start_frame,
            velocity: drum.velocity,
            noise_amp: drum.psg_noise_amp,
            instrument_id: if drum.psg_instrument_id.is_empty() {
                DEFAULT_NOISE_INSTRUMENT.to_string()
            } else {
                drum.psg_instrument_id.clone()
            },
            dac_chunk: drum.dac_chunk,
            dac_path: drum.dac_path.clone(),
        })
        .collect();
    VandNoteArrangement {
        sample_rate: plan.sample_rate,
        hop: plan.hop,
        total_frames: plan.total_frames,
        instrument_bank: plan.instrument_bank.clone(),
        notes,
        drums,
    }
}

fn ai_notes_to_note_arrangement(ai: &VandAiNotes, instrument_bank: String) -> VandNoteArrangement {
    let frames_per_second = ai.sample_rate as f32 / ai.hop.max(1) as f32;
    let notes = select_ai_role_notes(ai)
        .into_iter()
        .map(|(track, event)| ai_event_to_note_event(track, event, frames_per_second))
        .collect::<Vec<_>>();
    let inferred_frames = notes
        .iter()
        .map(|note| note.start_frame.saturating_add(note.frames))
        .max()
        .unwrap_or(0);
    let total_frames = ((ai.total_seconds.max(0.0) * frames_per_second).round() as usize)
        .max(inferred_frames)
        .max(1);
    VandNoteArrangement {
        sample_rate: ai.sample_rate,
        hop: ai.hop,
        total_frames,
        instrument_bank,
        notes,
        drums: synthesize_ai_drum_events(ai, total_frames, frames_per_second),
    }
}

fn make_psg_drum(start_frame: usize, velocity: f32, noise_amp: f32) -> VandDrumEvent {
    VandDrumEvent {
        start_frame,
        velocity: velocity.clamp(0.0, 1.0),
        noise_amp: noise_amp.clamp(0.0, 1.0),
        instrument_id: DEFAULT_NOISE_INSTRUMENT.to_string(),
        dac_chunk: None,
        dac_path: String::new(),
    }
}

fn push_or_merge_drum(drums: &mut Vec<VandDrumEvent>, drum: VandDrumEvent, merge_window: usize) {
    if let Some(existing) = drums
        .iter_mut()
        .find(|existing| existing.start_frame.abs_diff(drum.start_frame) <= merge_window)
    {
        existing.velocity = existing.velocity.max(drum.velocity);
        existing.noise_amp = existing.noise_amp.max(drum.noise_amp);
        return;
    }
    drums.push(drum);
}

fn synthesize_ai_drum_events(
    ai: &VandAiNotes,
    total_frames: usize,
    frames_per_second: f32,
) -> Vec<VandDrumEvent> {
    if total_frames == 0 || frames_per_second <= 0.0 {
        return Vec::new();
    }
    let step = (frames_per_second * 0.125).round().max(1.0) as usize;
    let beat = (frames_per_second * 0.50).round().max(step as f32) as usize;
    let merge_window = (frames_per_second * 0.045).round().max(1.0) as usize;
    let mut drums = Vec::new();

    for frame in (0..=total_frames).step_by(step) {
        let beat_pos = if beat > 0 { frame % beat } else { 0 };
        let velocity = if beat_pos <= merge_window { 0.36 } else { 0.20 };
        push_or_merge_drum(
            &mut drums,
            make_psg_drum(frame.min(total_frames), velocity, velocity * 0.72),
            merge_window,
        );
    }

    let two_beats = beat.saturating_mul(2).max(1);
    let four_beats = beat.saturating_mul(4).max(two_beats);
    for frame in (beat..=total_frames).step_by(two_beats) {
        let backbeat = if frame % four_beats == beat {
            0.74
        } else {
            0.64
        };
        push_or_merge_drum(
            &mut drums,
            make_psg_drum(frame, backbeat, backbeat * 0.88),
            merge_window,
        );
    }

    for event in flatten_ai_events(ai) {
        if event.midi <= 55 && event.velocity >= 0.30 {
            let frame = (event.start_time.max(0.0) * frames_per_second).round() as usize;
            push_or_merge_drum(
                &mut drums,
                make_psg_drum(frame.min(total_frames), 0.68 + event.velocity * 0.22, 0.32),
                merge_window,
            );
        } else if event.velocity >= 0.62 && event.duration <= 0.18 {
            let frame = (event.start_time.max(0.0) * frames_per_second).round() as usize;
            push_or_merge_drum(
                &mut drums,
                make_psg_drum(frame.min(total_frames), 0.45 + event.velocity * 0.22, 0.50),
                merge_window,
            );
        }
    }

    drums.sort_by_key(|drum| drum.start_frame);
    drums
}

fn add_dense_psg_drums(arrangement: &mut VandNoteArrangement) {
    let frames_per_second = arrangement.sample_rate as f32 / arrangement.hop.max(1) as f32;
    if arrangement.total_frames == 0 || frames_per_second <= 0.0 {
        return;
    }
    let step = (frames_per_second * 0.125).round().max(1.0) as usize;
    let beat = (frames_per_second * 0.50).round().max(step as f32) as usize;
    let merge_window = (frames_per_second * 0.040).round().max(1.0) as usize;
    for frame in (0..=arrangement.total_frames).step_by(step) {
        let velocity = if frame % beat <= merge_window {
            0.34
        } else {
            0.18
        };
        push_or_merge_drum(
            &mut arrangement.drums,
            make_psg_drum(frame, velocity, velocity * 0.72),
            merge_window,
        );
    }
    for frame in (beat..=arrangement.total_frames).step_by(beat.saturating_mul(2).max(1)) {
        push_or_merge_drum(
            &mut arrangement.drums,
            make_psg_drum(frame, 0.72, 0.66),
            merge_window,
        );
    }
    arrangement.drums.sort_by_key(|drum| drum.start_frame);
}

fn ai_event_to_note_event(
    track: &'static str,
    event: VandAiNoteEvent,
    frames_per_second: f32,
) -> VandNoteEvent {
    let hz = if event.hz > 0.0 {
        event.hz
    } else {
        midi_to_hz(event.midi)
    };
    let instrument_id = match track {
        "bass" => DEFAULT_BASS_INSTRUMENT,
        "pad" | "chord" => DEFAULT_PAD_INSTRUMENT,
        _ => DEFAULT_LEAD_INSTRUMENT,
    };
    let velocity = match track {
        "pad" => event.velocity * 0.62,
        "chord" => event.velocity * 0.52,
        "counter" => event.velocity * 0.70,
        _ => event.velocity,
    };
    VandNoteEvent {
        track: track.to_string(),
        start_frame: (event.start_time.max(0.0) * frames_per_second).round() as usize,
        frames: (event.duration.max(0.01) * frames_per_second)
            .round()
            .max(1.0) as usize,
        hz,
        velocity: velocity.clamp(0.0, 1.0),
        instrument_id: instrument_id.to_string(),
    }
}

fn ai_lead_score(event: &VandAiNoteEvent) -> f32 {
    let register = if event.midi < 48 {
        -0.45
    } else if event.midi < 60 {
        -0.12
    } else if event.midi <= 84 {
        0.22
    } else {
        0.04
    };
    let duration_bonus = event.duration.clamp(0.04, 0.60) * 0.22;
    event.velocity.clamp(0.0, 1.0) + register + duration_bonus
}

fn flatten_ai_events(ai: &VandAiNotes) -> Vec<VandAiNoteEvent> {
    let mut events = ai
        .tracks
        .iter()
        .flat_map(|track| {
            let _track_role = track.role.as_str();
            track.events.iter()
        })
        .filter(|event| event.duration >= 0.045 && event.velocity >= 0.24)
        .cloned()
        .collect::<Vec<_>>();
    events.sort_by(|a, b| {
        a.start_time
            .partial_cmp(&b.start_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    events
}

fn select_best_in_windows<F>(
    events: &[VandAiNoteEvent],
    window_seconds: f32,
    score: F,
) -> Vec<VandAiNoteEvent>
where
    F: Fn(&VandAiNoteEvent) -> f32,
{
    let mut out: Vec<VandAiNoteEvent> = Vec::new();
    let mut index = 0usize;
    while index < events.len() {
        let window_start = events[index].start_time;
        let window_end = window_start + window_seconds;
        let mut best = events[index].clone();
        index += 1;
        while index < events.len() && events[index].start_time <= window_end {
            if score(&events[index]) > score(&best) {
                best = events[index].clone();
            }
            index += 1;
        }
        if let Some(last) = out.last_mut() {
            let last_end = last.start_time + last.duration;
            if best.start_time < last_end + 0.035 {
                if best.midi == last.midi {
                    let new_end = (best.start_time + best.duration).max(last_end);
                    last.duration = new_end - last.start_time;
                    last.velocity = last.velocity.max(best.velocity);
                } else if score(&best) > score(last) + 0.10 {
                    out.push(best);
                }
                continue;
            }
        }
        out.push(best);
    }

    out
}

fn select_ai_lead_notes(ai: &VandAiNotes) -> Vec<VandAiNoteEvent> {
    let events = flatten_ai_events(ai)
        .into_iter()
        .filter(|event| event.midi >= 45)
        .collect::<Vec<_>>();
    select_best_in_windows(&events, 0.085, ai_lead_score)
}

fn select_ai_bass_notes(ai: &VandAiNotes) -> Vec<VandAiNoteEvent> {
    let events = flatten_ai_events(ai)
        .into_iter()
        .filter(|event| event.midi >= 28 && event.midi <= 55 && event.velocity >= 0.30)
        .collect::<Vec<_>>();
    select_best_in_windows(&events, 0.14, |event| {
        event.velocity + event.duration.clamp(0.05, 0.80) * 0.24
            - (event.midi as f32 - 36.0).abs() * 0.006
    })
}

fn select_ai_counter_notes(
    ai: &VandAiNotes,
    lead_notes: &[VandAiNoteEvent],
) -> Vec<VandAiNoteEvent> {
    let events = flatten_ai_events(ai)
        .into_iter()
        .filter(|event| event.midi >= 72 && event.midi <= 96 && event.velocity >= 0.28)
        .filter(|event| {
            !lead_notes.iter().any(|lead| {
                let overlap_start = event.start_time.max(lead.start_time);
                let overlap_end =
                    (event.start_time + event.duration).min(lead.start_time + lead.duration);
                overlap_end > overlap_start && (event.midi - lead.midi).abs() < 7
            })
        })
        .collect::<Vec<_>>();
    select_best_in_windows(&events, 0.16, |event| {
        event.velocity
            + event.duration.clamp(0.04, 0.45) * 0.14
            + (event.midi as f32 - 72.0) * 0.004
    })
}

fn overlaps_ai_note(event: &VandAiNoteEvent, other: &VandAiNoteEvent) -> bool {
    let overlap_start = event.start_time.max(other.start_time);
    let overlap_end = (event.start_time + event.duration).min(other.start_time + other.duration);
    overlap_end > overlap_start
}

fn select_ai_harmony_notes(
    ai: &VandAiNotes,
    lead_notes: &[VandAiNoteEvent],
) -> Vec<(&'static str, VandAiNoteEvent)> {
    let events = flatten_ai_events(ai)
        .into_iter()
        .filter(|event| {
            event.midi >= 48
                && event.midi <= 78
                && event.velocity >= 0.26
                && event.duration >= 0.10
                && !lead_notes
                    .iter()
                    .any(|lead| overlaps_ai_note(event, lead) && (event.midi - lead.midi).abs() < 3)
        })
        .collect::<Vec<_>>();
    let mut out = Vec::new();
    let mut index = 0usize;
    while index < events.len() {
        let window_start = events[index].start_time;
        let window_end = window_start + 0.30;
        let mut window = Vec::new();
        while index < events.len() && events[index].start_time <= window_end {
            window.push(events[index].clone());
            index += 1;
        }
        window.sort_by(|a, b| {
            let score_a = a.velocity + a.duration.clamp(0.10, 0.85) * 0.34;
            let score_b = b.velocity + b.duration.clamp(0.10, 0.85) * 0.34;
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut selected = Vec::new();
        for event in window {
            if selected.iter().any(|other: &VandAiNoteEvent| {
                (event.midi - other.midi).abs() < 4 || (event.midi - other.midi).abs() > 19
            }) {
                continue;
            }
            let mut event = event;
            event.start_time = window_start;
            event.duration = event.duration.clamp(0.18, 0.70);
            selected.push(event);
            if selected.len() == 2 {
                break;
            }
        }
        selected.sort_by_key(|event| event.midi);
        if selected.len() == 2 {
            out.push(("pad", selected[0].clone()));
            out.push(("chord", selected[1].clone()));
        } else if selected.len() == 1 && selected[0].duration >= 0.22 {
            out.push(("pad", selected[0].clone()));
        }
    }
    out
}

fn select_ai_role_notes(ai: &VandAiNotes) -> Vec<(&'static str, VandAiNoteEvent)> {
    let lead = select_ai_lead_notes(ai);
    let bass = select_ai_bass_notes(ai);
    let counter = select_ai_counter_notes(ai, &lead);
    let harmony = select_ai_harmony_notes(ai, &lead);
    let mut out = Vec::with_capacity(lead.len() + bass.len() + counter.len() + harmony.len());
    out.extend(bass.into_iter().map(|event| ("bass", event)));
    out.extend(harmony);
    out.extend(lead.into_iter().map(|event| ("lead", event)));
    out.extend(counter.into_iter().map(|event| ("counter", event)));
    out.sort_by(|a, b| {
        a.1.start_time
            .partial_cmp(&b.1.start_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

fn merge_hybrid_note_arrangement(
    mut base: VandNoteArrangement,
    ai: &VandAiNotes,
    instrument_bank: String,
) -> VandNoteArrangement {
    let mut ai_arrangement = ai_notes_to_note_arrangement(ai, instrument_bank.clone());
    ai_arrangement.notes.retain(|note| note.track != "bass");
    base.instrument_bank = instrument_bank;
    base.notes.retain(|note| {
        !matches!(
            note.track.as_str(),
            "lead" | "hook" | "counter" | "psg_lead" | "psg_counter"
        )
    });
    base.notes.extend(ai_arrangement.notes);
    base.drums.extend(ai_arrangement.drums);
    add_dense_psg_drums(&mut base);
    base.total_frames = base.total_frames.max(ai_arrangement.total_frames);
    base
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
            let selected_bass_id =
                choose_fm_instrument_id(instruments, next_bass_id, "bass", DEFAULT_BASS_INSTRUMENT);
            if bass_id != selected_bass_id {
                bass_id = selected_bass_id;
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
            let selected_lead_id =
                choose_fm_instrument_id(instruments, next_lead_id, "lead", DEFAULT_LEAD_INSTRUMENT);
            if lead_id != selected_lead_id {
                lead_id = selected_lead_id;
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
    mix: RenderMix,
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
    let mut pad_id = String::new();
    let mut chord_id = String::new();
    let mut counter_id = String::new();
    let mut hook_id = String::new();
    let mut bass_end = 0usize;
    let mut lead_end = 0usize;
    let mut pad_end = 0usize;
    let mut chord_end = 0usize;
    let mut counter_end = 0usize;
    let mut hook_end = 0usize;
    let mut psg_end = [0usize; 3];
    let mut bass_on = false;
    let mut lead_on = false;
    let mut pad_on = false;
    let mut chord_on = false;
    let mut counter_on = false;
    let mut hook_on = false;
    let mut psg_on = [false; 3];
    let mut noise_until = 0usize;
    let mut noise_amp = 0.0f32;

    for frame in 0..total_frames {
        audio.begin_frame();

        while note_index < notes.len() && notes[note_index].start_frame == frame {
            let note = &notes[note_index];
            if let Some(channel) = match note.track.as_str() {
                "psg_lead" => Some(0usize),
                "psg_counter" => Some(1usize),
                "psg_bass" => Some(2usize),
                _ => None,
            } {
                let effective_velocity = (note.velocity * mix.psg_gain).clamp(0.0, 1.0);
                psg_set_tone(
                    &mut audio,
                    channel as u8,
                    note.hz as f64,
                    amp_to_psg_volume(effective_velocity),
                );
                psg_on[channel] = true;
                psg_end[channel] = note.start_frame.saturating_add(note.frames);
                note_index += 1;
                continue;
            }

            let (channel, current_id, active, end_frame, fallback) = if note.track == "bass" {
                (
                    0usize,
                    &mut bass_id,
                    &mut bass_on,
                    &mut bass_end,
                    DEFAULT_BASS_INSTRUMENT,
                )
            } else if note.track == "pad" {
                (
                    2usize,
                    &mut pad_id,
                    &mut pad_on,
                    &mut pad_end,
                    DEFAULT_PAD_INSTRUMENT,
                )
            } else if note.track == "chord" {
                (
                    3usize,
                    &mut chord_id,
                    &mut chord_on,
                    &mut chord_end,
                    DEFAULT_PAD_INSTRUMENT,
                )
            } else if note.track == "counter" {
                (
                    4usize,
                    &mut counter_id,
                    &mut counter_on,
                    &mut counter_end,
                    DEFAULT_LEAD_INSTRUMENT,
                )
            } else if note.track == "hook" {
                (
                    5usize,
                    &mut hook_id,
                    &mut hook_on,
                    &mut hook_end,
                    DEFAULT_LEAD_INSTRUMENT,
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
            let selected_id = choose_fm_instrument_id(instruments, next_id, &note.track, fallback);
            if current_id.as_str() != selected_id {
                *current_id = selected_id;
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
                let role_gain = if note.track == "bass" {
                    mix.bass_gain
                } else if note.track == "pad" {
                    mix.lead_gain * 0.72
                } else if note.track == "chord" {
                    mix.lead_gain * 0.58
                } else if note.track == "counter" {
                    mix.lead_gain * 0.52
                } else if note.track == "hook" {
                    mix.lead_gain * 0.78
                } else {
                    mix.lead_gain
                };
                let effective_velocity = (note.velocity * role_gain).clamp(0.0, 1.0);
                let attenuation = ((1.0 - effective_velocity) * 28.0) as u8;
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
        if pad_on && frame >= pad_end {
            fm_key_on(&mut audio, 2, false);
            pad_on = false;
        }
        if chord_on && frame >= chord_end {
            fm_key_on(&mut audio, 3, false);
            chord_on = false;
        }
        if counter_on && frame >= counter_end {
            fm_key_on(&mut audio, 4, false);
            counter_on = false;
        }
        if hook_on && frame >= hook_end {
            fm_key_on(&mut audio, 5, false);
            hook_on = false;
        }
        for channel in 0..3 {
            if psg_on[channel] && frame >= psg_end[channel] {
                psg_latch_volume(&mut audio, channel as u8, 15);
                psg_on[channel] = false;
            }
        }

        while drum_index < drums.len() && drums[drum_index].start_frame == frame {
            let drum = &drums[drum_index];
            let id = if drum.instrument_id.is_empty() {
                DEFAULT_NOISE_INSTRUMENT
            } else {
                &drum.instrument_id
            };
            if instruments.contains_key(id) {
                let noise_scale = if drum.dac_chunk.is_some() || !drum.dac_path.is_empty() {
                    0.40
                } else {
                    0.14
                };
                noise_amp = (drum.noise_amp.max(drum.velocity * 0.65) * mix.psg_gain * noise_scale)
                    .clamp(0.0, 1.0);
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

    mix_dac_drums(
        &mut out,
        &drums,
        arrangement_dir,
        frame_samples,
        mix.dac_gain,
    );
    out
}

fn mix_dac_drums(
    out: &mut [i16],
    drums: &[VandDrumEvent],
    arrangement_dir: &Path,
    frame_samples: usize,
    dac_gain: f32,
) {
    const PREVIEW_RATE: usize = 44_100;
    const DAC_RATE: usize = 13_320;
    let mut cache: HashMap<String, Vec<u8>> = HashMap::new();

    for drum in drums {
        let start = drum.start_frame.saturating_mul(frame_samples);
        if start >= out.len() {
            continue;
        }
        let gain =
            drum.velocity.max(drum.noise_amp).clamp(0.0, 1.0) * dac_gain.clamp(0.0, 2.0) * 0.55;
        if drum.dac_chunk.is_none() || drum.dac_path.is_empty() {
            mix_synthetic_preview_drum(out, start, drum, gain);
            continue;
        }
        let bytes = if let Some(bytes) = cache.get(&drum.dac_path) {
            bytes
        } else {
            let path = resolve_existing_path(Path::new(""), arrangement_dir, &drum.dac_path);
            let Ok(bytes) = fs::read(path) else {
                mix_synthetic_preview_drum(out, start, drum, gain);
                continue;
            };
            cache.insert(drum.dac_path.clone(), bytes);
            cache.get(&drum.dac_path).unwrap()
        };
        if bytes.is_empty() {
            mix_synthetic_preview_drum(out, start, drum, gain);
            continue;
        }

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

fn mix_synthetic_preview_drum(out: &mut [i16], start: usize, drum: &VandDrumEvent, gain: f32) {
    const PREVIEW_RATE: usize = 44_100;
    if start >= out.len() || gain <= 0.0 {
        return;
    }

    let is_kick = drum.velocity >= 0.62 && drum.noise_amp < drum.velocity * 0.62;
    let is_snare = drum.velocity >= 0.58 && drum.noise_amp >= drum.velocity * 0.58;
    let len = if is_kick {
        PREVIEW_RATE * 15 / 100
    } else if is_snare {
        PREVIEW_RATE * 10 / 100
    } else {
        PREVIEW_RATE * 4 / 100
    };
    let mut noise_state = 0x9E37_79B9u32 ^ drum.start_frame as u32;
    let amp = drum.velocity.max(drum.noise_amp).clamp(0.0, 1.0) * gain.clamp(0.0, 2.0);

    for i in 0..len {
        let out_index = start + i;
        if out_index >= out.len() {
            break;
        }
        let t = i as f32 / PREVIEW_RATE as f32;
        let progress = i as f32 / len.max(1) as f32;
        let env = (1.0 - progress)
            .max(0.0)
            .powf(if is_kick { 2.6 } else { 3.8 });
        noise_state = noise_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let noise = ((noise_state >> 16) as f32 / 32768.0) - 1.0;
        let sample = if is_kick {
            let sweep_hz = 112.0 - progress * 70.0;
            (std::f32::consts::TAU * sweep_hz * t).sin() * env * 1.15 + noise * env * 0.10
        } else if is_snare {
            let body = (std::f32::consts::TAU * 185.0 * t).sin() * env * 0.34;
            body + noise * env * 0.78
        } else {
            noise * env * 0.36
        };
        let sample = (sample * amp * 32767.0).round() as i32;
        let mixed = i32::from(out[out_index]) + sample;
        out[out_index] = mixed.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
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

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn ai_cache_env(command: &mut Command, ai_home: &Path) {
    command
        .env("XDG_CACHE_HOME", ai_home.join("cache"))
        .env("HF_HOME", ai_home.join("huggingface"))
        .env("TORCH_HOME", ai_home.join("torch"));
}

fn find_ai_tool(command: &str, ai_home: &Path, ai_tool_dir: &Path) -> Option<PathBuf> {
    let requested = Path::new(command);
    if requested.components().count() > 1 && requested.is_file() {
        return Some(requested.to_path_buf());
    }

    if !ai_tool_dir.as_os_str().is_empty() {
        let candidate = ai_tool_dir.join(command);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    for dir in [
        ai_home.join("vand-ai-lism/bin"),
        ai_home.join(".venv/bin"),
        ai_home.join("bin"),
    ] {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    if let Some(paths) = env::var_os("PATH") {
        for dir in env::split_paths(&paths) {
            let candidate = dir.join(command);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

fn push_command_log(steps: &mut Vec<String>, label: &str, output: &std::process::Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = stdout.trim();
    let stderr = stderr.trim();
    if !stdout.is_empty() {
        steps.push(format!("{label} stdout: {stdout}"));
    }
    if !stderr.is_empty() {
        steps.push(format!("{label} stderr: {stderr}"));
    }
}

fn write_ai_manifest(
    bundle_dir: &Path,
    input: &Path,
    ai_backend: &str,
    ai_home: &Path,
    status: &str,
    steps: &[String],
    artifacts: &[PathBuf],
) -> Result<PathBuf, String> {
    let ai_dir = bundle_dir.join("ai");
    fs::create_dir_all(&ai_dir)
        .map_err(|err| format!("failed to create {}: {err}", ai_dir.display()))?;
    let manifest = ai_dir.join("manifest.json");
    let generated_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let mut text = String::new();
    text.push_str("{\n");
    text.push_str("  \"format\": \"vand-ai-lism-ai-manifest-v0\",\n");
    text.push_str(&format!("  \"generated_at_unix\": {},\n", generated_at));
    text.push_str(&format!(
        "  \"backend\": \"{}\",\n",
        json_escape(ai_backend)
    ));
    text.push_str(&format!("  \"status\": \"{}\",\n", json_escape(status)));
    text.push_str(&format!(
        "  \"source\": \"{}\",\n",
        json_escape(&input.display().to_string())
    ));
    text.push_str(&format!(
        "  \"ai_home\": \"{}\",\n",
        json_escape(&ai_home.display().to_string())
    ));
    text.push_str("  \"steps\": [\n");
    for (index, step) in steps.iter().enumerate() {
        let comma = if index + 1 == steps.len() { "" } else { "," };
        text.push_str(&format!("    \"{}\"{}\n", json_escape(step), comma));
    }
    text.push_str("  ],\n");
    text.push_str("  \"artifacts\": [\n");
    for (index, artifact) in artifacts.iter().enumerate() {
        let comma = if index + 1 == artifacts.len() {
            ""
        } else {
            ","
        };
        text.push_str(&format!(
            "    \"{}\"{}\n",
            json_escape(&artifact.display().to_string()),
            comma
        ));
    }
    text.push_str("  ]\n");
    text.push_str("}\n");
    fs::write(&manifest, text)
        .map_err(|err| format!("failed to write {}: {err}", manifest.display()))?;
    Ok(manifest)
}

fn find_first_file_with_suffix(dir: &Path, suffix: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(suffix))
        {
            return Some(path);
        }
    }
    None
}

fn run_optional_ai_backend(
    input: &Path,
    bundle_dir: &Path,
    ai_backend: &str,
    ai_home: &Path,
    ai_tool_dir: &Path,
) -> Result<(String, Option<PathBuf>), String> {
    let backend = ai_backend.trim();
    if backend.is_empty() || backend == "off" {
        return Ok(("AI backend off.".to_string(), None));
    }

    fs::create_dir_all(ai_home)
        .map_err(|err| format!("failed to create AI home {}: {err}", ai_home.display()))?;
    let ai_dir = bundle_dir.join("ai");
    fs::create_dir_all(&ai_dir)
        .map_err(|err| format!("failed to create {}: {err}", ai_dir.display()))?;

    let mut steps = vec![format!("selected backend: {backend}")];
    let mut artifacts = Vec::new();
    let mut ai_notes = None;
    let mut status = "ready";

    let basic_pitch = if matches!(backend, "basic-pitch" | "demucs-basic-pitch") {
        find_ai_tool("basic-pitch", ai_home, ai_tool_dir)
    } else {
        None
    };
    let demucs = if backend == "demucs-basic-pitch" {
        find_ai_tool("demucs", ai_home, ai_tool_dir)
    } else {
        None
    };
    if matches!(backend, "basic-pitch" | "demucs-basic-pitch") && basic_pitch.is_none() {
        status = "missing_ai_tools";
        steps.push(format!(
            "basic-pitch command not found; looked in {}, {}/vand-ai-lism/bin, {}/.venv/bin, {}/bin and PATH",
            if ai_tool_dir.as_os_str().is_empty() {
                "(no ai_tool_dir configured)".to_string()
            } else {
                ai_tool_dir.display().to_string()
            },
            ai_home.display(),
            ai_home.display(),
            ai_home.display()
        ));
    }
    if backend == "demucs-basic-pitch" && demucs.is_none() {
        status = "missing_ai_tools";
        steps.push(format!(
            "demucs command not found; looked in {}, {}/vand-ai-lism/bin, {}/.venv/bin, {}/bin and PATH",
            if ai_tool_dir.as_os_str().is_empty() {
                "(no ai_tool_dir configured)".to_string()
            } else {
                ai_tool_dir.display().to_string()
            },
            ai_home.display(),
            ai_home.display(),
            ai_home.display()
        ));
    }

    if status == "ready" {
        if backend == "demucs-basic-pitch" {
            let stems_dir = ai_dir.join("stems");
            fs::create_dir_all(&stems_dir)
                .map_err(|err| format!("failed to create {}: {err}", stems_dir.display()))?;
            let mut command = Command::new(demucs.expect("demucs resolved"));
            command.arg("-o").arg(&stems_dir).arg(input);
            ai_cache_env(&mut command, ai_home);
            let output = command
                .output()
                .map_err(|err| format!("failed to start demucs: {err}"))?;
            push_command_log(&mut steps, "demucs", &output);
            if !output.status.success() {
                status = "demucs_failed";
            }
            artifacts.push(stems_dir);
        }

        if status == "ready" {
            let midi_dir = ai_dir.join("basic_pitch");
            fs::create_dir_all(&midi_dir)
                .map_err(|err| format!("failed to create {}: {err}", midi_dir.display()))?;
            let mut command = Command::new(basic_pitch.expect("basic-pitch resolved"));
            command
                .arg("--save-midi")
                .arg("--save-note-events")
                .arg(&midi_dir)
                .arg(input);
            ai_cache_env(&mut command, ai_home);
            let output = command
                .output()
                .map_err(|err| format!("failed to start basic-pitch: {err}"))?;
            push_command_log(&mut steps, "basic-pitch", &output);
            if output.status.success() {
                ai_notes = find_first_file_with_suffix(&midi_dir, "_basic_pitch.csv");
                artifacts.push(midi_dir);
                status = "complete";
            } else {
                status = "basic_pitch_failed";
            }
        }
    }

    if let Some(path) = &ai_notes {
        artifacts.push(path.clone());
    }
    let manifest = write_ai_manifest(
        bundle_dir, input, backend, ai_home, status, &steps, &artifacts,
    )?;
    let ai_notes_json = if status == "complete" {
        if let Some(csv_path) = &ai_notes {
            let out_path = basic_pitch_notes_path(bundle_dir);
            parse_basic_pitch_csv(csv_path, &out_path)?;
            Some(out_path)
        } else {
            None
        }
    } else {
        None
    };
    Ok(format!(
        "AI backend {backend}: {status}\nmanifest {}",
        manifest.display()
    ))
    .map(|log| (log, ai_notes_json))
}

fn basic_pitch_notes_path(bundle_dir: &Path) -> PathBuf {
    bundle_dir
        .join("ai")
        .join("basic_pitch")
        .join("ai_notes.vand-ai-notes.json")
}

fn midi_to_hz(midi: i32) -> f32 {
    440.0 * 2.0f32.powf((midi as f32 - 69.0) / 12.0)
}

fn parse_basic_pitch_csv(csv_path: &Path, out_path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(csv_path).map_err(|err| {
        format!(
            "failed to read Basic Pitch CSV {}: {err}",
            csv_path.display()
        )
    })?;
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }

    let mut notes = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        if line_index == 0 || line.trim().is_empty() {
            continue;
        }
        let mut columns = line.split(',');
        let Some(start) = columns.next().and_then(|value| value.parse::<f32>().ok()) else {
            continue;
        };
        let Some(end) = columns.next().and_then(|value| value.parse::<f32>().ok()) else {
            continue;
        };
        let Some(midi) = columns.next().and_then(|value| value.parse::<i32>().ok()) else {
            continue;
        };
        let Some(velocity) = columns.next().and_then(|value| value.parse::<f32>().ok()) else {
            continue;
        };
        if end <= start {
            continue;
        }
        notes.push((start, end, midi, velocity));
    }
    notes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let total_seconds = notes
        .iter()
        .map(|(_, end, _, _)| *end)
        .fold(0.0f32, f32::max);
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"format\": \"vand-ai-lism-ai-notes-v0\",\n");
    out.push_str("  \"source\": \"basic-pitch\",\n");
    out.push_str(&format!(
        "  \"csv\": \"{}\",\n",
        json_escape(&csv_path.display().to_string())
    ));
    out.push_str(&format!("  \"total_seconds\": {:.6},\n", total_seconds));
    out.push_str("  \"tracks\": [\n");
    out.push_str(
        "    {\"role\": \"ai_basic_pitch\", \"chip\": \"ai\", \"channel\": 0, \"events\": [\n",
    );
    for (index, (start, end, midi, velocity)) in notes.iter().enumerate() {
        let comma = if index + 1 == notes.len() { "" } else { "," };
        out.push_str(&format!(
            "      {{\"start_time\": {:.6}, \"end_time\": {:.6}, \"duration\": {:.6}, \"midi\": {}, \"hz\": {:.3}, \"velocity\": {:.5}}}{}\n",
            start,
            end,
            end - start,
            midi,
            midi_to_hz(*midi),
            (velocity / 127.0).clamp(0.0, 1.0),
            comma
        ));
    }
    out.push_str("    ]}\n");
    out.push_str("  ]\n");
    out.push_str("}\n");
    fs::write(out_path, out)
        .map_err(|err| format!("failed to write AI notes {}: {err}", out_path.display()))
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

fn trim_dialog_path(bytes: &[u8]) -> Option<String> {
    let path = String::from_utf8_lossy(bytes).trim().to_string();
    if path.is_empty() { None } else { Some(path) }
}

fn run_dialog_command(command: &mut Command) -> Result<Option<String>, String> {
    match command.output() {
        Ok(output) if output.status.success() => Ok(trim_dialog_path(&output.stdout)),
        Ok(output) if output.status.code() == Some(1) => Ok(None),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.is_empty() {
                Ok(None)
            } else {
                Err(stderr)
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.to_string()),
    }
}

fn pick_file_native(title: &str, filters: &[&str]) -> Result<Option<String>, String> {
    let mut zenity = Command::new("zenity");
    zenity
        .arg("--file-selection")
        .arg("--title")
        .arg(title)
        .arg("--filename")
        .arg("audio/source/");
    if !filters.is_empty() {
        zenity.arg(format!("--file-filter=Audio files | {}", filters.join(" ")));
    }
    zenity.arg("--file-filter=All files | *");
    match run_dialog_command(&mut zenity) {
        Ok(Some(path)) => return Ok(Some(path)),
        Ok(None) => {}
        Err(err) => eprintln!("zenity file picker failed: {err}"),
    }

    let mut kdialog = Command::new("kdialog");
    kdialog.arg("--title").arg(title).arg("--getopenfilename");
    if !filters.is_empty() {
        kdialog.arg("audio/source/").arg(filters.join(" "));
    } else {
        kdialog.arg("audio/source/");
    }
    run_dialog_command(&mut kdialog)
}

fn pick_folder_native(title: &str, initial: &str) -> Result<Option<String>, String> {
    let mut zenity = Command::new("zenity");
    zenity
        .arg("--file-selection")
        .arg("--directory")
        .arg("--title")
        .arg(title)
        .arg("--filename")
        .arg(initial);
    match run_dialog_command(&mut zenity) {
        Ok(Some(path)) => return Ok(Some(path)),
        Ok(None) => {}
        Err(err) => eprintln!("zenity folder picker failed: {err}"),
    }

    let mut kdialog = Command::new("kdialog");
    kdialog
        .arg("--title")
        .arg(title)
        .arg("--getexistingdirectory")
        .arg(initial);
    run_dialog_command(&mut kdialog)
}

#[tauri::command]
fn pick_audio_file() -> Result<Option<String>, String> {
    pick_file_native(
        "Open audio",
        &["*.mp3", "*.ogg", "*.wav", "*.flac", "*.aiff", "*.aif"],
    )
}

#[tauri::command]
fn pick_output_dir() -> Result<Option<String>, String> {
    pick_folder_native("Choose output folder", "audio/converted/")
}

#[tauri::command]
fn pick_instrument_dir() -> Result<Option<String>, String> {
    pick_folder_native("Choose Furnace instrument folder", "audio/instruments/")
}

#[tauri::command]
fn pick_instrument_bank() -> Result<Option<String>, String> {
    pick_file_native("Open Vand-AI-lism instrument bank", &["*.json"])
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
async fn analyse_audio(
    input: String,
    output_dir: String,
    ai_backend: String,
    ai_home: String,
    ai_tool_dir: String,
) -> Result<AnalysisResult, String> {
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
        let (ai_log, ai_notes_json) = run_optional_ai_backend(
            &input_path,
            &bundle_dir,
            &ai_backend,
            &PathBuf::from(if ai_home.is_empty() {
                default_ai_home()
            } else {
                ai_home
            }),
            &PathBuf::from(ai_tool_dir),
        )
        .unwrap_or_else(|err| (format!("AI backend error: {err}"), None));
        let log = format!("{log}\n{ai_log}\n");

        let ai_manifest = bundle_dir.join("ai").join("manifest.json");
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
            render_plan: bundle_dir
                .join("render_plan.vand-audio.json")
                .display()
                .to_string(),
            preview_report: preview_report_path.display().to_string(),
            key: preview_report.key,
            bpm: preview_report.bpm,
            bass_notes: preview_report.bass_notes,
            lead_notes: preview_report.lead_notes,
            drum_events: preview_report.drum_events,
            loop_start: preview_report.loop_start,
            loop_end: preview_report.loop_end,
            import_metadata: bundle_dir.join("import.json").display().to_string(),
            dac_preview: bundle_dir.join("dac_preview.wav").display().to_string(),
            runtime_preview: bundle_dir.join("runtime_preview.wav").display().to_string(),
            bundle_dir: bundle_dir.display().to_string(),
            ai_manifest: if ai_manifest.exists() {
                ai_manifest.display().to_string()
            } else {
                String::new()
            },
            ai_notes: ai_notes_json
                .map(|path| path.display().to_string())
                .unwrap_or_default(),
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
async fn read_text_file(path: String) -> Result<String, String> {
    run_blocking(move || {
        let path = PathBuf::from(path);
        fs::read_to_string(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))
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
async fn note_preview_data_url(
    path: String,
    bank_path: String,
    bass_gain: f32,
    lead_gain: f32,
    psg_gain: f32,
    dac_gain: f32,
) -> Result<String, String> {
    run_blocking(move || {
        let repo = repo_root()?;
        let arrangement_path = PathBuf::from(path);
        let text = fs::read_to_string(&arrangement_path)
            .map_err(|err| format!("failed to read {}: {err}", arrangement_path.display()))?;
        let (arrangement, instruments) = if let Ok(plan) =
            serde_json::from_str::<VandRenderPlan>(&text)
        {
            let instruments =
                load_instrument_bank_for_render_plan(&repo, &arrangement_path, &plan, &bank_path)?;
            (render_plan_to_note_arrangement(&plan), instruments)
        } else {
            let arrangement: VandNoteArrangement = serde_json::from_str(&text)
                .map_err(|err| format!("invalid render plan or note arrangement JSON: {err}"))?;
            let instruments = load_instrument_bank_for_note_arrangement(
                &repo,
                &arrangement_path,
                &arrangement,
                &bank_path,
            )?;
            (arrangement, instruments)
        };
        let arrangement_dir = arrangement_path.parent().unwrap_or_else(|| Path::new("."));
        let mix = RenderMix {
            bass_gain: bass_gain.clamp(0.0, 2.0),
            lead_gain: lead_gain.clamp(0.0, 2.0),
            psg_gain: psg_gain.clamp(0.0, 2.0),
            dac_gain: dac_gain.clamp(0.0, 2.0),
        };
        let mut samples =
            render_note_arrangement_samples(&arrangement, &instruments, arrangement_dir, mix);
        normalize_preview_samples(&mut samples, 22_000);
        let preview = std::env::temp_dir().join("vand-ai-lism-note-preview.wav");
        write_preview_wav(&preview, 44_100, &samples)?;
        let bytes = fs::read(&preview)
            .map_err(|err| format!("failed to read {}: {err}", preview.display()))?;
        Ok(format!("data:audio/wav;base64,{}", base64_encode(&bytes)))
    })
    .await
}

#[tauri::command]
async fn ai_note_preview_data_url(
    path: String,
    bank_path: String,
    lead_gain: f32,
) -> Result<String, String> {
    run_blocking(move || {
        let repo = repo_root()?;
        let ai_notes_path = PathBuf::from(path);
        let text = fs::read_to_string(&ai_notes_path)
            .map_err(|err| format!("failed to read {}: {err}", ai_notes_path.display()))?;
        let ai_notes: VandAiNotes =
            serde_json::from_str(&text).map_err(|err| format!("invalid AI notes JSON: {err}"))?;
        let instrument_bank = if bank_path.is_empty() {
            DEFAULT_INSTRUMENT_BANK.to_string()
        } else {
            bank_path.clone()
        };
        let arrangement = ai_notes_to_note_arrangement(&ai_notes, instrument_bank);
        let instruments = load_instrument_bank_for_note_arrangement(
            &repo,
            &ai_notes_path,
            &arrangement,
            &bank_path,
        )?;
        let arrangement_dir = ai_notes_path.parent().unwrap_or_else(|| Path::new("."));
        let mix = RenderMix {
            bass_gain: 0.0,
            lead_gain: lead_gain.clamp(0.0, 2.0),
            psg_gain: 1.0,
            dac_gain: 0.0,
        };
        let mut samples =
            render_note_arrangement_samples(&arrangement, &instruments, arrangement_dir, mix);
        normalize_preview_samples(&mut samples, 22_000);
        let preview = std::env::temp_dir().join("vand-ai-lism-ai-note-preview.wav");
        write_preview_wav(&preview, 44_100, &samples)?;
        let bytes = fs::read(&preview)
            .map_err(|err| format!("failed to read {}: {err}", preview.display()))?;
        Ok(format!("data:audio/wav;base64,{}", base64_encode(&bytes)))
    })
    .await
}

#[tauri::command]
async fn hybrid_note_preview_data_url(
    path: String,
    ai_path: String,
    bank_path: String,
    bass_gain: f32,
    lead_gain: f32,
    psg_gain: f32,
    dac_gain: f32,
) -> Result<String, String> {
    run_blocking(move || {
        let repo = repo_root()?;
        let arrangement_path = PathBuf::from(path);
        let text = fs::read_to_string(&arrangement_path)
            .map_err(|err| format!("failed to read {}: {err}", arrangement_path.display()))?;
        let base = if let Ok(plan) = serde_json::from_str::<VandRenderPlan>(&text) {
            render_plan_to_note_arrangement(&plan)
        } else {
            serde_json::from_str::<VandNoteArrangement>(&text)
                .map_err(|err| format!("invalid render plan or note arrangement JSON: {err}"))?
        };

        let ai_notes_path = PathBuf::from(ai_path);
        let ai_text = fs::read_to_string(&ai_notes_path)
            .map_err(|err| format!("failed to read {}: {err}", ai_notes_path.display()))?;
        let ai_notes: VandAiNotes = serde_json::from_str(&ai_text)
            .map_err(|err| format!("invalid AI notes JSON: {err}"))?;
        let instrument_bank = if bank_path.is_empty() {
            DEFAULT_INSTRUMENT_BANK.to_string()
        } else {
            bank_path.clone()
        };
        let arrangement = merge_hybrid_note_arrangement(base, &ai_notes, instrument_bank);
        let instruments = load_instrument_bank_for_note_arrangement(
            &repo,
            &arrangement_path,
            &arrangement,
            &bank_path,
        )?;
        let arrangement_dir = arrangement_path.parent().unwrap_or_else(|| Path::new("."));
        let mix = RenderMix {
            bass_gain: bass_gain.clamp(0.0, 2.0),
            lead_gain: lead_gain.clamp(0.0, 2.0),
            psg_gain: psg_gain.clamp(0.0, 2.0),
            dac_gain: dac_gain.clamp(0.0, 2.0),
        };
        let mut samples =
            render_note_arrangement_samples(&arrangement, &instruments, arrangement_dir, mix);
        normalize_preview_samples(&mut samples, 22_000);
        let preview = std::env::temp_dir().join("vand-ai-lism-hybrid-note-preview.wav");
        write_preview_wav(&preview, 44_100, &samples)?;
        let bytes = fs::read(&preview)
            .map_err(|err| format!("failed to read {}: {err}", preview.display()))?;
        Ok(format!("data:audio/wav;base64,{}", base64_encode(&bytes)))
    })
    .await
}

fn main() {
    configure_native_wayland_environment();

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
            read_text_file,
            instrument_preview_data_url,
            arrangement_preview_data_url,
            note_preview_data_url,
            ai_note_preview_data_url,
            hybrid_note_preview_data_url
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
            RenderMix::default(),
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
    fn renders_render_plan_with_core_bank() {
        let repo = repo_root().expect("repo root");
        let plan_path = repo.join("audio/converted/test-render-plan.vand-audio.json");
        let plan = VandRenderPlan {
            sample_rate: 44_100,
            hop: 512,
            total_frames: 96,
            instrument_bank: DEFAULT_INSTRUMENT_BANK.to_string(),
            tracks: vec![
                VandRenderTrack {
                    role: "bass".to_string(),
                    events: vec![VandRenderNoteEvent {
                        start_frame: 0,
                        frames: 48,
                        hz: 110.0,
                        velocity: 0.8,
                        instrument_id: DEFAULT_BASS_INSTRUMENT.to_string(),
                    }],
                },
                VandRenderTrack {
                    role: "lead".to_string(),
                    events: vec![VandRenderNoteEvent {
                        start_frame: 16,
                        frames: 32,
                        hz: 220.0,
                        velocity: 0.7,
                        instrument_id: DEFAULT_LEAD_INSTRUMENT.to_string(),
                    }],
                },
            ],
            drums: vec![VandRenderDrum {
                start_frame: 24,
                velocity: 0.8,
                psg_noise_amp: 0.5,
                psg_instrument_id: DEFAULT_NOISE_INSTRUMENT.to_string(),
                dac_chunk: None,
                dac_path: String::new(),
            }],
        };
        let arrangement = render_plan_to_note_arrangement(&plan);
        let instruments =
            load_instrument_bank_for_render_plan(&repo, &plan_path, &plan, "").expect("core bank");
        let samples = render_note_arrangement_samples(
            &arrangement,
            &instruments,
            plan_path.parent().unwrap(),
            RenderMix::default(),
        );
        assert!(!samples.is_empty());
        assert!(samples.iter().any(|sample| *sample != 0));
    }

    #[test]
    fn serializes_audio_lab_settings_as_toml() {
        let mut presets = HashMap::new();
        presets.insert("preview".to_string(), "note".to_string());
        let settings = AudioLabSettings {
            source: "/tmp/song.ogg".to_string(),
            output_dir: "audio/converted".to_string(),
            instrument_bank: "audio/instruments/core.json".to_string(),
            ai_backend: "off".to_string(),
            ai_home: "/home/nichlas/ai".to_string(),
            ai_tool_dir: "/home/nichlas/ai/vand-ai-lism/bin".to_string(),
            presets,
        };
        let text = toml::to_string_pretty(&settings).expect("settings toml");
        assert!(text.contains("source = \"/tmp/song.ogg\""));
        assert!(text.contains("[presets]"));
        let decoded: AudioLabSettings = toml::from_str(&text).expect("decode settings");
        assert_eq!(decoded.source, settings.source);
        assert_eq!(decoded.output_dir, settings.output_dir);
        assert_eq!(decoded.instrument_bank, settings.instrument_bank);
        assert_eq!(decoded.ai_tool_dir, settings.ai_tool_dir);
        assert_eq!(
            decoded.presets.get("preview").map(String::as_str),
            Some("note")
        );
    }

    #[test]
    fn parses_basic_pitch_csv_to_ai_notes_json() {
        let dir = std::env::temp_dir().join("vand-ai-lism-basic-pitch-csv-test");
        fs::create_dir_all(&dir).expect("temp dir");
        let csv = dir.join("song_basic_pitch.csv");
        let out = dir.join("ai_notes.vand-ai-notes.json");
        fs::write(
            &csv,
            "start_time_s,end_time_s,pitch_midi,velocity,pitch_bend\n0.100,0.350,64,96,1,1\n0.400,0.550,52,48,1\n",
        )
        .expect("csv");
        parse_basic_pitch_csv(&csv, &out).expect("parse csv");
        let text = fs::read_to_string(out).expect("ai notes");
        assert!(text.contains("\"format\": \"vand-ai-lism-ai-notes-v0\""));
        assert!(text.contains("\"role\": \"ai_basic_pitch\""));
        assert!(text.contains("\"midi\": 64"));
        assert!(text.contains("\"velocity\": 0.75591"));
    }

    #[test]
    fn renders_ai_notes_preview_with_core_bank() {
        let repo = repo_root().expect("repo root");
        let dir = std::env::temp_dir().join("vand-ai-lism-ai-preview-test");
        fs::create_dir_all(&dir).expect("temp dir");
        let csv = dir.join("song_basic_pitch.csv");
        let out = dir.join("ai_notes.vand-ai-notes.json");
        fs::write(
            &csv,
            "start_time_s,end_time_s,pitch_midi,velocity,pitch_bend\n0.000,0.500,64,96,1\n0.500,0.900,67,88,1\n",
        )
        .expect("csv");
        parse_basic_pitch_csv(&csv, &out).expect("parse csv");
        let text = fs::read_to_string(&out).expect("ai notes");
        let ai_notes: VandAiNotes = serde_json::from_str(&text).expect("ai notes json");
        let arrangement =
            ai_notes_to_note_arrangement(&ai_notes, DEFAULT_INSTRUMENT_BANK.to_string());
        let instruments = load_instrument_bank_for_note_arrangement(&repo, &out, &arrangement, "")
            .expect("core bank");
        let samples = render_note_arrangement_samples(
            &arrangement,
            &instruments,
            out.parent().unwrap(),
            RenderMix::default(),
        );
        assert!(!samples.is_empty());
        assert!(samples.iter().any(|sample| *sample != 0));
    }

    #[test]
    fn selects_melodic_ai_lead_over_low_notes() {
        let ai = VandAiNotes {
            sample_rate: 44_100,
            hop: 512,
            total_seconds: 1.0,
            tracks: vec![VandAiNoteTrack {
                role: "ai_basic_pitch".to_string(),
                events: vec![
                    VandAiNoteEvent {
                        start_time: 0.00,
                        duration: 0.30,
                        hz: midi_to_hz(36),
                        midi: 36,
                        velocity: 0.95,
                    },
                    VandAiNoteEvent {
                        start_time: 0.02,
                        duration: 0.24,
                        hz: midi_to_hz(72),
                        midi: 72,
                        velocity: 0.70,
                    },
                    VandAiNoteEvent {
                        start_time: 0.20,
                        duration: 0.20,
                        hz: midi_to_hz(74),
                        midi: 74,
                        velocity: 0.78,
                    },
                ],
            }],
        };
        let selected = select_ai_lead_notes(&ai);
        assert!(selected.iter().any(|event| event.midi == 72));
        assert!(selected.iter().all(|event| event.midi != 36));
    }

    #[test]
    fn maps_ai_notes_to_multiple_limited_roles() {
        let ai = VandAiNotes {
            sample_rate: 44_100,
            hop: 512,
            total_seconds: 1.0,
            tracks: vec![VandAiNoteTrack {
                role: "ai_basic_pitch".to_string(),
                events: vec![
                    VandAiNoteEvent {
                        start_time: 0.00,
                        duration: 0.40,
                        hz: midi_to_hz(40),
                        midi: 40,
                        velocity: 0.80,
                    },
                    VandAiNoteEvent {
                        start_time: 0.02,
                        duration: 0.25,
                        hz: midi_to_hz(67),
                        midi: 67,
                        velocity: 0.72,
                    },
                    VandAiNoteEvent {
                        start_time: 0.08,
                        duration: 0.18,
                        hz: midi_to_hz(84),
                        midi: 84,
                        velocity: 0.62,
                    },
                    VandAiNoteEvent {
                        start_time: 0.10,
                        duration: 0.40,
                        hz: midi_to_hz(60),
                        midi: 60,
                        velocity: 0.60,
                    },
                    VandAiNoteEvent {
                        start_time: 0.12,
                        duration: 0.36,
                        hz: midi_to_hz(64),
                        midi: 64,
                        velocity: 0.58,
                    },
                ],
            }],
        };
        let arrangement = ai_notes_to_note_arrangement(&ai, DEFAULT_INSTRUMENT_BANK.to_string());
        assert!(arrangement.notes.iter().any(|note| note.track == "bass"));
        assert!(arrangement.notes.iter().any(|note| note.track == "lead"));
        assert!(arrangement.notes.iter().any(|note| note.track == "counter"));
        assert!(arrangement.notes.iter().any(|note| note.track == "pad"));
        assert!(arrangement.notes.iter().any(|note| note.track == "chord"));
        assert!(arrangement.drums.len() >= 6);
    }

    #[test]
    fn renders_hybrid_ai_lead_with_rust_bass() {
        let repo = repo_root().expect("repo root");
        let dir = std::env::temp_dir().join("vand-ai-lism-hybrid-preview-test");
        fs::create_dir_all(&dir).expect("temp dir");
        let csv = dir.join("song_basic_pitch.csv");
        let out = dir.join("ai_notes.vand-ai-notes.json");
        fs::write(
            &csv,
            "start_time_s,end_time_s,pitch_midi,velocity,pitch_bend\n0.000,0.500,76,104,1\n0.500,0.900,79,96,1\n",
        )
        .expect("csv");
        parse_basic_pitch_csv(&csv, &out).expect("parse csv");
        let text = fs::read_to_string(&out).expect("ai notes");
        let ai_notes: VandAiNotes = serde_json::from_str(&text).expect("ai notes json");
        let base = VandNoteArrangement {
            sample_rate: 44_100,
            hop: 512,
            total_frames: 96,
            instrument_bank: DEFAULT_INSTRUMENT_BANK.to_string(),
            notes: vec![
                VandNoteEvent {
                    track: "bass".to_string(),
                    start_frame: 0,
                    frames: 80,
                    hz: 110.0,
                    velocity: 0.8,
                    instrument_id: DEFAULT_BASS_INSTRUMENT.to_string(),
                },
                VandNoteEvent {
                    track: "lead".to_string(),
                    start_frame: 0,
                    frames: 80,
                    hz: 220.0,
                    velocity: 0.8,
                    instrument_id: DEFAULT_LEAD_INSTRUMENT.to_string(),
                },
            ],
            drums: Vec::new(),
        };
        let arrangement =
            merge_hybrid_note_arrangement(base, &ai_notes, DEFAULT_INSTRUMENT_BANK.to_string());
        assert!(arrangement.notes.iter().any(|note| note.track == "bass"));
        assert!(
            arrangement
                .notes
                .iter()
                .any(|note| { note.track == "lead" && (note.hz - midi_to_hz(76)).abs() < 0.5 })
        );
        assert!(
            !arrangement
                .notes
                .iter()
                .any(|note| { note.track == "lead" && (note.hz - 220.0).abs() < 0.5 })
        );
        assert!(arrangement.drums.len() >= 6);
        let instruments = load_instrument_bank_for_note_arrangement(&repo, &out, &arrangement, "")
            .expect("core bank");
        let samples = render_note_arrangement_samples(
            &arrangement,
            &instruments,
            out.parent().unwrap(),
            RenderMix::default(),
        );
        assert!(samples.iter().any(|sample| *sample != 0));
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
        mix_dac_drums(&mut out, &drums, &dir, 32, 1.0);
        assert!(out.iter().any(|sample| *sample != 0));

        let fallback_drums = vec![VandDrumEvent {
            start_frame: 1,
            velocity: 0.85,
            noise_amp: 0.20,
            instrument_id: DEFAULT_NOISE_INSTRUMENT.to_string(),
            dac_chunk: None,
            dac_path: String::new(),
        }];
        let mut fallback_out = vec![0i16; 8_000];
        mix_dac_drums(&mut fallback_out, &fallback_drums, &dir, 32, 1.0);
        assert!(fallback_out.iter().any(|sample| *sample != 0));
    }
}
