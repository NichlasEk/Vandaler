use euther_oxide::audio::Audio;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    source_dir: String,
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

#[derive(Deserialize)]
struct VandInstrument {
    name: String,
    chip: String,
    category: String,
    fm: Option<VandFmPatch>,
}

#[derive(Deserialize)]
struct VandFmPatch {
    algorithm: u8,
    feedback: u8,
    operators: Vec<VandFmOperator>,
}

#[derive(Deserialize)]
struct VandFmOperator {
    mult: u8,
    tl: u8,
    ar: u8,
    dr: u8,
    sl: u8,
    rr: u8,
    dt: u8,
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

fn render_instrument_samples(instrument: &VandInstrument) -> Vec<i16> {
    const RATE: u32 = 44_100;
    const FRAME_SAMPLES: usize = 735;
    const FRAMES: usize = 96;
    const KEY_OFF_FRAME: usize = 62;
    const CHANNEL: usize = 0;
    const OP_OFFSETS: [u8; 4] = [0x00, 0x04, 0x08, 0x0c];
    let mut audio = Audio::new();
    audio.reset();
    let mut out = Vec::with_capacity(FRAMES * FRAME_SAMPLES);
    let name_lc = instrument.name.to_ascii_lowercase();
    let (fnum, block) = if instrument.category == "bass" || name_lc.contains("bass") {
        ym2612_pitch_for_hz(110.0)
    } else {
        ym2612_pitch_for_hz(220.0)
    };

    if let Some(fm) = instrument.fm.as_ref() {
        audio.ym2612.write_register(0, 0x22, 0x00, None, true);
        audio.ym2612.write_register(0, 0x27, 0x00, None, true);
        audio.ym2612.write_register(
            0,
            0xb0 + CHANNEL as u8,
            (fm.algorithm & 0x07) | ((fm.feedback & 0x07) << 3),
            None,
            true,
        );
        audio
            .ym2612
            .write_register(0, 0xb4 + CHANNEL as u8, 0xc0, None, true);
        for (index, op) in fm.operators.iter().take(4).enumerate() {
            let reg = OP_OFFSETS[index] + CHANNEL as u8;
            audio.ym2612.write_register(
                0,
                0x30 + reg,
                ((op.dt & 0x07) << 4) | (op.mult & 0x0f),
                None,
                true,
            );
            audio
                .ym2612
                .write_register(0, 0x40 + reg, op.tl.min(127), None, true);
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
        audio.ym2612.write_register(
            0,
            0xa4 + CHANNEL as u8,
            ((block & 0x07) << 3) | ((fnum >> 8) as u8 & 0x07),
            None,
            true,
        );
        audio
            .ym2612
            .write_register(0, 0xa0 + CHANNEL as u8, (fnum & 0xff) as u8, None, true);
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
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let instrument: VandInstrument =
        serde_json::from_str(&text).map_err(|err| format!("invalid instrument JSON: {err}"))?;
    let samples = render_instrument_samples(&instrument);
    let preview = std::env::temp_dir().join("vand-ai-lism-instrument-preview.wav");
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
            instrument_preview_data_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vand-AI-lism");
}
