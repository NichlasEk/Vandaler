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

#[derive(Serialize)]
struct AnalysisSummary {
    frames: u32,
    active_frames: u32,
    runtime_events: u32,
    dac_chunks: u32,
    arrangement: String,
    bundle_dir: String,
    import_metadata: String,
    dac_preview: String,
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

fn amp_to_fm_attenuation(amp: f32) -> u8 {
    let amp = amp.clamp(0.0, 1.0);
    ((1.0 - amp) * 48.0).round().clamp(0.0, 64.0) as u8
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
        out.extend(
            audio
                .render_frame_samples(FRAME_SAMPLES, RATE as usize)
                .into_iter()
                .map(|sample| (sample.clamp(-1.0, 1.0) * 32767.0) as i16),
        );
    }
    out
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
    let bank_path = if !explicit_bank_path.is_empty() {
        PathBuf::from(explicit_bank_path)
    } else if !arrangement.instrument_bank.is_empty() {
        resolve_existing_path(repo, arrangement_dir, &arrangement.instrument_bank)
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

fn render_arrangement_samples(
    arrangement: &VandArrangement,
    instruments: &HashMap<String, VandInstrument>,
) -> Vec<i16> {
    const RATE: u32 = 44_100;
    let frame_samples = ((arrangement.hop as u64 * RATE as u64
        + (arrangement.sample_rate / 2) as u64)
        / arrangement.sample_rate.max(1) as u64)
        .max(64) as usize;
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
                    write_fm_patch(
                        &mut audio,
                        instrument,
                        0,
                        amp_to_fm_attenuation(frame.bass_amp),
                    );
                }
                bass_on = false;
            } else if let Some(instrument) = instruments.get(&bass_id) {
                write_fm_patch(
                    &mut audio,
                    instrument,
                    0,
                    amp_to_fm_attenuation(frame.bass_amp),
                );
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
                    write_fm_patch(
                        &mut audio,
                        instrument,
                        1,
                        amp_to_fm_attenuation(frame.lead_amp),
                    );
                }
                lead_on = false;
            } else if let Some(instrument) = instruments.get(&lead_id) {
                write_fm_patch(
                    &mut audio,
                    instrument,
                    1,
                    amp_to_fm_attenuation(frame.lead_amp),
                );
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
        if instruments.contains_key(noise_id) && noise_amp > 0.015 {
            psg_set_noise(&mut audio, noise_amp);
        } else {
            psg_latch_volume(&mut audio, 3, 15);
        }

        out.extend(
            audio
                .render_frame_samples(frame_samples, RATE as usize)
                .into_iter()
                .map(|sample| (sample.clamp(-1.0, 1.0) * 32767.0) as i16),
        );
    }

    if bass_on {
        fm_key_on(&mut audio, 0, false);
    }
    if lead_on {
        fm_key_on(&mut audio, 1, false);
    }

    out
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
fn analyse_audio(input: String, output_dir: String) -> Result<AnalysisResult, String> {
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
    let summary = AnalysisSummary {
        frames: parse_metric(&log, "frames,"),
        active_frames: parse_metric(&log, "active,"),
        runtime_events: parse_metric(&log, "runtime events,"),
        dac_chunks: parse_metric(&log, "dac chunks"),
        arrangement: output_path.display().to_string(),
        import_metadata: bundle_dir.join("import.json").display().to_string(),
        dac_preview: bundle_dir.join("dac_preview.wav").display().to_string(),
        bundle_dir: bundle_dir.display().to_string(),
    };

    Ok(AnalysisResult { summary, log })
}

#[tauri::command]
fn import_instrument_dir(
    input_dir: String,
    output_dir: String,
) -> Result<InstrumentBankResult, String> {
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
}

#[tauri::command]
fn audio_data_url(path: String) -> Result<String, String> {
    let path = PathBuf::from(path);
    let mime = mime_for_path(&path)?;
    let bytes =
        fs::read(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    Ok(format!("data:{mime};base64,{}", base64_encode(&bytes)))
}

#[tauri::command]
fn instrument_preview_data_url(path: String) -> Result<String, String> {
    let path = PathBuf::from(path);
    let instrument = read_instrument(&path)?;
    let samples = render_instrument_samples(&instrument);
    let preview = std::env::temp_dir().join("vand-ai-lism-instrument-preview.wav");
    write_preview_wav(&preview, 44_100, &samples)?;
    let bytes =
        fs::read(&preview).map_err(|err| format!("failed to read {}: {err}", preview.display()))?;
    Ok(format!("data:audio/wav;base64,{}", base64_encode(&bytes)))
}

#[tauri::command]
fn arrangement_preview_data_url(path: String, bank_path: String) -> Result<String, String> {
    let repo = repo_root()?;
    let arrangement_path = PathBuf::from(path);
    let text = fs::read_to_string(&arrangement_path)
        .map_err(|err| format!("failed to read {}: {err}", arrangement_path.display()))?;
    let arrangement: VandArrangement =
        serde_json::from_str(&text).map_err(|err| format!("invalid arrangement JSON: {err}"))?;
    let instruments =
        load_instrument_bank_for_arrangement(&repo, &arrangement_path, &arrangement, &bank_path)?;
    let samples = render_arrangement_samples(&arrangement, &instruments);
    let preview = std::env::temp_dir().join("vand-ai-lism-arrangement-preview.wav");
    write_preview_wav(&preview, 44_100, &samples)?;
    let bytes =
        fs::read(&preview).map_err(|err| format!("failed to read {}: {err}", preview.display()))?;
    Ok(format!("data:audio/wav;base64,{}", base64_encode(&bytes)))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            pick_audio_file,
            pick_output_dir,
            pick_instrument_dir,
            pick_instrument_bank,
            load_instrument_bank,
            analyse_audio,
            import_instrument_dir,
            audio_data_url,
            instrument_preview_data_url,
            arrangement_preview_data_url
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
        let samples = render_arrangement_samples(&arrangement, &instruments);
        assert!(!samples.is_empty());
        assert!(samples.iter().any(|sample| *sample != 0));
    }
}
