use serde::Serialize;
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
fn analyse_audio(input: String, output_dir: String) -> Result<AnalysisResult, String> {
    let repo = repo_root()?;
    let input_path = PathBuf::from(&input);
    let output_dir = PathBuf::from(&output_dir);
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            pick_audio_file,
            pick_output_dir,
            analyse_audio
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vand-AI-lism");
}
