use euther_oxide::{Emulator, controller::Controller};
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_SECONDS: u32 = 5;
const DEFAULT_RATE: u32 = 44_100;
const ANALYSIS_FRAME: usize = 1024;
const ANALYSIS_HOP: usize = 512;
const DAC_SAMPLE_RATE: u32 = 8_000;
const DEFAULT_INSTRUMENT_BANK: &str =
    "audio/instruments/vand_furnace_core/bank.vand-instruments.json";
const DEFAULT_BASS_INSTRUMENT: &str = "growl_bass_wobbly";
const DEFAULT_LEAD_INSTRUMENT: &str = "fm_grinder";
const DEFAULT_NOISE_INSTRUMENT: &str = "psg_echo_warble";

struct Args {
    command: CommandMode,
    rom: Option<PathBuf>,
    wav: PathBuf,
    report: Option<PathBuf>,
    out: Option<PathBuf>,
    seconds: u32,
    sample_rate: u32,
    play: bool,
    install_sgdk: bool,
    start_malmo: bool,
}

#[derive(Clone, Copy)]
enum CommandMode {
    RenderRom,
    TestRom,
    AnalyseWav,
    InspectInstrument,
    ImportInstrument,
    ImportInstrumentDir,
    Gui,
}

fn usage() {
    eprintln!(
        "usage:\n  vandaler-audio-lab gui\n  vandaler-audio-lab analyse-audio input.mp3|input.ogg|input.wav [--out arrangement.vand-audio.json] [--install-sgdk]\n  vandaler-audio-lab analyse-wav input.wav [--out arrangement.vand-audio.json] [--install-sgdk]\n  vandaler-audio-lab inspect-instrument instrument.fui\n  vandaler-audio-lab import-instrument instrument.fui [--out audio/instruments/imported/name.vand-instrument.json]\n  vandaler-audio-lab import-instrument-dir instruments/OPN/bass [--out audio/instruments/imported]\n  vandaler-audio-lab render-rom ROM [--wav out.wav] [--report report.json] [--seconds N] [--rate HZ] [--play] [--start-malmo]\n  vandaler-audio-lab test-rom [--wav out/audio-test.wav] [--report out/audio-test-report.json] [--seconds N] [--rate HZ] [--play]"
    );
}

fn parse_args() -> io::Result<Args> {
    let mut iter = env::args().skip(1);
    let mut command = None;
    let mut rom = None;
    let mut wav = PathBuf::from("out/audio-lab-render.wav");
    let mut report = None;
    let mut out = None;
    let mut seconds = DEFAULT_SECONDS;
    let mut sample_rate = DEFAULT_RATE;
    let mut play = false;
    let mut install_sgdk = false;
    let mut start_malmo = false;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "render-rom" if command.is_none() => command = Some(CommandMode::RenderRom),
            "analyse-audio" | "analyse-wav" if command.is_none() => {
                command = Some(CommandMode::AnalyseWav)
            }
            "inspect-instrument" if command.is_none() => {
                command = Some(CommandMode::InspectInstrument)
            }
            "import-instrument" if command.is_none() => {
                command = Some(CommandMode::ImportInstrument)
            }
            "import-instrument-dir" if command.is_none() => {
                command = Some(CommandMode::ImportInstrumentDir)
            }
            "gui" if command.is_none() => command = Some(CommandMode::Gui),
            "test-rom" if command.is_none() => {
                command = Some(CommandMode::TestRom);
                wav = PathBuf::from("out/audio-test.wav");
                report = Some(PathBuf::from("out/audio-test-report.json"));
            }
            "--wav" => {
                let value = iter.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--wav needs a path")
                })?;
                wav = PathBuf::from(value);
            }
            "--report" => {
                let value = iter.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--report needs a path")
                })?;
                report = Some(PathBuf::from(value));
            }
            "--out" => {
                let value = iter.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--out needs a path")
                })?;
                out = Some(PathBuf::from(value));
            }
            "--seconds" => {
                let value = iter.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--seconds needs a value")
                })?;
                seconds = value.parse().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "invalid --seconds value")
                })?;
            }
            "--rate" => {
                let value = iter.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--rate needs a value")
                })?;
                sample_rate = value.parse().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "invalid --rate value")
                })?;
            }
            "--play" => play = true,
            "--start-malmo" => start_malmo = true,
            "--install-sgdk" => install_sgdk = true,
            "-h" | "--help" => {
                usage();
                std::process::exit(0);
            }
            other if other.starts_with('-') => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown argument {other}"),
                ));
            }
            other => {
                if command.is_none() {
                    command = Some(CommandMode::RenderRom);
                }
                if rom.replace(PathBuf::from(other)).is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("unexpected extra ROM path {other}"),
                    ));
                }
            }
        }
    }

    let command = command.unwrap_or(CommandMode::RenderRom);
    if matches!(
        command,
        CommandMode::RenderRom
            | CommandMode::AnalyseWav
            | CommandMode::InspectInstrument
            | CommandMode::ImportInstrument
            | CommandMode::ImportInstrumentDir
    ) && rom.is_none()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "missing input path",
        ));
    }
    Ok(Args {
        command,
        rom,
        wav,
        report,
        out,
        seconds,
        sample_rate,
        play,
        install_sgdk,
        start_malmo,
    })
}

fn write_wav_i16_stereo(path: &Path, sample_rate: u32, samples: &[i16]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    let data_len = (samples.len() * 2) as u32;
    let byte_rate = sample_rate * 2 * 2;
    let block_align = 2 * 2;

    file.write_all(b"RIFF")?;
    file.write_all(&(36 + data_len).to_le_bytes())?;
    file.write_all(b"WAVE")?;
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&2u16.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&(block_align as u16).to_le_bytes())?;
    file.write_all(&16u16.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_len.to_le_bytes())?;
    for sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }
    Ok(())
}

struct AudioStats {
    frames: u32,
    sample_rate: u32,
    stereo_samples: usize,
    peak: i16,
    nonzero_samples: usize,
    rms: f64,
}

fn audio_stats(frames: u32, sample_rate: u32, samples: &[i16]) -> AudioStats {
    let mut sum_sq = 0.0;
    let mut peak = 0i16;
    let mut nonzero_samples = 0usize;

    for sample in samples {
        let abs = sample.saturating_abs();
        if abs > peak {
            peak = abs;
        }
        if *sample != 0 {
            nonzero_samples += 1;
        }
        let normalized = f64::from(*sample) / 32768.0;
        sum_sq += normalized * normalized;
    }

    let rms = if samples.is_empty() {
        0.0
    } else {
        (sum_sq / samples.len() as f64).sqrt()
    };

    AudioStats {
        frames,
        sample_rate,
        stereo_samples: samples.len() / 2,
        peak,
        nonzero_samples,
        rms,
    }
}

fn json_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn write_report(path: &Path, rom: &Path, wav: &Path, stats: &AudioStats) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let report = format!(
        concat!(
            "{{\n",
            "  \"backend\": \"euther_oxide\",\n",
            "  \"rom\": \"{}\",\n",
            "  \"wav\": \"{}\",\n",
            "  \"frames\": {},\n",
            "  \"sample_rate\": {},\n",
            "  \"stereo_samples\": {},\n",
            "  \"peak\": {},\n",
            "  \"nonzero_samples\": {},\n",
            "  \"rms\": {:.8}\n",
            "}}\n"
        ),
        json_escape(&rom.display().to_string()),
        json_escape(&wav.display().to_string()),
        stats.frames,
        stats.sample_rate,
        stats.stereo_samples,
        stats.peak,
        stats.nonzero_samples,
        stats.rms,
    );
    std::fs::write(path, report)
}

fn render_rom_to_wav(
    rom: &Path,
    wav: &Path,
    report: Option<&Path>,
    seconds: u32,
    sample_rate: u32,
    play: bool,
    start_malmo: bool,
) -> io::Result<AudioStats> {
    let mut emulator = Emulator::new();
    let mut pcm = Vec::new();
    let frames = seconds.saturating_mul(60).max(1);

    emulator.load_rom_file(rom)?;
    for frame in 0..frames {
        if start_malmo {
            let press_start = matches!(frame, 12 | 28);
            emulator
                .bus
                .controller_a
                .set_pressed(Controller::START, press_start);
        }
        emulator.run_frame();
        pcm.extend(emulator.render_audio_frame_i16_stereo(sample_rate as usize));
    }

    write_wav_i16_stereo(wav, sample_rate, &pcm)?;
    let stats = audio_stats(frames, sample_rate, &pcm);
    if let Some(report_path) = report {
        write_report(report_path, rom, wav, &stats)?;
    }
    println!(
        "wrote {} frames, {} stereo samples to {} | peak {} | rms {:.6}",
        frames,
        pcm.len() / 2,
        wav.display(),
        stats.peak,
        stats.rms,
    );

    if play {
        let status = Command::new("pw-play").arg(wav).status()?;
        if !status.success() {
            return Err(io::Error::other("pw-play failed"));
        }
    }

    Ok(stats)
}

fn build_audio_test_rom() -> io::Result<PathBuf> {
    let status = Command::new("make").arg("audio-test").status()?;
    if !status.success() {
        return Err(io::Error::other("make audio-test failed"));
    }
    Ok(PathBuf::from("out/audio-test.bin"))
}

fn render_rom(args: &Args) -> io::Result<()> {
    let rom = args
        .rom
        .as_deref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing ROM path"))?;
    render_rom_to_wav(
        rom,
        &args.wav,
        args.report.as_deref(),
        args.seconds,
        args.sample_rate,
        args.play,
        args.start_malmo,
    )?;
    Ok(())
}

fn test_rom(args: &Args) -> io::Result<()> {
    let rom = build_audio_test_rom()?;
    render_rom_to_wav(
        &rom,
        &args.wav,
        args.report.as_deref(),
        args.seconds,
        args.sample_rate,
        args.play,
        false,
    )?;
    Ok(())
}

struct WavData {
    sample_rate: u32,
    samples: Vec<f32>,
}

struct ImportedAudio {
    wav: WavData,
    analysis_input: PathBuf,
    decoded_wav: Option<PathBuf>,
    decoder: &'static str,
}

#[derive(Clone)]
struct AnalysisFrame {
    index: usize,
    time: f32,
    rms: f32,
    transient: f32,
    bass_hz: f32,
    bass_amp: f32,
    lead_hz: f32,
    lead_amp: f32,
    noise_amp: f32,
    class_name: &'static str,
}

#[derive(Clone, PartialEq)]
struct RuntimeEvent {
    frames: u16,
    fm0_fnum: u16,
    fm0_block: u8,
    fm0_level: u8,
    fm1_fnum: u16,
    fm1_block: u8,
    fm1_level: u8,
    psg_noise_level: u8,
    kind: u8,
    dac_chunk: u8,
    dac_level: u8,
}

struct DacChunk {
    id: usize,
    start_frame: usize,
    end_frame: usize,
    start_sample: usize,
    source_sample_count: usize,
    sample_count: usize,
    sample_rate: u32,
    peak: f32,
    rms: f32,
    path: String,
    samples: Vec<u8>,
}

struct AnalysisSummary {
    frames: usize,
    active_frames: usize,
    runtime_events: usize,
    dac_chunks: usize,
    arrangement: PathBuf,
    note_arrangement: PathBuf,
    render_plan: PathBuf,
    preview_report: PathBuf,
    bundle_dir: PathBuf,
    import_metadata: PathBuf,
    dac_preview: PathBuf,
    runtime_preview: PathBuf,
}

#[derive(Clone)]
struct NoteEvent {
    track: &'static str,
    start_frame: usize,
    frames: usize,
    midi: i32,
    hz: f32,
    velocity: f32,
    instrument_id: &'static str,
}

#[derive(Clone, Copy)]
struct MusicalContext {
    key_root: i32,
    mode: &'static str,
    key_confidence: f32,
    bpm: f32,
    frames_per_beat: f32,
    grid_frames: usize,
    loop_start_frame: usize,
    loop_end_frame: usize,
}

#[derive(Clone)]
struct DrumEvent {
    start_frame: usize,
    kind: &'static str,
    velocity: f32,
    noise_amp: f32,
    instrument_id: &'static str,
    dac_chunk: Option<usize>,
    dac_path: String,
}

fn read_u16_le(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[0], bytes[1]])
}

fn read_u32_le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

#[derive(Clone)]
struct FmOperatorPatch {
    mult: u8,
    total_level: u8,
    attack_rate: u8,
    decay_rate: u8,
    sustain_rate: u8,
    sustain_level: u8,
    release_rate: u8,
    detune: u8,
    rate_scale: u8,
    amp_mod: u8,
    ssg_eg: u8,
}

struct FmPatch {
    algorithm: u8,
    feedback: u8,
    ams: u8,
    fms: u8,
    operators: Vec<FmOperatorPatch>,
}

struct MacroSummary {
    code: u8,
    length: u8,
    loop_at: u8,
    release_at: u8,
    mode: u8,
    word_size: u8,
}

struct FurnaceInstrument {
    path: PathBuf,
    format_version: u16,
    instrument_type: u16,
    name: String,
    fm: Option<FmPatch>,
    macros: Vec<MacroSummary>,
    feature_codes: Vec<String>,
}

struct ImportedInstrumentRecord {
    id: String,
    name: String,
    chip: String,
    category: String,
    source: PathBuf,
    output: PathBuf,
}

fn instrument_type_name(kind: u16) -> &'static str {
    match kind {
        0 => "SN76489",
        1 => "FM OPN/YM2612",
        _ => "other",
    }
}

fn chip_name(kind: u16) -> &'static str {
    match kind {
        0 => "sn76489",
        1 => "ym2612",
        _ => "unknown",
    }
}

fn sanitize_id(input: &str) -> String {
    let mut out = String::new();
    let mut previous_was_separator = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            out.push('_');
            previous_was_separator = true;
        }
    }
    let trimmed = out.trim_matches('_').to_string();
    if trimmed.is_empty() {
        "instrument".to_string()
    } else {
        trimmed
    }
}

fn category_from_path(path: &Path) -> String {
    let mut parts = path
        .parent()
        .into_iter()
        .flat_map(|parent| parent.components())
        .filter_map(|component| component.as_os_str().to_str())
        .map(|part| part.to_ascii_lowercase())
        .collect::<Vec<_>>();
    parts.reverse();

    for part in parts {
        match part.as_str() {
            "bass" => return "bass".to_string(),
            "drums" | "percussion" | "smsperc" => return "drum".to_string(),
            "effect" | "effects" | "sfx" => return "sfx".to_string(),
            "guitar" => return "guitar".to_string(),
            "horn" | "wind" => return "lead".to_string(),
            "keys" => return "keys".to_string(),
            "strings" => return "pad".to_string(),
            "synth" => return "lead".to_string(),
            "sn7" => return "psg".to_string(),
            "opn" => return "fm".to_string(),
            _ => {}
        }
    }
    "uncurated".to_string()
}

fn read_c_string(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_string()
}

fn parse_furnace_fm(data: &[u8]) -> io::Result<FmPatch> {
    if data.len() < 5 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "short Furnace FM feature",
        ));
    }

    let flags = data[0];
    let op_count = (flags & 0x0f).clamp(0, 4) as usize;
    let algorithm_feedback = data[1];
    let lfo = data[2];
    let mut offset = 5usize;
    let mut operators = Vec::new();

    for _ in 0..op_count {
        if offset + 8 > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "truncated Furnace FM operator",
            ));
        }
        let op = &data[offset..offset + 8];
        operators.push(FmOperatorPatch {
            mult: op[0] & 0x0f,
            total_level: op[1] & 0x7f,
            attack_rate: op[2] & 0x1f,
            decay_rate: op[3] & 0x1f,
            sustain_rate: op[4] & 0x1f,
            sustain_level: (op[5] >> 4) & 0x0f,
            release_rate: op[5] & 0x0f,
            detune: (op[0] >> 4) & 0x07,
            rate_scale: (op[2] >> 6) & 0x03,
            amp_mod: (op[3] >> 7) & 0x01,
            ssg_eg: op[6] & 0x0f,
        });
        offset += 8;
    }

    Ok(FmPatch {
        algorithm: (algorithm_feedback >> 4) & 0x07,
        feedback: algorithm_feedback & 0x07,
        ams: (lfo >> 3) & 0x03,
        fms: (lfo & 0x07) | ((lfo >> 5) & 0x18),
        operators,
    })
}

fn parse_furnace_macros(data: &[u8]) -> Vec<MacroSummary> {
    if data.len() < 2 {
        return Vec::new();
    }
    let header_len = read_u16_le(&data[0..2]) as usize;
    let mut offset = 2usize.saturating_add(header_len);
    if offset > data.len() {
        offset = 2;
    }

    let mut macros = Vec::new();
    while offset < data.len() {
        let code = data[offset];
        offset += 1;
        if code == 255 {
            break;
        }
        if offset + 6 > data.len() {
            break;
        }
        let length = data[offset];
        let loop_at = data[offset + 1];
        let release_at = data[offset + 2];
        let mode = data[offset + 3];
        let type_word = data[offset + 4];
        let word_size_code = (type_word >> 6) & 0x03;
        let word_size = match word_size_code {
            0 | 1 => 1,
            2 => 2,
            _ => 4,
        };
        offset += 6;
        let data_len = length as usize * word_size as usize;
        if offset + data_len > data.len() {
            break;
        }
        offset += data_len;
        macros.push(MacroSummary {
            code,
            length,
            loop_at,
            release_at,
            mode,
            word_size,
        });
    }
    macros
}

fn parse_old_furnace_instrument(path: &Path, data: &[u8]) -> io::Result<FurnaceInstrument> {
    if data.len() < 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "short old Furnace instrument header",
        ));
    }

    let format_version = read_u16_le(&data[16..18]);
    let instrument_ptr = read_u32_le(&data[20..24]) as usize;
    if instrument_ptr + 12 > data.len() || &data[instrument_ptr..instrument_ptr + 4] != b"INST" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "old Furnace instrument is missing INST block",
        ));
    }

    let block_size = read_u32_le(&data[instrument_ptr + 4..instrument_ptr + 8]) as usize;
    let block_end = if block_size == 0 {
        data.len()
    } else {
        (instrument_ptr + 8 + block_size).min(data.len())
    };
    let block = &data[instrument_ptr..block_end];
    if block.len() < 12 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "short old Furnace INST block",
        ));
    }

    let instrument_type = block[10] as u16;
    let name_start = 12usize;
    let name_end = block[name_start..]
        .iter()
        .position(|byte| *byte == 0)
        .map(|pos| name_start + pos)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing instrument name"))?;
    let name = read_c_string(&block[name_start..name_end]);
    let fm_start = name_end + 1;
    let fm = if instrument_type == 1 && fm_start + 8 + (4 * 32) <= block.len() {
        let header = &block[fm_start..fm_start + 8];
        let mut operators = Vec::new();
        let mut offset = fm_start + 8;
        for _ in 0..4 {
            let op = &block[offset..offset + 32];
            operators.push(FmOperatorPatch {
                amp_mod: op[0],
                attack_rate: op[1],
                decay_rate: op[2],
                mult: op[3],
                release_rate: op[4],
                sustain_level: op[5],
                total_level: op[6],
                detune: op[9],
                sustain_rate: op[10],
                ssg_eg: op[11],
                rate_scale: op[8],
            });
            offset += 32;
        }
        Some(FmPatch {
            algorithm: header[0],
            feedback: header[1],
            fms: header[2],
            ams: header[3],
            operators,
        })
    } else {
        None
    };

    Ok(FurnaceInstrument {
        path: path.to_path_buf(),
        format_version,
        instrument_type,
        name,
        fm,
        macros: Vec::new(),
        feature_codes: vec!["OLD".to_string(), "INST".to_string()],
    })
}

fn parse_furnace_instrument(path: &Path) -> io::Result<FurnaceInstrument> {
    let mut data = Vec::new();
    File::open(path)?.read_to_end(&mut data)?;
    if data.len() >= 16 && &data[0..16] == b"-Furnace instr.-" {
        return parse_old_furnace_instrument(path, &data);
    }
    if data.len() < 8 || &data[0..4] != b"FINS" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected Furnace .fui FINS or old -Furnace instr.- header",
        ));
    }

    let format_version = read_u16_le(&data[4..6]);
    let instrument_type = read_u16_le(&data[6..8]);
    let mut offset = 8usize;
    let mut name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("instrument")
        .to_string();
    let mut fm = None;
    let mut macros = Vec::new();
    let mut feature_codes = Vec::new();

    while offset + 4 <= data.len() {
        let code_bytes = [data[offset], data[offset + 1]];
        let code = String::from_utf8_lossy(&code_bytes).to_string();
        let len = read_u16_le(&data[offset + 2..offset + 4]) as usize;
        offset += 4;
        if offset + len > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("truncated Furnace feature {code}"),
            ));
        }
        let block = &data[offset..offset + len];
        offset += len;
        feature_codes.push(code.clone());

        match code.as_str() {
            "EN" => break,
            "NA" => {
                let parsed = read_c_string(block);
                if !parsed.is_empty() {
                    name = parsed;
                }
            }
            "FM" => fm = Some(parse_furnace_fm(block)?),
            "MA" => macros.extend(parse_furnace_macros(block)),
            _ => {}
        }
    }

    Ok(FurnaceInstrument {
        path: path.to_path_buf(),
        format_version,
        instrument_type,
        name,
        fm,
        macros,
        feature_codes,
    })
}

fn write_furnace_instrument_json(
    path: &Path,
    instrument: &FurnaceInstrument,
    category: &str,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut text = String::new();
    text.push_str("{\n");
    text.push_str("  \"format\": \"vandaler-instrument-v0\",\n");
    text.push_str(&format!(
        "  \"id\": \"{}\",\n",
        json_escape(&sanitize_id(&instrument.name))
    ));
    text.push_str(&format!(
        "  \"name\": \"{}\",\n",
        json_escape(&instrument.name)
    ));
    text.push_str(&format!(
        "  \"chip\": \"{}\",\n",
        chip_name(instrument.instrument_type)
    ));
    text.push_str(&format!("  \"category\": \"{}\",\n", json_escape(category)));
    text.push_str("  \"source\": {\n");
    text.push_str("    \"kind\": \"furnace_fui\",\n");
    text.push_str(&format!(
        "    \"path\": \"{}\",\n",
        json_escape(&instrument.path.display().to_string())
    ));
    text.push_str("    \"license_review\": \"pending_curated_import\"\n");
    text.push_str("  },\n");
    text.push_str(&format!(
        "  \"furnace\": {{ \"format_version\": {}, \"instrument_type\": {}, \"features\": [",
        instrument.format_version, instrument.instrument_type
    ));
    for (i, code) in instrument.feature_codes.iter().enumerate() {
        if i > 0 {
            text.push_str(", ");
        }
        text.push_str(&format!("\"{}\"", json_escape(code)));
    }
    text.push_str("] },\n");

    if let Some(fm) = &instrument.fm {
        text.push_str("  \"fm\": {\n");
        text.push_str(&format!(
            "    \"algorithm\": {}, \"feedback\": {}, \"ams\": {}, \"fms\": {},\n",
            fm.algorithm, fm.feedback, fm.ams, fm.fms
        ));
        text.push_str("    \"operators\": [\n");
        for (i, op) in fm.operators.iter().enumerate() {
            let comma = if i + 1 == fm.operators.len() { "" } else { "," };
            text.push_str(&format!(
                "      {{ \"mult\": {}, \"tl\": {}, \"ar\": {}, \"dr\": {}, \"sr\": {}, \"sl\": {}, \"rr\": {}, \"dt\": {}, \"rs\": {}, \"am\": {}, \"ssg_eg\": {} }}{}\n",
                op.mult,
                op.total_level,
                op.attack_rate,
                op.decay_rate,
                op.sustain_rate,
                op.sustain_level,
                op.release_rate,
                op.detune,
                op.rate_scale,
                op.amp_mod,
                op.ssg_eg,
                comma
            ));
        }
        text.push_str("    ]\n");
        text.push_str("  },\n");
    } else {
        text.push_str("  \"fm\": null,\n");
    }

    text.push_str("  \"macros\": [\n");
    for (i, item) in instrument.macros.iter().enumerate() {
        let comma = if i + 1 == instrument.macros.len() {
            ""
        } else {
            ","
        };
        text.push_str(&format!(
            "    {{ \"code\": {}, \"length\": {}, \"loop\": {}, \"release\": {}, \"mode\": {}, \"word_size\": {} }}{}\n",
            item.code, item.length, item.loop_at, item.release_at, item.mode, item.word_size, comma
        ));
    }
    text.push_str("  ]\n");
    text.push_str("}\n");
    std::fs::write(path, text)
}

fn inspect_instrument(args: &Args) -> io::Result<()> {
    let input = args
        .rom
        .as_deref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing instrument path"))?;
    let instrument = parse_furnace_instrument(input)?;
    println!(
        "{} | Furnace v{} | type {} ({}) | features {}",
        instrument.name,
        instrument.format_version,
        instrument.instrument_type,
        instrument_type_name(instrument.instrument_type),
        instrument.feature_codes.join(",")
    );
    if let Some(fm) = &instrument.fm {
        println!(
            "FM alg={} feedback={} ams={} fms={} operators={}",
            fm.algorithm,
            fm.feedback,
            fm.ams,
            fm.fms,
            fm.operators.len()
        );
        for (i, op) in fm.operators.iter().enumerate() {
            println!(
                "  op{} mult={} tl={} ar={} dr={} sr={} sl={} rr={} dt={} rs={} am={} ssg={}",
                i + 1,
                op.mult,
                op.total_level,
                op.attack_rate,
                op.decay_rate,
                op.sustain_rate,
                op.sustain_level,
                op.release_rate,
                op.detune,
                op.rate_scale,
                op.amp_mod,
                op.ssg_eg
            );
        }
    }
    if !instrument.macros.is_empty() {
        println!("macros={}", instrument.macros.len());
    }
    Ok(())
}

fn import_instrument(args: &Args) -> io::Result<()> {
    let input = args
        .rom
        .as_deref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing instrument path"))?;
    let instrument = parse_furnace_instrument(input)?;
    let out = args.out.clone().unwrap_or_else(|| {
        let stem = input
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("instrument");
        PathBuf::from("audio/instruments/imported").join(format!("{stem}.vand-instrument.json"))
    });
    let category = category_from_path(input);
    write_furnace_instrument_json(&out, &instrument, &category)?;
    println!(
        "imported {} ({}, {}) to {}",
        instrument.name,
        chip_name(instrument.instrument_type),
        category,
        out.display()
    );
    Ok(())
}

fn collect_fui_files(root: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_fui_files(&path, out)?;
        } else if file_type.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("fui"))
        {
            out.push(path);
        }
    }
    Ok(())
}

fn unique_output_path(base: &Path, id: &str, used_ids: &mut Vec<String>) -> PathBuf {
    let mut candidate = id.to_string();
    let mut suffix = 2usize;
    while used_ids.iter().any(|used| used == &candidate) {
        candidate = format!("{id}_{suffix}");
        suffix += 1;
    }
    used_ids.push(candidate.clone());
    base.join(format!("{candidate}.vand-instrument.json"))
}

fn write_instrument_bank_manifest(
    path: &Path,
    source_dir: &Path,
    records: &[ImportedInstrumentRecord],
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut text = String::new();
    text.push_str("{\n");
    text.push_str("  \"format\": \"vandaler-instrument-bank-v0\",\n");
    text.push_str(&format!(
        "  \"source_dir\": \"{}\",\n",
        json_escape(&source_dir.display().to_string())
    ));
    text.push_str("  \"license_review\": \"pending_curated_import\",\n");
    text.push_str(&format!("  \"instrument_count\": {},\n", records.len()));
    text.push_str("  \"instruments\": [\n");
    for (i, record) in records.iter().enumerate() {
        let comma = if i + 1 == records.len() { "" } else { "," };
        text.push_str("    {\n");
        text.push_str(&format!("      \"id\": \"{}\",\n", json_escape(&record.id)));
        text.push_str(&format!(
            "      \"name\": \"{}\",\n",
            json_escape(&record.name)
        ));
        text.push_str(&format!(
            "      \"chip\": \"{}\",\n",
            json_escape(&record.chip)
        ));
        text.push_str(&format!(
            "      \"category\": \"{}\",\n",
            json_escape(&record.category)
        ));
        text.push_str(&format!(
            "      \"source\": \"{}\",\n",
            json_escape(&record.source.display().to_string())
        ));
        text.push_str(&format!(
            "      \"file\": \"{}\"\n",
            json_escape(&record.output.display().to_string())
        ));
        text.push_str(&format!("    }}{}\n", comma));
    }
    text.push_str("  ]\n");
    text.push_str("}\n");
    std::fs::write(path, text)
}

fn import_instrument_dir(args: &Args) -> io::Result<()> {
    let input = args.rom.as_deref().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "missing instrument directory path",
        )
    })?;
    if !input.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "import-instrument-dir needs a directory",
        ));
    }

    let out_dir = args.out.clone().unwrap_or_else(|| {
        let name = input
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("bank");
        PathBuf::from("audio/instruments/imported").join(sanitize_id(name))
    });
    std::fs::create_dir_all(&out_dir)?;

    let mut files = Vec::new();
    collect_fui_files(input, &mut files)?;
    files.sort();

    let mut used_ids = Vec::new();
    let mut records = Vec::new();
    let mut skipped = 0usize;
    for file in files {
        match parse_furnace_instrument(&file) {
            Ok(instrument) => {
                let category = category_from_path(&file);
                let id = sanitize_id(&instrument.name);
                let output = unique_output_path(&out_dir, &id, &mut used_ids);
                write_furnace_instrument_json(&output, &instrument, &category)?;
                records.push(ImportedInstrumentRecord {
                    id: output
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or(&id)
                        .trim_end_matches(".vand-instrument")
                        .to_string(),
                    name: instrument.name,
                    chip: chip_name(instrument.instrument_type).to_string(),
                    category,
                    source: file,
                    output,
                });
            }
            Err(err) => {
                skipped += 1;
                eprintln!("skipped {}: {err}", file.display());
            }
        }
    }

    let manifest = out_dir.join("bank.vand-instruments.json");
    write_instrument_bank_manifest(&manifest, input, &records)?;
    println!(
        "imported {} instruments to {} | skipped {} | manifest {}",
        records.len(),
        out_dir.display(),
        skipped,
        manifest.display()
    );
    Ok(())
}

fn read_wav_mono(path: &Path) -> io::Result<WavData> {
    let mut data = Vec::new();
    File::open(path)?.read_to_end(&mut data)?;
    if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected RIFF/WAVE file",
        ));
    }

    let mut offset = 12usize;
    let mut channels = 0u16;
    let mut sample_rate = 0u32;
    let mut bits_per_sample = 0u16;
    let mut audio_format = 0u16;
    let mut pcm_data: Option<&[u8]> = None;

    while offset + 8 <= data.len() {
        let id = &data[offset..offset + 4];
        let size = read_u32_le(&data[offset + 4..offset + 8]) as usize;
        offset += 8;
        if offset + size > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "truncated WAV chunk",
            ));
        }

        match id {
            b"fmt " => {
                if size < 16 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "short fmt chunk",
                    ));
                }
                audio_format = read_u16_le(&data[offset..offset + 2]);
                channels = read_u16_le(&data[offset + 2..offset + 4]);
                sample_rate = read_u32_le(&data[offset + 4..offset + 8]);
                bits_per_sample = read_u16_le(&data[offset + 14..offset + 16]);
            }
            b"data" => pcm_data = Some(&data[offset..offset + size]),
            _ => {}
        }

        offset += size + (size & 1);
    }

    let pcm_data =
        pcm_data.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing data chunk"))?;
    if audio_format != 1 || bits_per_sample != 16 || channels == 0 || sample_rate == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "only 16-bit PCM WAV is supported for the first Rust analyser",
        ));
    }

    let channels_usize = channels as usize;
    let mut samples = Vec::with_capacity(pcm_data.len() / 2 / channels_usize);
    for frame in pcm_data.chunks_exact(2 * channels_usize) {
        let mut mixed = 0.0f32;
        for ch in 0..channels_usize {
            let at = ch * 2;
            let value = i16::from_le_bytes([frame[at], frame[at + 1]]) as f32 / 32768.0;
            mixed += value;
        }
        samples.push(mixed / channels as f32);
    }

    Ok(WavData {
        sample_rate,
        samples,
    })
}

fn decode_audio_with_ffmpeg(input: &Path, bundle_dir: &Path) -> io::Result<PathBuf> {
    std::fs::create_dir_all(bundle_dir)?;
    let decoded = bundle_dir.join("source_import.wav");
    let status = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg(input)
        .arg("-ac")
        .arg("1")
        .arg("-ar")
        .arg(DEFAULT_RATE.to_string())
        .arg("-sample_fmt")
        .arg("s16")
        .arg(&decoded)
        .status()
        .map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("ffmpeg is required for non-PCM imports: {err}"),
            )
        })?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "ffmpeg failed to import {}",
            input.display()
        )));
    }
    Ok(decoded)
}

fn import_audio(input: &Path, bundle_dir: &Path) -> io::Result<ImportedAudio> {
    let is_wav = input
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"));

    if is_wav {
        if let Ok(wav) = read_wav_mono(input) {
            return Ok(ImportedAudio {
                wav,
                analysis_input: input.to_path_buf(),
                decoded_wav: None,
                decoder: "direct_pcm_wav",
            });
        }
    }

    let decoded = decode_audio_with_ffmpeg(input, bundle_dir)?;
    let wav = read_wav_mono(&decoded)?;
    Ok(ImportedAudio {
        wav,
        analysis_input: decoded.clone(),
        decoded_wav: Some(decoded),
        decoder: "ffmpeg_pcm16_mono",
    })
}

fn write_import_metadata(path: &Path, source: &Path, imported: &ImportedAudio) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let duration = imported.wav.samples.len() as f32 / imported.wav.sample_rate as f32;
    let decoded = imported
        .decoded_wav
        .as_ref()
        .map(|path| format!("\"{}\"", json_escape(&path.display().to_string())))
        .unwrap_or_else(|| "null".to_string());
    let text = format!(
        concat!(
            "{{\n",
            "  \"format\": \"vandaler-audio-import-rust-v0\",\n",
            "  \"source\": \"{}\",\n",
            "  \"analysis_input\": \"{}\",\n",
            "  \"decoded_wav\": {},\n",
            "  \"decoder\": \"{}\",\n",
            "  \"sample_rate\": {},\n",
            "  \"sample_count\": {},\n",
            "  \"duration\": {:.6}\n",
            "}}\n"
        ),
        json_escape(&source.display().to_string()),
        json_escape(&imported.analysis_input.display().to_string()),
        decoded,
        imported.decoder,
        imported.wav.sample_rate,
        imported.wav.samples.len(),
        duration
    );
    std::fs::write(path, text)
}

fn midi_to_hz(midi: i32) -> f32 {
    440.0 * 2.0f32.powf((midi as f32 - 69.0) / 12.0)
}

fn hz_to_midi(hz: f32) -> Option<i32> {
    if hz <= 0.0 || !hz.is_finite() {
        return None;
    }
    Some((69.0 + 12.0 * (hz / 440.0).log2()).round() as i32)
}

fn note_name(hz: f32) -> String {
    if hz <= 0.0 {
        return "---".to_string();
    }
    let midi = hz_to_midi(hz).unwrap_or(0);
    note_name_from_midi(midi)
}

fn note_name_from_midi(midi: i32) -> String {
    let names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    format!("{}{}", names[midi.rem_euclid(12) as usize], midi / 12 - 1)
}

fn fold_midi_to_range(mut midi: i32, min_midi: i32, max_midi: i32) -> i32 {
    while midi > max_midi {
        midi -= 12;
    }
    while midi < min_midi {
        midi += 12;
    }
    midi.clamp(min_midi, max_midi)
}

fn pitch_class_name(root: i32) -> &'static str {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    NAMES[root.rem_euclid(12) as usize]
}

fn mode_intervals(mode: &str) -> &'static [i32] {
    if mode == "minor" {
        &[0, 2, 3, 5, 7, 8, 10]
    } else {
        &[0, 2, 4, 5, 7, 9, 11]
    }
}

fn scale_contains(root: i32, mode: &str, midi: i32) -> bool {
    let pc = (midi - root).rem_euclid(12);
    mode_intervals(mode).contains(&pc)
}

fn snap_midi_to_scale(midi: i32, root: i32, mode: &str, min_midi: i32, max_midi: i32) -> i32 {
    if scale_contains(root, mode, midi) {
        return midi;
    }
    let mut best = midi;
    let mut best_distance = i32::MAX;
    for candidate in (midi - 2)..=(midi + 2) {
        if candidate < min_midi || candidate > max_midi || !scale_contains(root, mode, candidate) {
            continue;
        }
        let distance = (candidate - midi).abs();
        if distance < best_distance {
            best = candidate;
            best_distance = distance;
        }
    }
    best
}

fn detect_key(frames: &[AnalysisFrame]) -> (i32, &'static str, f32) {
    const MAJOR_PROFILE: [f32; 12] = [
        6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
    ];
    const MINOR_PROFILE: [f32; 12] = [
        6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
    ];

    let mut histogram = [0.0f32; 12];
    for frame in frames {
        if let Some(midi) = hz_to_midi(frame.bass_hz) {
            histogram[midi.rem_euclid(12) as usize] += frame.bass_amp * 1.6;
        }
        if let Some(midi) = hz_to_midi(frame.lead_hz) {
            histogram[midi.rem_euclid(12) as usize] += frame.lead_amp;
        }
    }

    let total: f32 = histogram.iter().sum();
    if total <= 0.0 {
        return (0, "minor", 0.0);
    }

    let mut best = (0, "minor", f32::MIN);
    let mut second = f32::MIN;
    for root in 0..12 {
        for (mode, profile) in [("major", MAJOR_PROFILE), ("minor", MINOR_PROFILE)] {
            let mut score = 0.0;
            for pc in 0..12 {
                score += histogram[(root + pc) % 12] * profile[pc];
            }
            if score > best.2 {
                second = best.2;
                best = (root as i32, mode, score);
            } else if score > second {
                second = score;
            }
        }
    }

    let confidence = ((best.2 - second) / best.2.max(1.0)).clamp(0.0, 1.0);
    (best.0, best.1, confidence)
}

fn detect_bpm(frames: &[AnalysisFrame]) -> f32 {
    let mut onsets = Vec::new();
    let mut last = 0usize;
    for frame in frames {
        let strong = frame.transient > 0.55 && frame.rms > 0.025;
        if strong && frame.index.saturating_sub(last) > 6 {
            onsets.push(frame.index);
            last = frame.index;
        }
    }
    if onsets.len() < 4 {
        return 120.0;
    }

    let mut bins = [0.0f32; 121];
    for pair in onsets.windows(2) {
        let mut bpm =
            60.0 * 44_100.0 / (pair[1].saturating_sub(pair[0]).max(1) as f32 * ANALYSIS_HOP as f32);
        while bpm < 80.0 {
            bpm *= 2.0;
        }
        while bpm > 200.0 {
            bpm *= 0.5;
        }
        if (80.0..=200.0).contains(&bpm) {
            bins[(bpm.round() as usize).saturating_sub(80).min(120)] += 1.0;
        }
    }

    bins.iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(index, _)| 80.0 + index as f32)
        .unwrap_or(120.0)
}

fn analyse_musical_context(frames: &[AnalysisFrame], sample_rate: u32) -> MusicalContext {
    let (key_root, mode, key_confidence) = detect_key(frames);
    let bpm = detect_bpm(frames);
    let frames_per_beat = (sample_rate as f32 * 60.0) / (bpm * ANALYSIS_HOP as f32);
    let grid_frames = (frames_per_beat / 4.0).round().max(2.0) as usize;
    let bar_frames = (frames_per_beat * 4.0).round().max(16.0) as usize;
    let min_loop_frames = bar_frames * 4;
    let mut loop_end_frame = frames.len() / bar_frames * bar_frames;
    if loop_end_frame < min_loop_frames || frames.len().saturating_sub(loop_end_frame) > bar_frames
    {
        loop_end_frame = frames.len();
    }
    MusicalContext {
        key_root,
        mode,
        key_confidence,
        bpm,
        frames_per_beat,
        grid_frames,
        loop_start_frame: 0,
        loop_end_frame: loop_end_frame.max(1),
    }
}

fn snap_frame_to_grid(frame: usize, grid_frames: usize, max_shift: usize) -> usize {
    if grid_frames < 2 {
        return frame;
    }
    let lower = frame / grid_frames * grid_frames;
    let upper = lower + grid_frames;
    let nearest = if frame - lower <= upper - frame {
        lower
    } else {
        upper
    };
    if frame.abs_diff(nearest) <= max_shift {
        nearest
    } else {
        frame
    }
}

fn goertzel_power(frame: &[f32], sample_rate: u32, hz: f32) -> f32 {
    let omega = 2.0 * std::f32::consts::PI * hz / sample_rate as f32;
    let coeff = 2.0 * omega.cos();
    let mut s_prev = 0.0;
    let mut s_prev2 = 0.0;

    for (i, sample) in frame.iter().enumerate() {
        let window =
            0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / (frame.len() - 1) as f32).cos();
        let s = sample * window + coeff * s_prev - s_prev2;
        s_prev2 = s_prev;
        s_prev = s;
    }

    (s_prev2 * s_prev2 + s_prev * s_prev - coeff * s_prev * s_prev2).sqrt() / frame.len() as f32
}

fn best_note_power(frame: &[f32], sample_rate: u32, lo_midi: i32, hi_midi: i32) -> (f32, f32) {
    let mut best_hz = 0.0;
    let mut best_power = 0.0;
    for midi in lo_midi..=hi_midi {
        let hz = midi_to_hz(midi);
        let power = goertzel_power(frame, sample_rate, hz);
        if power > best_power {
            best_power = power;
            best_hz = hz;
        }
    }
    (best_hz, best_power)
}

fn classify_frame(
    rms: f32,
    transient: f32,
    bass_amp: f32,
    lead_amp: f32,
    noise_amp: f32,
) -> &'static str {
    if rms < 0.006 {
        "silence"
    } else if transient > 0.50 && noise_amp > 0.22 {
        "drum"
    } else if bass_amp > lead_amp * 0.90 && bass_amp > 0.018 {
        "bass"
    } else if lead_amp > 0.018 {
        "tonal"
    } else if noise_amp > 0.18 {
        "noise"
    } else if transient > 0.24 {
        "sample"
    } else {
        "ambience"
    }
}

fn analyse_samples(wav: &WavData) -> Vec<AnalysisFrame> {
    if wav.samples.is_empty() {
        return Vec::new();
    }

    let mut padded = wav.samples.clone();
    if padded.len() < ANALYSIS_FRAME {
        padded.resize(ANALYSIS_FRAME, 0.0);
    }

    let mut raw = Vec::new();
    let mut max_rms = 1e-6f32;
    let mut max_tonal = 1e-6f32;
    let mut prev_rms = 0.0f32;
    let mut index = 0usize;
    let mut start = 0usize;

    while start + ANALYSIS_FRAME <= padded.len() {
        let frame = &padded[start..start + ANALYSIS_FRAME];
        let rms = (frame.iter().map(|s| s * s).sum::<f32>() / frame.len() as f32).sqrt();
        let transient = ((rms - prev_rms).max(0.0) / (prev_rms + 0.012)).min(1.0);
        let (bass_hz, bass_power) = best_note_power(frame, wav.sample_rate, 29, 57);
        let (lead_hz, lead_power) = best_note_power(frame, wav.sample_rate, 58, 86);
        let high_power = [2200.0, 3200.0, 4600.0, 6200.0, 7800.0]
            .iter()
            .map(|hz| goertzel_power(frame, wav.sample_rate, *hz))
            .sum::<f32>()
            / 5.0;

        max_rms = max_rms.max(rms);
        max_tonal = max_tonal.max(bass_power).max(lead_power).max(high_power);
        raw.push((
            index, start, rms, transient, bass_hz, bass_power, lead_hz, lead_power, high_power,
        ));
        prev_rms = rms;
        index += 1;
        start += ANALYSIS_HOP;
    }

    raw.into_iter()
        .map(
            |(
                index,
                start,
                rms,
                transient,
                bass_hz,
                bass_power,
                lead_hz,
                lead_power,
                high_power,
            )| {
                let bass_amp = (bass_power / max_tonal).min(1.0);
                let lead_amp = (lead_power / max_tonal).min(1.0);
                let noise_amp = ((high_power / max_tonal) * (rms / max_rms).sqrt()).min(1.0);
                let bass_hz = if bass_amp > 0.07 { bass_hz } else { 0.0 };
                let lead_hz = if lead_amp > 0.07 { lead_hz } else { 0.0 };
                let class_name =
                    classify_frame(rms / max_rms, transient, bass_amp, lead_amp, noise_amp);
                AnalysisFrame {
                    index,
                    time: start as f32 / wav.sample_rate as f32,
                    rms: rms / max_rms,
                    transient,
                    bass_hz,
                    bass_amp,
                    lead_hz,
                    lead_amp,
                    noise_amp,
                    class_name,
                }
            },
        )
        .collect()
}

fn default_analysis_path(input: &Path) -> PathBuf {
    analysis_path_in_dir(input, Path::new("audio/converted"))
}

fn analysis_path_in_dir(input: &Path, output_dir: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    output_dir
        .join(format!("{stem}.vand-audio"))
        .join("arrangement.vand-audio.json")
}

fn class_id(name: &str) -> u8 {
    match name {
        "ambience" => 1,
        "noise" => 2,
        "sample" => 3,
        "drum" => 4,
        "bass" => 5,
        "tonal" => 6,
        _ => 0,
    }
}

fn level15(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 15.0).round() as u8
}

fn ym2612_pitch(hz: f32) -> (u16, u8) {
    const YM2612_CLOCK: f32 = 7_670_454.0;

    if hz <= 0.0 {
        return (0, 0);
    }

    let base = YM2612_CLOCK / 144.0;
    let mut best_fnum = 0x400u16;
    let mut best_block = 4u8;
    let mut best_score = f32::MAX;

    for block in 1..=7u8 {
        let divider = 2.0f32.powi(block as i32 - 1);
        let fnum = (hz * (1u32 << 20) as f32 / base / divider).round();
        if (256.0..=2047.0).contains(&fnum) {
            let score = (fnum - 1024.0).abs();
            if score < best_score {
                best_score = score;
                best_fnum = fnum as u16;
                best_block = block;
            }
        }
    }

    if best_score == f32::MAX {
        let fnum = (hz * (1u32 << 20) as f32 / base / 8.0)
            .round()
            .clamp(256.0, 2047.0);
        best_fnum = fnum as u16;
        best_block = 4;
    }

    (best_fnum, best_block)
}

fn equivalent_runtime(a: &RuntimeEvent, b: &RuntimeEvent) -> bool {
    a.fm0_fnum == b.fm0_fnum
        && a.fm0_block == b.fm0_block
        && a.fm0_level == b.fm0_level
        && a.fm1_fnum == b.fm1_fnum
        && a.fm1_block == b.fm1_block
        && a.fm1_level == b.fm1_level
        && a.psg_noise_level == b.psg_noise_level
        && a.kind == b.kind
        && a.dac_chunk == b.dac_chunk
        && a.dac_level == b.dac_level
}

fn analysis_frame_to_tick(frame: usize, sample_rate: u32) -> u32 {
    let seconds = frame as f32 * ANALYSIS_HOP as f32 / sample_rate as f32;
    (seconds * 60.0).round().max(0.0) as u32
}

fn note_active_at_tick(note: &NoteEvent, tick: u32, sample_rate: u32) -> bool {
    let start = analysis_frame_to_tick(note.start_frame, sample_rate);
    let end = analysis_frame_to_tick(note.start_frame + note.frames.max(1), sample_rate);
    tick >= start && tick < end.max(start + 1)
}

fn note_to_runtime_pitch(note: Option<&NoteEvent>) -> (u16, u8, u8) {
    if let Some(note) = note {
        let (fnum, block) = ym2612_pitch(note.hz);
        let level = if fnum == 0 {
            0
        } else {
            level15(note.velocity * 0.95)
        };
        return (fnum, block, level);
    }
    (0, 0, 0)
}

fn drum_at_tick<'a>(drums: &'a [DrumEvent], tick: u32, sample_rate: u32) -> Option<&'a DrumEvent> {
    drums
        .iter()
        .filter(|drum| analysis_frame_to_tick(drum.start_frame, sample_rate) == tick)
        .max_by(|a, b| {
            let a_score = a.velocity + if a.dac_chunk.is_some() { 1.0 } else { 0.0 };
            let b_score = b.velocity + if b.dac_chunk.is_some() { 1.0 } else { 0.0 };
            a_score
                .partial_cmp(&b_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn push_note_change_points(points: &mut Vec<u32>, note: &NoteEvent, sample_rate: u32) {
    let start = analysis_frame_to_tick(note.start_frame, sample_rate);
    let end = analysis_frame_to_tick(note.start_frame + note.frames.max(1), sample_rate);
    points.push(start);
    points.push(end.max(start + 1));
}

fn build_runtime_events_from_render_plan(
    sample_rate: u32,
    loop_end_frame: usize,
    bass_notes: &[NoteEvent],
    lead_notes: &[NoteEvent],
    drum_events: &[DrumEvent],
) -> Vec<RuntimeEvent> {
    let loop_end_tick = analysis_frame_to_tick(loop_end_frame.max(1), sample_rate).max(1);
    let mut points = vec![0, loop_end_tick];

    for note in bass_notes {
        push_note_change_points(&mut points, note, sample_rate);
    }
    for note in lead_notes {
        push_note_change_points(&mut points, note, sample_rate);
    }
    for drum in drum_events {
        let tick = analysis_frame_to_tick(drum.start_frame, sample_rate);
        if tick < loop_end_tick {
            points.push(tick);
            points.push((tick + 1).min(loop_end_tick));
        }
    }

    points.sort_unstable();
    points.dedup();

    let mut out = Vec::new();
    for pair in points.windows(2) {
        let tick = pair[0];
        let next_tick = pair[1];
        if tick >= loop_end_tick || next_tick <= tick {
            continue;
        }

        let bass = bass_notes
            .iter()
            .find(|note| note_active_at_tick(note, tick, sample_rate));
        let lead = lead_notes
            .iter()
            .find(|note| note_active_at_tick(note, tick, sample_rate));
        let drum = drum_at_tick(drum_events, tick, sample_rate);
        let (fm0_fnum, fm0_block, fm0_level) = note_to_runtime_pitch(bass);
        let (fm1_fnum, fm1_block, fm1_level) = note_to_runtime_pitch(lead);
        let psg_noise_level = drum
            .map(|drum| level15(drum.noise_amp.max(drum.velocity * 0.7)))
            .unwrap_or(0);
        let dac_chunk = drum.and_then(|drum| drum.dac_chunk).unwrap_or(255).min(254) as u8;
        let dac_level = drum.map(|drum| level15(drum.velocity)).unwrap_or(0);
        let kind = if drum.is_some() {
            class_id("drum")
        } else if bass.is_some() {
            class_id("bass")
        } else if lead.is_some() {
            class_id("tonal")
        } else {
            class_id("silence")
        };

        let next = RuntimeEvent {
            frames: (next_tick - tick).clamp(1, u16::MAX as u32) as u16,
            fm0_fnum,
            fm0_block,
            fm0_level,
            fm1_fnum,
            fm1_block,
            fm1_level,
            psg_noise_level,
            kind,
            dac_chunk,
            dac_level,
        };

        if let Some(last) = out.last_mut() {
            if equivalent_runtime(last, &next) {
                last.frames = last.frames.saturating_add(next.frames);
                continue;
            }
        }
        out.push(next);
    }

    out
}

fn build_render_plan_runtime_events(
    frames: &[AnalysisFrame],
    sample_rate: u32,
    chunks: &[DacChunk],
) -> Vec<RuntimeEvent> {
    let (context, bass_notes, lead_notes, drum_events) =
        build_note_preview_data(frames, sample_rate, chunks);
    build_runtime_events_from_render_plan(
        sample_rate,
        context.loop_end_frame,
        &bass_notes,
        &lead_notes,
        &drum_events,
    )
}

fn write_runtime_binary(path: &Path, sample_rate: u32, events: &[RuntimeEvent]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    file.write_all(b"VADB")?;
    file.write_all(&3u16.to_le_bytes())?;
    file.write_all(&(events.len() as u16).to_le_bytes())?;
    file.write_all(&(ANALYSIS_HOP as u16).to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;

    for event in events {
        file.write_all(&event.frames.to_le_bytes())?;
        file.write_all(&event.fm0_fnum.to_le_bytes())?;
        file.write_all(&[event.fm0_block, event.fm0_level])?;
        file.write_all(&event.fm1_fnum.to_le_bytes())?;
        file.write_all(&[event.fm1_block, event.fm1_level])?;
        file.write_all(&[event.psg_noise_level, event.kind])?;
        file.write_all(&[event.dac_chunk, event.dac_level])?;
    }

    Ok(())
}

fn write_sgdk_audio(
    header: &Path,
    source: &Path,
    events: &[RuntimeEvent],
    chunks: &[DacChunk],
) -> io::Result<()> {
    if let Some(parent) = header.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = source.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let header_text = concat!(
        "#ifndef GENERATED_AUDIO_H\n",
        "#define GENERATED_AUDIO_H\n\n",
        "#include \"vand_audio.h\"\n\n",
        "extern const VandAudioEvent generatedAudioEvents[];\n",
        "extern const u16 generatedAudioEventCount;\n\n",
        "extern const u8 * const generatedAudioDacSamples[];\n",
        "extern const u16 generatedAudioDacLengths[];\n",
        "extern const u16 generatedAudioDacRates[];\n",
        "extern const u16 generatedAudioDacCount;\n\n",
        "#endif\n"
    );
    std::fs::write(header, header_text)?;

    let include_name = header
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("generated_audio.h");
    let mut c = String::new();
    c.push_str(&format!("#include \"{}\"\n\n", include_name));
    for chunk in chunks {
        c.push_str(&format!(
            "static const u8 generatedAudioDacChunk{:02}[] =\n{{\n",
            chunk.id
        ));
        for row in chunk.samples.chunks(16) {
            c.push_str("    ");
            for sample in row {
                c.push_str(&format!("{}, ", sample));
            }
            c.push('\n');
        }
        c.push_str("};\n\n");
    }
    c.push_str("const u8 * const generatedAudioDacSamples[] =\n{\n");
    if chunks.is_empty() {
        c.push_str("    NULL,\n");
    } else {
        for chunk in chunks {
            c.push_str(&format!("    generatedAudioDacChunk{:02},\n", chunk.id));
        }
    }
    c.push_str("};\n\n");
    c.push_str("const u16 generatedAudioDacLengths[] =\n{\n");
    if chunks.is_empty() {
        c.push_str("    0,\n");
    } else {
        for chunk in chunks {
            c.push_str(&format!("    {},\n", chunk.samples.len()));
        }
    }
    c.push_str("};\n\n");
    c.push_str("const u16 generatedAudioDacRates[] =\n{\n");
    if chunks.is_empty() {
        c.push_str("    0,\n");
    } else {
        for chunk in chunks {
            c.push_str(&format!("    {},\n", chunk.sample_rate));
        }
    }
    c.push_str("};\n\n");
    c.push_str(&format!(
        "const u16 generatedAudioDacCount = {};\n\n",
        chunks.len()
    ));
    c.push_str("const VandAudioEvent generatedAudioEvents[] =\n{\n");
    for event in events {
        c.push_str(&format!(
            "    {{{}, 0x{:03X}, {}, {}, 0x{:03X}, {}, {}, {}, {}, {}, {}}},\n",
            event.frames,
            event.fm0_fnum,
            event.fm0_block,
            event.fm0_level,
            event.fm1_fnum,
            event.fm1_block,
            event.fm1_level,
            event.psg_noise_level,
            event.kind,
            event.dac_chunk,
            event.dac_level
        ));
    }
    c.push_str("};\n\n");
    c.push_str(
        "const u16 generatedAudioEventCount = sizeof(generatedAudioEvents) / sizeof(generatedAudioEvents[0]);\n",
    );
    std::fs::write(source, c)
}

fn write_frame_track_json(
    path: &Path,
    input: &Path,
    track_name: &str,
    frames: &[AnalysisFrame],
    value_for_frame: impl Fn(&AnalysisFrame) -> (f32, f32),
    instrument_for_frame: impl Fn(&AnalysisFrame, f32, f32) -> &'static str,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"format\": \"vandaler-vand-audio-track-rust-v0\",\n");
    out.push_str(&format!("  \"track\": \"{}\",\n", track_name));
    out.push_str(&format!(
        "  \"source\": \"{}\",\n",
        json_escape(&input.display().to_string())
    ));
    out.push_str(&format!(
        "  \"instrument_bank\": \"{}\",\n",
        DEFAULT_INSTRUMENT_BANK
    ));
    out.push_str(&format!("  \"hop\": {},\n", ANALYSIS_HOP));
    out.push_str("  \"frames\": [\n");
    for (i, frame) in frames.iter().enumerate() {
        let comma = if i + 1 == frames.len() { "" } else { "," };
        let (hz, amp) = value_for_frame(frame);
        let instrument_id = instrument_for_frame(frame, hz, amp);
        out.push_str(&format!(
            "    {{\"index\": {}, \"t\": {:.6}, \"hz\": {:.3}, \"amp\": {:.5}, \"instrument_id\": \"{}\"}}{}\n",
            frame.index,
            frame.time,
            hz,
            amp,
            instrument_id,
            comma
        ));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    std::fs::write(path, out)
}

fn write_split_tracks(bundle_dir: &Path, input: &Path, frames: &[AnalysisFrame]) -> io::Result<()> {
    write_frame_track_json(
        &bundle_dir.join("fm_bass.json"),
        input,
        "fm_bass",
        frames,
        |frame| (frame.bass_hz, frame.bass_amp),
        |_frame, hz, amp| {
            if hz > 0.0 && amp > 0.0 {
                DEFAULT_BASS_INSTRUMENT
            } else {
                ""
            }
        },
    )?;
    write_frame_track_json(
        &bundle_dir.join("fm_lead.json"),
        input,
        "fm_lead",
        frames,
        |frame| (frame.lead_hz, frame.lead_amp),
        |_frame, hz, amp| {
            if hz > 0.0 && amp > 0.0 {
                DEFAULT_LEAD_INSTRUMENT
            } else {
                ""
            }
        },
    )?;
    write_frame_track_json(
        &bundle_dir.join("psg_noise.json"),
        input,
        "psg_noise",
        frames,
        |frame| (0.0, frame.noise_amp.max(frame.transient * 0.65)),
        |_frame, _hz, amp| {
            if amp > 0.0 {
                DEFAULT_NOISE_INSTRUMENT
            } else {
                ""
            }
        },
    )
}

fn is_dac_candidate(frame: &AnalysisFrame) -> bool {
    matches!(frame.class_name, "drum" | "sample")
        || (frame.transient > 0.42 && frame.rms > 0.035)
        || (frame.noise_amp > 0.42 && frame.rms > 0.060)
}

fn resample_dac_chunk(slice: &[f32], source_rate: u32, target_rate: u32, peak: f32) -> Vec<u8> {
    if slice.is_empty() || source_rate == 0 || target_rate == 0 {
        return Vec::new();
    }

    let out_len = ((slice.len() as u64 * target_rate as u64 + (source_rate / 2) as u64)
        / source_rate as u64)
        .max(1) as usize;
    let fade_len = (target_rate as usize / 250).clamp(4, 48).min(out_len / 2);
    let mut out = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let src_pos = i as f32 * source_rate as f32 / target_rate as f32;
        let src_idx = src_pos.floor() as usize;
        let frac = src_pos - src_idx as f32;
        let a = slice[src_idx.min(slice.len() - 1)];
        let b = slice[(src_idx + 1).min(slice.len() - 1)];
        let mut normalized = ((a * (1.0 - frac) + b * frac) / peak * 0.88).clamp(-1.0, 1.0);

        if fade_len > 0 {
            let fade_in = (i + 1).min(fade_len) as f32 / fade_len as f32;
            let fade_out = (out_len - i).min(fade_len) as f32 / fade_len as f32;
            normalized *= fade_in.min(fade_out).min(1.0);
        }

        out.push((normalized * 127.0 + 128.0).round().clamp(0.0, 255.0) as u8);
    }

    out
}

fn extract_dac_chunks(
    bundle_dir: &Path,
    wav: &WavData,
    frames: &[AnalysisFrame],
) -> io::Result<Vec<DacChunk>> {
    let chunk_dir = bundle_dir.join("dac_chunks");
    std::fs::create_dir_all(&chunk_dir)?;

    let mut chunks = Vec::new();
    let mut index = 0usize;
    while index < frames.len() && chunks.len() < 32 {
        if !is_dac_candidate(&frames[index]) {
            index += 1;
            continue;
        }

        let start_frame = index;
        let mut end_frame = index;
        while end_frame + 1 < frames.len() && is_dac_candidate(&frames[end_frame + 1]) {
            end_frame += 1;
        }

        let start_sample = start_frame.saturating_mul(ANALYSIS_HOP);
        let mut end_sample =
            ((end_frame + 1) * ANALYSIS_HOP + ANALYSIS_FRAME).min(wav.samples.len());
        let max_len = (wav.sample_rate as usize / 4).max(1);
        end_sample = end_sample.min(start_sample.saturating_add(max_len));

        if end_sample > start_sample + 32 {
            let slice = &wav.samples[start_sample..end_sample];
            let peak = slice
                .iter()
                .map(|sample| sample.abs())
                .fold(0.0f32, f32::max)
                .max(1e-6);
            let rms = (slice.iter().map(|sample| sample * sample).sum::<f32>()
                / slice.len() as f32)
                .sqrt();
            let file_name = format!("chunk_{:02}.u8", chunks.len());
            let path = chunk_dir.join(&file_name);
            let pcm = resample_dac_chunk(slice, wav.sample_rate, DAC_SAMPLE_RATE, peak);
            std::fs::write(&path, &pcm)?;
            chunks.push(DacChunk {
                id: chunks.len(),
                start_frame,
                end_frame,
                start_sample,
                source_sample_count: slice.len(),
                sample_count: pcm.len(),
                sample_rate: DAC_SAMPLE_RATE,
                peak,
                rms,
                path: format!("dac_chunks/{file_name}"),
                samples: pcm,
            });
        }

        index = end_frame + 1;
    }

    Ok(chunks)
}

fn write_dac_chunks_json(
    path: &Path,
    input: &Path,
    sample_rate: u32,
    chunks: &[DacChunk],
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"format\": \"vandaler-dac-chunks-rust-v0\",\n");
    out.push_str(&format!(
        "  \"source\": \"{}\",\n",
        json_escape(&input.display().to_string())
    ));
    out.push_str(&format!("  \"sample_rate\": {},\n", sample_rate));
    out.push_str(&format!(
        "  \"playback_sample_rate\": {},\n",
        DAC_SAMPLE_RATE
    ));
    out.push_str("  \"encoding\": \"unsigned_u8_pcm_center_128\",\n");
    out.push_str("  \"chunks\": [\n");
    for (i, chunk) in chunks.iter().enumerate() {
        let comma = if i + 1 == chunks.len() { "" } else { "," };
        out.push_str(&format!(
            concat!(
                "    {{\"id\": {}, \"path\": \"{}\", \"start_frame\": {}, ",
                "\"end_frame\": {}, \"source_start_sample\": {}, ",
                "\"source_sample_count\": {}, \"sample_rate\": {}, \"sample_count\": {}, ",
                "\"peak\": {:.6}, \"rms\": {:.6}}}{}\n"
            ),
            chunk.id,
            json_escape(&chunk.path),
            chunk.start_frame,
            chunk.end_frame,
            chunk.start_sample,
            chunk.source_sample_count,
            chunk.sample_rate,
            chunk.sample_count,
            chunk.peak,
            chunk.rms,
            comma
        ));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    std::fs::write(path, out)
}

fn write_dac_preview_wav(bundle_dir: &Path, chunks: &[DacChunk]) -> io::Result<PathBuf> {
    let path = bundle_dir.join("dac_preview.wav");
    let mut stereo = Vec::new();
    let gap_samples = (DAC_SAMPLE_RATE / 12) as usize;

    for chunk in chunks {
        for sample in &chunk.samples {
            let centered = i16::from(*sample) - 128;
            let value = centered.saturating_mul(256);
            stereo.push(value);
            stereo.push(value);
        }
        stereo.extend(std::iter::repeat_n(0, gap_samples * 2));
    }

    if stereo.is_empty() {
        stereo.extend(std::iter::repeat_n(0, (DAC_SAMPLE_RATE / 4) as usize * 2));
    }

    write_wav_i16_stereo(&path, DAC_SAMPLE_RATE, &stereo)?;
    Ok(path)
}

fn hz_from_ym2612_pitch(fnum: u16, block: u8) -> f32 {
    if fnum == 0 || block == 0 {
        return 0.0;
    }

    let base = 7_670_454.0 / 144.0;
    let divider = 2.0f32.powi(block as i32 - 1);
    (fnum as f32 * base * divider) / (1u32 << 20) as f32
}

fn write_runtime_preview_wav(
    bundle_dir: &Path,
    events: &[RuntimeEvent],
    chunks: &[DacChunk],
) -> io::Result<PathBuf> {
    const PREVIEW_RATE: u32 = 44_100;
    let path = bundle_dir.join("runtime_preview.wav");
    let mut stereo = Vec::new();
    let mut bass_phase = 0.0f32;
    let mut lead_phase = 0.0f32;
    let mut noise_state = 0x1234ABCDu32;
    let mut dac_active: Option<(&[u8], usize, u32)> = None;
    let mut dac_rate_accum = 0u32;

    for event in events {
        if event.dac_chunk != 255 && event.dac_level > 0 {
            dac_active = chunks
                .iter()
                .find(|chunk| chunk.id == event.dac_chunk as usize)
                .map(|chunk| (chunk.samples.as_slice(), 0usize, chunk.sample_rate.max(1)));
            dac_rate_accum = 0;
        }

        let samples = ((event.frames as u64 * PREVIEW_RATE as u64 + 30) / 60).max(1) as usize;
        let bass_hz = hz_from_ym2612_pitch(event.fm0_fnum, event.fm0_block);
        let lead_hz = hz_from_ym2612_pitch(event.fm1_fnum, event.fm1_block);
        let bass_gain = event.fm0_level as f32 / 15.0 * 0.35;
        let lead_gain = event.fm1_level as f32 / 15.0 * 0.28;
        let noise_gain = event.psg_noise_level as f32 / 15.0 * 0.18;
        let dac_gain = event.dac_level as f32 / 15.0 * 0.50;

        for _ in 0..samples {
            let mut sample = 0.0f32;

            if bass_hz > 0.0 && event.fm0_level > 0 {
                sample += bass_phase.sin() * bass_gain;
                bass_phase += std::f32::consts::TAU * bass_hz / PREVIEW_RATE as f32;
            }
            if lead_hz > 0.0 && event.fm1_level > 0 {
                sample += lead_phase.sin() * lead_gain;
                lead_phase += std::f32::consts::TAU * lead_hz / PREVIEW_RATE as f32;
            }
            if event.psg_noise_level > 0 {
                noise_state = noise_state
                    .wrapping_mul(1_664_525)
                    .wrapping_add(1_013_904_223);
                let noise = ((noise_state >> 16) as f32 / 32768.0) - 1.0;
                sample += noise * noise_gain;
            }

            if let Some((bytes, pos, rate)) = dac_active.as_mut() {
                if !bytes.is_empty() {
                    let centered = bytes[*pos] as f32 - 128.0;
                    sample += centered / 128.0 * dac_gain;
                    dac_rate_accum += *rate;
                    while dac_rate_accum >= PREVIEW_RATE {
                        dac_rate_accum -= PREVIEW_RATE;
                        *pos += 1;
                        if *pos >= bytes.len() {
                            dac_active = None;
                            break;
                        }
                    }
                }
            }

            let value = (sample.clamp(-1.0, 1.0) * 24_000.0).round() as i16;
            stereo.push(value);
            stereo.push(value);
        }
    }

    if stereo.is_empty() {
        stereo.extend(std::iter::repeat_n(0, (PREVIEW_RATE / 4) as usize * 2));
    }

    write_wav_i16_stereo(&path, PREVIEW_RATE, &stereo)?;
    Ok(path)
}

fn write_analysis_json(
    path: &Path,
    input: &Path,
    wav: &WavData,
    frames: &[AnalysisFrame],
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let duration = wav.samples.len() as f32 / wav.sample_rate as f32;
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"format\": \"vandaler-vand-audio-rust-v0\",\n");
    out.push_str(&format!(
        "  \"source\": \"{}\",\n",
        json_escape(&input.display().to_string())
    ));
    out.push_str(&format!("  \"sample_rate\": {},\n", wav.sample_rate));
    out.push_str(&format!("  \"duration\": {:.6},\n", duration));
    out.push_str(&format!("  \"frame_size\": {},\n", ANALYSIS_FRAME));
    out.push_str(&format!("  \"hop\": {},\n", ANALYSIS_HOP));
    out.push_str(&format!(
        "  \"instrument_bank\": \"{}\",\n",
        DEFAULT_INSTRUMENT_BANK
    ));
    out.push_str("  \"pipeline\": [\"audio_import_pcm16\", \"goertzel_pitch_tracking\", \"transient_detection\", \"frame_classification\", \"split_track_export\", \"dac_candidate_extraction\"],\n");
    out.push_str("  \"events\": [\n");
    for (i, frame) in frames.iter().enumerate() {
        let comma = if i + 1 == frames.len() { "" } else { "," };
        out.push_str(&format!(
            concat!(
                "    {{\"index\": {}, \"t\": {:.6}, \"class\": \"{}\", ",
                "\"rms\": {:.5}, \"transient\": {:.5}, ",
                "\"bass_hz\": {:.3}, \"bass_note\": \"{}\", \"bass_amp\": {:.5}, \"bass_instrument_id\": \"{}\", ",
                "\"lead_hz\": {:.3}, \"lead_note\": \"{}\", \"lead_amp\": {:.5}, \"lead_instrument_id\": \"{}\", ",
                "\"noise_amp\": {:.5}, \"noise_instrument_id\": \"{}\"}}{}\n"
            ),
            frame.index,
            frame.time,
            frame.class_name,
            frame.rms,
            frame.transient,
            frame.bass_hz,
            note_name(frame.bass_hz),
            frame.bass_amp,
            if frame.bass_hz > 0.0 && frame.bass_amp > 0.0 {
                DEFAULT_BASS_INSTRUMENT
            } else {
                ""
            },
            frame.lead_hz,
            note_name(frame.lead_hz),
            frame.lead_amp,
            if frame.lead_hz > 0.0 && frame.lead_amp > 0.0 {
                DEFAULT_LEAD_INSTRUMENT
            } else {
                ""
            },
            frame.noise_amp,
            if frame.noise_amp.max(frame.transient * 0.65) > 0.0 {
                DEFAULT_NOISE_INSTRUMENT
            } else {
                ""
            },
            comma,
        ));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    std::fs::write(path, out)
}

fn stable_midi_candidates(
    frames: &[AnalysisFrame],
    track: &'static str,
    min_midi: i32,
    max_midi: i32,
    context: &MusicalContext,
) -> Vec<Option<(i32, f32)>> {
    let mut raw = Vec::with_capacity(frames.len());
    let threshold = if track == "bass" { 0.06 } else { 0.05 };
    for frame in frames {
        let (hz, amp) = if track == "bass" {
            (frame.bass_hz, frame.bass_amp)
        } else {
            (frame.lead_hz, frame.lead_amp)
        };
        let midi = hz_to_midi(hz).map(|midi| {
            let folded = fold_midi_to_range(midi, min_midi, max_midi);
            if amp < threshold * 2.25 || context.key_confidence > 0.08 {
                snap_midi_to_scale(folded, context.key_root, context.mode, min_midi, max_midi)
            } else {
                folded
            }
        });
        raw.push(midi.filter(|_| amp >= threshold).map(|midi| (midi, amp)));
    }

    let mut stable = Vec::with_capacity(raw.len());
    let mut current: Option<i32> = None;
    let mut pending: Option<i32> = None;
    let mut pending_count = 0usize;

    for candidate in raw {
        let Some((midi, amp)) = candidate else {
            pending = None;
            pending_count = 0;
            current = None;
            stable.push(None);
            continue;
        };

        if current.is_none() {
            current = Some(midi);
            pending = None;
            pending_count = 0;
            stable.push(Some((midi, amp)));
            continue;
        }

        let locked = current.unwrap_or(midi);
        if (locked - midi).abs() <= 1 {
            pending = None;
            pending_count = 0;
            stable.push(Some((locked, amp)));
            continue;
        }

        if pending == Some(midi) {
            pending_count += 1;
        } else {
            pending = Some(midi);
            pending_count = 1;
        }

        let confirm_frames = if track == "bass" { 4 } else { 3 };
        if pending_count >= confirm_frames {
            current = Some(midi);
            stable.push(Some((midi, amp)));
        } else {
            stable.push(Some((locked, amp * 0.75)));
        }
    }

    stable
}

fn build_note_events_for_track(
    frames: &[AnalysisFrame],
    track: &'static str,
    min_midi: i32,
    max_midi: i32,
    instrument_id: &'static str,
    context: &MusicalContext,
) -> Vec<NoteEvent> {
    let min_note_frames = if track == "bass" { 8 } else { 6 };
    let max_bridge_gap = if track == "bass" { 3 } else { 2 };
    let candidates = stable_midi_candidates(frames, track, min_midi, max_midi, context);
    let mut notes = Vec::new();
    let mut start = 0usize;
    let mut active_midi: Option<i32> = None;
    let mut velocity_sum = 0.0;
    let mut velocity_count = 0usize;
    let mut gap = 0usize;

    let finish_note = |notes: &mut Vec<NoteEvent>,
                       start: usize,
                       end: usize,
                       midi: i32,
                       velocity_sum: f32,
                       velocity_count: usize| {
        let frames_len = end.saturating_sub(start);
        if frames_len >= min_note_frames && velocity_count > 0 {
            notes.push(NoteEvent {
                track,
                start_frame: start,
                frames: frames_len,
                midi,
                hz: midi_to_hz(midi),
                velocity: (velocity_sum / velocity_count as f32).clamp(0.05, 1.0),
                instrument_id,
            });
        }
    };

    for (index, candidate) in candidates.iter().enumerate() {
        match (active_midi, candidate) {
            (None, Some((midi, amp))) => {
                active_midi = Some(*midi);
                start = index;
                velocity_sum = *amp;
                velocity_count = 1;
                gap = 0;
            }
            (Some(midi), Some((next_midi, amp))) if midi == *next_midi => {
                velocity_sum += *amp;
                velocity_count += 1;
                gap = 0;
            }
            (Some(midi), Some((next_midi, amp))) => {
                finish_note(
                    &mut notes,
                    start,
                    index.saturating_sub(gap),
                    midi,
                    velocity_sum,
                    velocity_count,
                );
                active_midi = Some(*next_midi);
                start = index;
                velocity_sum = *amp;
                velocity_count = 1;
                gap = 0;
            }
            (Some(midi), None) => {
                gap += 1;
                if gap <= max_bridge_gap {
                    continue;
                }
                finish_note(
                    &mut notes,
                    start,
                    index + 1 - gap,
                    midi,
                    velocity_sum,
                    velocity_count,
                );
                active_midi = None;
                velocity_sum = 0.0;
                velocity_count = 0;
                gap = 0;
            }
            (None, None) => {}
        }
    }

    if let Some(midi) = active_midi {
        finish_note(
            &mut notes,
            start,
            candidates.len().saturating_sub(gap),
            midi,
            velocity_sum,
            velocity_count,
        );
    }

    quantize_note_grid(merge_neighbor_notes(notes), context)
}

fn quantize_note_grid(notes: Vec<NoteEvent>, context: &MusicalContext) -> Vec<NoteEvent> {
    let max_shift = (context.grid_frames / 3).max(1);
    let mut quantized = Vec::with_capacity(notes.len());
    for mut note in notes {
        let end = note.start_frame.saturating_add(note.frames);
        let snapped_start = snap_frame_to_grid(note.start_frame, context.grid_frames, max_shift);
        let snapped_end = snap_frame_to_grid(end, context.grid_frames, max_shift);
        if snapped_end > snapped_start {
            note.start_frame = snapped_start;
            note.frames = snapped_end - snapped_start;
        }
        quantized.push(note);
    }
    merge_neighbor_notes(quantized)
}

fn merge_neighbor_notes(notes: Vec<NoteEvent>) -> Vec<NoteEvent> {
    let mut merged: Vec<NoteEvent> = Vec::new();
    for note in notes {
        if let Some(last) = merged.last_mut() {
            let last_end = last.start_frame.saturating_add(last.frames);
            let gap = note.start_frame.saturating_sub(last_end);
            if last.midi == note.midi && gap <= 4 {
                let old_frames = last.frames;
                let total_frames = old_frames + gap + note.frames;
                let weighted_velocity =
                    last.velocity * old_frames as f32 + note.velocity * note.frames as f32;
                last.frames = total_frames;
                last.velocity =
                    (weighted_velocity / (old_frames + note.frames) as f32).clamp(0.05, 1.0);
                continue;
            }
        }
        merged.push(note);
    }
    merged
}

fn note_overlap_frames(a: &NoteEvent, b: &NoteEvent) -> usize {
    let a_end = a.start_frame.saturating_add(a.frames);
    let b_end = b.start_frame.saturating_add(b.frames);
    a_end
        .min(b_end)
        .saturating_sub(a.start_frame.max(b.start_frame))
}

fn filter_lead_notes_against_bass(
    lead_notes: Vec<NoteEvent>,
    bass_notes: &[NoteEvent],
) -> Vec<NoteEvent> {
    lead_notes
        .into_iter()
        .filter(|lead| lead.velocity >= 0.075 && lead.frames >= 8)
        .filter(|lead| {
            !bass_notes.iter().any(|bass| {
                let overlap = note_overlap_frames(lead, bass);
                overlap * 2 >= lead.frames
                    && lead.midi <= bass.midi + 12
                    && (lead.midi - bass.midi).abs() <= 16
            })
        })
        .collect()
}

fn lock_bass_pattern(notes: Vec<NoteEvent>, context: &MusicalContext) -> Vec<NoteEvent> {
    let bar_frames = (context.frames_per_beat * 4.0).round().max(16.0) as usize;
    let slot_count = 16usize;
    let slot_frames = (bar_frames / slot_count).max(1);
    let mut slot_counts = vec![[0u16; 12]; slot_count];

    for note in &notes {
        if note.velocity < 0.08 {
            continue;
        }
        let slot = (note.start_frame % bar_frames) / slot_frames;
        slot_counts[slot.min(slot_count - 1)][note.midi.rem_euclid(12) as usize] += 1;
    }

    notes
        .into_iter()
        .map(|mut note| {
            let slot = ((note.start_frame % bar_frames) / slot_frames).min(slot_count - 1);
            let (pc, count) = slot_counts[slot]
                .iter()
                .enumerate()
                .max_by_key(|(_, count)| **count)
                .map(|(pc, count)| (pc as i32, *count))
                .unwrap_or((note.midi.rem_euclid(12), 0));
            if count >= 3
                && note.velocity < 0.42
                && note.frames < (context.frames_per_beat * 1.25) as usize
            {
                let mut candidate = note.midi;
                while candidate.rem_euclid(12) != pc {
                    candidate += 1;
                    if candidate > 47 {
                        candidate -= 12;
                    }
                }
                note.midi = fold_midi_to_range(candidate, 24, 47);
                note.hz = midi_to_hz(note.midi);
            }
            note
        })
        .collect()
}

fn nearest_dac_chunk(frame_index: usize, chunks: &[DacChunk]) -> Option<&DacChunk> {
    chunks
        .iter()
        .filter(|chunk| frame_index.abs_diff(chunk.start_frame) <= 18)
        .min_by_key(|chunk| frame_index.abs_diff(chunk.start_frame))
}

fn build_drum_events(frames: &[AnalysisFrame], chunks: &[DacChunk]) -> Vec<DrumEvent> {
    let mut drums = Vec::new();
    let mut last_frame = 0usize;

    for frame in frames {
        let noise_hit = matches!(frame.class_name, "drum" | "sample")
            && frame.transient > 0.18
            && frame.noise_amp > 0.06;
        let transient_hit = frame.transient > 0.72 && frame.rms > 0.04;
        let clear_drum = noise_hit || transient_hit;
        if clear_drum && frame.index.saturating_sub(last_frame) > 6 {
            let dac = nearest_dac_chunk(frame.index, chunks);
            drums.push(DrumEvent {
                start_frame: frame.index,
                kind: if frame.rms > 0.18 {
                    "kick"
                } else if noise_hit {
                    "noise_hit"
                } else {
                    "transient_hit"
                },
                velocity: frame.transient.clamp(0.1, 1.0),
                noise_amp: frame.noise_amp.max(frame.transient * 0.35).clamp(0.0, 1.0),
                instrument_id: DEFAULT_NOISE_INSTRUMENT,
                dac_chunk: dac.map(|chunk| chunk.id),
                dac_path: dac.map(|chunk| chunk.path.clone()).unwrap_or_default(),
            });
            last_frame = frame.index;
        }
    }

    drums
}

fn build_note_preview_data(
    frames: &[AnalysisFrame],
    sample_rate: u32,
    chunks: &[DacChunk],
) -> (
    MusicalContext,
    Vec<NoteEvent>,
    Vec<NoteEvent>,
    Vec<DrumEvent>,
) {
    let context = analyse_musical_context(frames, sample_rate);
    let bass_notes = lock_bass_pattern(
        build_note_events_for_track(frames, "bass", 24, 47, DEFAULT_BASS_INSTRUMENT, &context),
        &context,
    );
    let lead_notes = filter_lead_notes_against_bass(
        build_note_events_for_track(frames, "lead", 48, 84, DEFAULT_LEAD_INSTRUMENT, &context),
        &bass_notes,
    );
    let drum_events = build_drum_events(frames, chunks);
    (context, bass_notes, lead_notes, drum_events)
}

fn write_preview_report_json(
    path: &Path,
    input: &Path,
    wav: &WavData,
    frames: &[AnalysisFrame],
    chunks: &[DacChunk],
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let (context, bass_notes, lead_notes, drum_events) =
        build_note_preview_data(frames, wav.sample_rate, chunks);
    let dac_drums = drum_events
        .iter()
        .filter(|drum| drum.dac_chunk.is_some())
        .count();
    let active_frames = frames
        .iter()
        .filter(|frame| frame.class_name != "silence")
        .count();
    let avg_bass_frames = if bass_notes.is_empty() {
        0.0
    } else {
        bass_notes
            .iter()
            .map(|note| note.frames as f32)
            .sum::<f32>()
            / bass_notes.len() as f32
    };
    let avg_lead_frames = if lead_notes.is_empty() {
        0.0
    } else {
        lead_notes
            .iter()
            .map(|note| note.frames as f32)
            .sum::<f32>()
            / lead_notes.len() as f32
    };
    let text = format!(
        concat!(
            "{{\n",
            "  \"format\": \"vandaler-vand-preview-report-v0\",\n",
            "  \"source\": \"{}\",\n",
            "  \"key\": \"{} {}\",\n",
            "  \"key_root\": {},\n",
            "  \"mode\": \"{}\",\n",
            "  \"key_confidence\": {:.5},\n",
            "  \"bpm\": {:.3},\n",
            "  \"frames_per_beat\": {:.3},\n",
            "  \"grid_frames\": {},\n",
            "  \"loop_start_frame\": {},\n",
            "  \"loop_end_frame\": {},\n",
            "  \"loop_start\": {:.6},\n",
            "  \"loop_end\": {:.6},\n",
            "  \"frames\": {},\n",
            "  \"active_frames\": {},\n",
            "  \"bass_notes\": {},\n",
            "  \"lead_notes\": {},\n",
            "  \"drum_events\": {},\n",
            "  \"dac_drum_events\": {},\n",
            "  \"avg_bass_note_frames\": {:.3},\n",
            "  \"avg_lead_note_frames\": {:.3}\n",
            "}}\n"
        ),
        json_escape(&input.display().to_string()),
        pitch_class_name(context.key_root),
        context.mode,
        context.key_root,
        context.mode,
        context.key_confidence,
        context.bpm,
        context.frames_per_beat,
        context.grid_frames,
        context.loop_start_frame,
        context.loop_end_frame,
        context.loop_start_frame as f32 * ANALYSIS_HOP as f32 / wav.sample_rate as f32,
        context.loop_end_frame as f32 * ANALYSIS_HOP as f32 / wav.sample_rate as f32,
        frames.len(),
        active_frames,
        bass_notes.len(),
        lead_notes.len(),
        drum_events.len(),
        dac_drums,
        avg_bass_frames,
        avg_lead_frames
    );
    std::fs::write(path, text)
}

fn write_note_arrangement_json(
    path: &Path,
    input: &Path,
    wav: &WavData,
    frames: &[AnalysisFrame],
    chunks: &[DacChunk],
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let (context, bass_notes, lead_notes, drum_events) =
        build_note_preview_data(frames, wav.sample_rate, chunks);
    let dac_drums = drum_events
        .iter()
        .filter(|drum| drum.dac_chunk.is_some())
        .count();
    let duration = wav.samples.len() as f32 / wav.sample_rate as f32;
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"format\": \"vandaler-vand-note-arrangement-v0\",\n");
    out.push_str(&format!(
        "  \"source\": \"{}\",\n",
        json_escape(&input.display().to_string())
    ));
    out.push_str(&format!("  \"sample_rate\": {},\n", wav.sample_rate));
    out.push_str(&format!("  \"duration\": {:.6},\n", duration));
    out.push_str(&format!("  \"hop\": {},\n", ANALYSIS_HOP));
    out.push_str(&format!("  \"total_frames\": {},\n", frames.len()));
    out.push_str(&format!(
        "  \"instrument_bank\": \"{}\",\n",
        DEFAULT_INSTRUMENT_BANK
    ));
    out.push_str(&format!(
        concat!(
            "  \"analysis\": {{\"key\": \"{} {}\", \"key_root\": {}, \"mode\": \"{}\", ",
            "\"key_confidence\": {:.5}, \"bpm\": {:.3}, \"frames_per_beat\": {:.3}, ",
            "\"grid_frames\": {}, \"loop_start_frame\": {}, \"loop_end_frame\": {}, ",
            "\"bass_notes\": {}, \"lead_notes\": {}, \"drum_events\": {}, ",
            "\"dac_drum_events\": {}}},\n"
        ),
        pitch_class_name(context.key_root),
        context.mode,
        context.key_root,
        context.mode,
        context.key_confidence,
        context.bpm,
        context.frames_per_beat,
        context.grid_frames,
        context.loop_start_frame,
        context.loop_end_frame,
        bass_notes.len(),
        lead_notes.len(),
        drum_events.len(),
        dac_drums
    ));
    out.push_str("  \"pipeline\": [\"frame_analysis\", \"key_detection\", \"scale_snap\", \"beat_grid\", \"pitch_hysteresis\", \"bass_pattern_lock\", \"lead_confidence_filter\", \"drum_gate\"],\n");

    out.push_str("  \"notes\": [\n");
    let total_notes = bass_notes.len() + lead_notes.len();
    let mut note_index = 0usize;
    for note in bass_notes.iter().chain(lead_notes.iter()) {
        note_index += 1;
        let comma = if note_index == total_notes { "" } else { "," };
        out.push_str(&format!(
            concat!(
                "    {{\"track\": \"{}\", \"start_frame\": {}, \"frames\": {}, ",
                "\"t\": {:.6}, \"duration\": {:.6}, \"midi\": {}, \"hz\": {:.3}, ",
                "\"note\": \"{}\", \"velocity\": {:.5}, \"instrument_id\": \"{}\"}}{}\n"
            ),
            note.track,
            note.start_frame,
            note.frames,
            note.start_frame as f32 * ANALYSIS_HOP as f32 / wav.sample_rate as f32,
            note.frames as f32 * ANALYSIS_HOP as f32 / wav.sample_rate as f32,
            note.midi,
            note.hz,
            note_name_from_midi(note.midi),
            note.velocity,
            note.instrument_id,
            comma,
        ));
    }
    out.push_str("  ],\n");

    out.push_str("  \"drums\": [\n");
    for (i, drum) in drum_events.iter().enumerate() {
        let comma = if i + 1 == drum_events.len() { "" } else { "," };
        out.push_str(&format!(
            concat!(
                "    {{\"start_frame\": {}, \"t\": {:.6}, \"kind\": \"{}\", ",
                "\"velocity\": {:.5}, \"noise_amp\": {:.5}, \"instrument_id\": \"{}\", ",
                "\"dac_chunk\": {}, \"dac_path\": \"{}\"}}{}\n"
            ),
            drum.start_frame,
            drum.start_frame as f32 * ANALYSIS_HOP as f32 / wav.sample_rate as f32,
            drum.kind,
            drum.velocity,
            drum.noise_amp,
            drum.instrument_id,
            drum.dac_chunk
                .map(|id| id.to_string())
                .unwrap_or_else(|| "null".to_string()),
            json_escape(&drum.dac_path),
            comma,
        ));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    std::fs::write(path, out)
}

fn write_render_plan_json(
    path: &Path,
    input: &Path,
    wav: &WavData,
    frames: &[AnalysisFrame],
    chunks: &[DacChunk],
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let (context, bass_notes, lead_notes, drum_events) =
        build_note_preview_data(frames, wav.sample_rate, chunks);
    let duration = wav.samples.len() as f32 / wav.sample_rate as f32;
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"format\": \"vandaler-render-plan-v0\",\n");
    out.push_str(&format!(
        "  \"source\": \"{}\",\n",
        json_escape(&input.display().to_string())
    ));
    out.push_str(&format!("  \"sample_rate\": {},\n", wav.sample_rate));
    out.push_str(&format!("  \"duration\": {:.6},\n", duration));
    out.push_str(&format!("  \"hop\": {},\n", ANALYSIS_HOP));
    out.push_str(&format!("  \"total_frames\": {},\n", frames.len()));
    out.push_str(&format!(
        "  \"instrument_bank\": \"{}\",\n",
        DEFAULT_INSTRUMENT_BANK
    ));
    out.push_str(&format!(
        "  \"tempo\": {{\"bpm\": {:.3}, \"frames_per_beat\": {:.3}, \"grid_frames\": {}}},\n",
        context.bpm, context.frames_per_beat, context.grid_frames
    ));
    out.push_str(&format!(
        "  \"loop\": {{\"start_frame\": {}, \"end_frame\": {}, \"start\": {:.6}, \"end\": {:.6}}},\n",
        context.loop_start_frame,
        context.loop_end_frame,
        context.loop_start_frame as f32 * ANALYSIS_HOP as f32 / wav.sample_rate as f32,
        context.loop_end_frame as f32 * ANALYSIS_HOP as f32 / wav.sample_rate as f32
    ));
    out.push_str("  \"mix\": {\"bass_gain\": 1.0, \"lead_gain\": 0.85, \"psg_gain\": 0.85, \"dac_gain\": 1.0},\n");
    out.push_str("  \"tracks\": [\n");

    for (track_index, (role, chip, channel, notes)) in [
        ("bass", "ym2612", 0usize, bass_notes.as_slice()),
        ("lead", "ym2612", 1usize, lead_notes.as_slice()),
    ]
    .iter()
    .enumerate()
    {
        let track_comma = if track_index == 1 { "" } else { "," };
        out.push_str(&format!(
            "    {{\"role\": \"{}\", \"chip\": \"{}\", \"channel\": {}, \"events\": [\n",
            role, chip, channel
        ));
        for (i, note) in notes.iter().enumerate() {
            let comma = if i + 1 == notes.len() { "" } else { "," };
            out.push_str(&format!(
                concat!(
                    "      {{\"start_frame\": {}, \"frames\": {}, \"midi\": {}, ",
                    "\"hz\": {:.3}, \"velocity\": {:.5}, \"instrument_id\": \"{}\"}}{}\n"
                ),
                note.start_frame,
                note.frames,
                note.midi,
                note.hz,
                note.velocity,
                note.instrument_id,
                comma
            ));
        }
        out.push_str(&format!("    ]}}{}\n", track_comma));
    }
    out.push_str("  ],\n");

    out.push_str("  \"drums\": [\n");
    for (i, drum) in drum_events.iter().enumerate() {
        let comma = if i + 1 == drum_events.len() { "" } else { "," };
        out.push_str(&format!(
            concat!(
                "    {{\"start_frame\": {}, \"kind\": \"{}\", \"velocity\": {:.5}, ",
                "\"psg_noise_amp\": {:.5}, \"psg_instrument_id\": \"{}\", ",
                "\"dac_chunk\": {}, \"dac_path\": \"{}\"}}{}\n"
            ),
            drum.start_frame,
            drum.kind,
            drum.velocity,
            drum.noise_amp,
            drum.instrument_id,
            drum.dac_chunk
                .map(|id| id.to_string())
                .unwrap_or_else(|| "null".to_string()),
            json_escape(&drum.dac_path),
            comma
        ));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    std::fs::write(path, out)
}

fn analyse_input(
    input: &Path,
    out: Option<PathBuf>,
    install_sgdk: bool,
) -> io::Result<AnalysisSummary> {
    let out = out.unwrap_or_else(|| default_analysis_path(input));
    let bundle_dir = out.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
    let imported = import_audio(input, &bundle_dir)?;
    let frames = analyse_samples(&imported.wav);
    let dac_chunks = extract_dac_chunks(&bundle_dir, &imported.wav, &frames)?;
    let runtime_events =
        build_render_plan_runtime_events(&frames, imported.wav.sample_rate, &dac_chunks);
    let import_metadata = bundle_dir.join("import.json");
    let note_arrangement = bundle_dir.join("note_arrangement.vand-audio.json");
    let render_plan = bundle_dir.join("render_plan.vand-audio.json");
    let preview_report = bundle_dir.join("preview_report.json");
    let dac_preview = write_dac_preview_wav(&bundle_dir, &dac_chunks)?;
    let runtime_preview = write_runtime_preview_wav(&bundle_dir, &runtime_events, &dac_chunks)?;
    write_import_metadata(&import_metadata, input, &imported)?;
    write_analysis_json(&out, input, &imported.wav, &frames)?;
    write_note_arrangement_json(
        &note_arrangement,
        input,
        &imported.wav,
        &frames,
        &dac_chunks,
    )?;
    write_render_plan_json(&render_plan, input, &imported.wav, &frames, &dac_chunks)?;
    write_preview_report_json(&preview_report, input, &imported.wav, &frames, &dac_chunks)?;
    write_split_tracks(&bundle_dir, input, &frames)?;
    write_dac_chunks_json(
        &bundle_dir.join("dac_chunks.json"),
        input,
        imported.wav.sample_rate,
        &dac_chunks,
    )?;
    write_runtime_binary(
        &bundle_dir.join("events.vandbin"),
        imported.wav.sample_rate,
        &runtime_events,
    )?;
    write_sgdk_audio(
        &bundle_dir.join("sgdk_audio.h"),
        &bundle_dir.join("sgdk_audio.c"),
        &runtime_events,
        &dac_chunks,
    )?;
    if install_sgdk {
        write_sgdk_audio(
            Path::new("out/generated_audio.h"),
            Path::new("out/generated_audio.c"),
            &runtime_events,
            &dac_chunks,
        )?;
    }
    let active = frames
        .iter()
        .filter(|frame| frame.class_name != "silence")
        .count();
    Ok(AnalysisSummary {
        frames: frames.len(),
        active_frames: active,
        runtime_events: runtime_events.len(),
        dac_chunks: dac_chunks.len(),
        arrangement: out,
        note_arrangement,
        render_plan,
        preview_report,
        bundle_dir,
        import_metadata,
        dac_preview,
        runtime_preview,
    })
}

fn analyse_wav(args: &Args) -> io::Result<()> {
    let input = args
        .rom
        .as_deref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing audio path"))?;
    let summary = analyse_input(input, args.out.clone(), args.install_sgdk)?;
    println!(
        "analysed {} | {} frames, {} active, {} runtime events, {} dac chunks | wrote {} | note {} | render {} | report {}",
        input.display(),
        summary.frames,
        summary.active_frames,
        summary.runtime_events,
        summary.dac_chunks,
        summary.arrangement.display(),
        summary.note_arrangement.display(),
        summary.render_plan.display(),
        summary.preview_report.display()
    );
    Ok(())
}

fn play_audio(path: &Path) -> io::Result<()> {
    let status = Command::new("pw-play").arg(path).status()?;
    if !status.success() {
        return Err(io::Error::other("pw-play failed"));
    }
    Ok(())
}

fn prompt_line(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn print_gui_help(selected: Option<&Path>, output_dir: &Path, last: Option<&AnalysisSummary>) {
    println!();
    println!("============================================================");
    println!("  Vand-AI-lism");
    println!("  Mega Drive audio transcription lab");
    println!("============================================================");
    println!(
        "source: {}",
        selected
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "(none)".to_string())
    );
    println!("output: {}", output_dir.display());
    if let Some(summary) = last {
        println!(
            "last: {} frames, {} active, {} runtime events, {} DAC chunks",
            summary.frames, summary.active_frames, summary.runtime_events, summary.dac_chunks
        );
        println!("bundle: {}", summary.bundle_dir.display());
    }
    println!();
    println!("commands:");
    println!("  o <audio> open MP3/OGG/FLAC/WAV");
    println!("  out <dir> choose output directory for bundles");
    println!("  a        analyse/export current source");
    println!("  a <dir>  analyse once to a specific output directory");
    println!("  p        play original source with PipeWire");
    println!("  d        play DAC chunk audition WAV");
    println!("  t        build and render SGDK audio test ROM");
    println!("  h        show this help");
    println!("  q        quit");
    println!();
}

fn launch_gui() -> io::Result<()> {
    let mut selected: Option<PathBuf> = None;
    let mut output_dir = PathBuf::from("audio/converted");
    let mut last_summary: Option<AnalysisSummary> = None;
    print_gui_help(selected.as_deref(), &output_dir, last_summary.as_ref());

    loop {
        let input = prompt_line("Vand-AI-lism> ")?;
        let mut parts = input.split_whitespace();
        let Some(command) = parts.next() else {
            continue;
        };

        match command {
            "o" | "open" => {
                let path_text = parts.collect::<Vec<_>>().join(" ");
                if path_text.is_empty() {
                    println!("open needs an audio path");
                    continue;
                }
                selected = Some(PathBuf::from(path_text));
                last_summary = None;
                print_gui_help(selected.as_deref(), &output_dir, last_summary.as_ref());
            }
            "out" | "output" => {
                let path_text = parts.collect::<Vec<_>>().join(" ");
                if path_text.is_empty() {
                    println!("output needs a directory path");
                    continue;
                }
                output_dir = PathBuf::from(path_text);
                last_summary = None;
                print_gui_help(selected.as_deref(), &output_dir, last_summary.as_ref());
            }
            "a" | "analyse" | "analyze" => {
                let Some(path) = selected.as_deref() else {
                    println!("choose audio first with: o path/to/file.ogg");
                    continue;
                };
                let override_dir = parts.collect::<Vec<_>>().join(" ");
                let active_output_dir = if override_dir.is_empty() {
                    output_dir.clone()
                } else {
                    PathBuf::from(override_dir)
                };
                let out = analysis_path_in_dir(path, &active_output_dir);
                println!(
                    "analysing {} -> {} ...",
                    path.display(),
                    active_output_dir.display()
                );
                match analyse_input(path, Some(out), false) {
                    Ok(summary) => {
                        println!(
                            "wrote {}, {}, {}, and {}",
                            summary.arrangement.display(),
                            summary.import_metadata.display(),
                            summary.dac_preview.display(),
                            summary.runtime_preview.display()
                        );
                        last_summary = Some(summary);
                    }
                    Err(err) => println!("analysis failed: {err}"),
                }
            }
            "p" | "play" => {
                let Some(path) = selected.as_deref() else {
                    println!("choose audio first with: o path/to/file.ogg");
                    continue;
                };
                if let Err(err) = play_audio(path) {
                    println!("play failed: {err}");
                }
            }
            "d" | "dac" => {
                let Some(summary) = last_summary.as_ref() else {
                    println!("run analysis first with: a");
                    continue;
                };
                if let Err(err) = play_audio(&summary.dac_preview) {
                    println!("DAC preview failed: {err}");
                }
            }
            "t" | "test" => {
                let args = Args {
                    command: CommandMode::TestRom,
                    rom: None,
                    wav: PathBuf::from("out/audio-test.wav"),
                    report: Some(PathBuf::from("out/audio-test-report.json")),
                    out: None,
                    seconds: DEFAULT_SECONDS,
                    sample_rate: DEFAULT_RATE,
                    play: true,
                    install_sgdk: false,
                    start_malmo: false,
                };
                if let Err(err) = test_rom(&args) {
                    println!("test ROM failed: {err}");
                }
            }
            "h" | "help" => print_gui_help(selected.as_deref(), &output_dir, last_summary.as_ref()),
            "q" | "quit" | "exit" => break,
            _ => println!("unknown command: {command}"),
        }
    }

    Ok(())
}

fn run() -> io::Result<()> {
    let args = parse_args()?;
    match args.command {
        CommandMode::AnalyseWav => analyse_wav(&args),
        CommandMode::Gui => launch_gui(),
        CommandMode::ImportInstrumentDir => import_instrument_dir(&args),
        CommandMode::ImportInstrument => import_instrument(&args),
        CommandMode::InspectInstrument => inspect_instrument(&args),
        CommandMode::RenderRom => render_rom(&args),
        CommandMode::TestRom => test_rom(&args),
    }
}

fn main() {
    if let Err(err) = run() {
        usage();
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
