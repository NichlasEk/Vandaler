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
    const SECONDS: f32 = 1.35;
    let sample_count = (RATE as f32 * SECONDS) as usize;
    let mut out = Vec::with_capacity(sample_count);
    let name_lc = instrument.name.to_ascii_lowercase();
    let base_hz = if instrument.category == "bass" || name_lc.contains("bass") {
        110.0
    } else {
        220.0
    };
    let fm = instrument.fm.as_ref();

    for i in 0..sample_count {
        let t = i as f32 / RATE as f32;
        let release_start = 1.0;
        let release = if t > release_start {
            (1.0 - (t - release_start) / (SECONDS - release_start)).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let attack = (t / 0.035).min(1.0);
        let envelope = attack * release;
        let mut value = 0.0f32;

        if let Some(fm) = fm {
            let feedback = 1.0 + fm.feedback as f32 * 0.065;
            let algorithm_bias = 1.0 + fm.algorithm as f32 * 0.025;
            for (op_index, op) in fm.operators.iter().enumerate() {
                let mult = op.mult.max(1) as f32;
                let level = (1.0 - (op.tl.min(96) as f32 / 96.0)).powf(1.35);
                let rate_shape =
                    0.65 + op.ar as f32 * 0.012 - op.dr as f32 * 0.004 + op.rr as f32 * 0.003;
                let sustain = 1.0 - op.sl.min(15) as f32 / 22.0;
                let detune = (op.dt as f32 - 3.0) * 0.003;
                let phase = 2.0
                    * std::f32::consts::PI
                    * base_hz
                    * (mult * algorithm_bias + detune)
                    * feedback
                    * t;
                let op_env = envelope * sustain.max(0.18) * rate_shape.clamp(0.2, 1.4);
                value += phase.sin() * level * op_env / (op_index as f32 + 1.25);
            }
        } else if instrument.chip == "sn76489" {
            let phase = 2.0 * std::f32::consts::PI * 440.0 * t;
            value = if phase.sin() >= 0.0 { 0.35 } else { -0.35 };
            value *= envelope;
        }

        let sample = (value.clamp(-0.92, 0.92) * 32767.0) as i16;
        out.push(sample);
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
