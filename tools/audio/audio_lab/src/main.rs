use euther_oxide::Emulator;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_SECONDS: u32 = 5;
const DEFAULT_RATE: u32 = 44_100;
const ANALYSIS_FRAME: usize = 1024;
const ANALYSIS_HOP: usize = 512;

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
}

#[derive(Clone, Copy)]
enum CommandMode {
    RenderRom,
    TestRom,
    AnalyseWav,
}

fn usage() {
    eprintln!(
        "usage:\n  vandaler-audio-lab analyse-wav input.wav [--out arrangement.vand-audio.json] [--install-sgdk]\n  vandaler-audio-lab render-rom ROM [--wav out.wav] [--report report.json] [--seconds N] [--rate HZ] [--play]\n  vandaler-audio-lab test-rom [--wav out/audio-test.wav] [--report out/audio-test-report.json] [--seconds N] [--rate HZ] [--play]"
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

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "render-rom" if command.is_none() => command = Some(CommandMode::RenderRom),
            "analyse-wav" if command.is_none() => command = Some(CommandMode::AnalyseWav),
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
    if matches!(command, CommandMode::RenderRom | CommandMode::AnalyseWav) && rom.is_none() {
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
) -> io::Result<AudioStats> {
    let mut emulator = Emulator::new();
    let mut pcm = Vec::new();
    let frames = seconds.saturating_mul(60).max(1);

    emulator.load_rom_file(rom)?;
    for _ in 0..frames {
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
    )?;
    Ok(())
}

struct WavData {
    sample_rate: u32,
    samples: Vec<f32>,
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
}

fn read_u16_le(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[0], bytes[1]])
}

fn read_u32_le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
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

fn midi_to_hz(midi: i32) -> f32 {
    440.0 * 2.0f32.powf((midi as f32 - 69.0) / 12.0)
}

fn note_name(hz: f32) -> String {
    if hz <= 0.0 {
        return "---".to_string();
    }
    let midi = (69.0 + 12.0 * (hz / 440.0).log2()).round() as i32;
    let names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    format!("{}{}", names[midi.rem_euclid(12) as usize], midi / 12 - 1)
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
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    PathBuf::from("audio")
        .join("converted")
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

fn frame_to_runtime(frame: &AnalysisFrame) -> RuntimeEvent {
    let (fm0_fnum, fm0_block) = ym2612_pitch(frame.bass_hz);
    let (fm1_fnum, fm1_block) = ym2612_pitch(frame.lead_hz);
    let kind = class_id(frame.class_name);
    let noise = if kind == 3 || kind == 4 {
        frame.noise_amp.max(frame.transient)
    } else {
        frame.noise_amp * 0.75
    };

    RuntimeEvent {
        frames: 1,
        fm0_fnum,
        fm0_block,
        fm0_level: if fm0_fnum == 0 {
            0
        } else {
            level15(frame.bass_amp)
        },
        fm1_fnum,
        fm1_block,
        fm1_level: if fm1_fnum == 0 {
            0
        } else {
            level15(frame.lead_amp)
        },
        psg_noise_level: level15(noise),
        kind,
    }
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
}

fn runtime_ticks(frame_count: usize, sample_rate: u32) -> u16 {
    let seconds = frame_count as f32 * ANALYSIS_HOP as f32 / sample_rate as f32;
    (seconds * 60.0).round().clamp(1.0, u16::MAX as f32) as u16
}

fn build_runtime_events(frames: &[AnalysisFrame], sample_rate: u32) -> Vec<RuntimeEvent> {
    let mut out = Vec::new();
    let mut pending: Option<RuntimeEvent> = None;
    let mut pending_frames = 0usize;

    for frame in frames {
        let next = frame_to_runtime(frame);
        match pending.as_mut() {
            Some(current) if equivalent_runtime(current, &next) => {
                pending_frames += 1;
            }
            Some(current) => {
                current.frames = runtime_ticks(pending_frames, sample_rate);
                out.push(current.clone());
                pending = Some(next);
                pending_frames = 1;
            }
            None => {
                pending = Some(next);
                pending_frames = 1;
            }
        }
    }

    if let Some(mut current) = pending {
        current.frames = runtime_ticks(pending_frames, sample_rate);
        out.push(current);
    }

    out
}

fn write_runtime_binary(path: &Path, sample_rate: u32, events: &[RuntimeEvent]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    file.write_all(b"VADB")?;
    file.write_all(&1u16.to_le_bytes())?;
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
    }

    Ok(())
}

fn write_sgdk_audio(header: &Path, source: &Path, events: &[RuntimeEvent]) -> io::Result<()> {
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
        "#endif\n"
    );
    std::fs::write(header, header_text)?;

    let include_name = header
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("generated_audio.h");
    let mut c = String::new();
    c.push_str(&format!("#include \"{}\"\n\n", include_name));
    c.push_str("const VandAudioEvent generatedAudioEvents[] =\n{\n");
    for event in events {
        c.push_str(&format!(
            "    {{{}, 0x{:03X}, {}, {}, 0x{:03X}, {}, {}, {}, {}}},\n",
            event.frames,
            event.fm0_fnum,
            event.fm0_block,
            event.fm0_level,
            event.fm1_fnum,
            event.fm1_block,
            event.fm1_level,
            event.psg_noise_level,
            event.kind
        ));
    }
    c.push_str("};\n\n");
    c.push_str(
        "const u16 generatedAudioEventCount = sizeof(generatedAudioEvents) / sizeof(generatedAudioEvents[0]);\n",
    );
    std::fs::write(source, c)
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
    out.push_str("  \"pipeline\": [\"wav_decode_pcm16\", \"goertzel_pitch_tracking\", \"transient_detection\", \"frame_classification\"],\n");
    out.push_str("  \"events\": [\n");
    for (i, frame) in frames.iter().enumerate() {
        let comma = if i + 1 == frames.len() { "" } else { "," };
        out.push_str(&format!(
            concat!(
                "    {{\"index\": {}, \"t\": {:.6}, \"class\": \"{}\", ",
                "\"rms\": {:.5}, \"transient\": {:.5}, ",
                "\"bass_hz\": {:.3}, \"bass_note\": \"{}\", \"bass_amp\": {:.5}, ",
                "\"lead_hz\": {:.3}, \"lead_note\": \"{}\", \"lead_amp\": {:.5}, ",
                "\"noise_amp\": {:.5}}}{}\n"
            ),
            frame.index,
            frame.time,
            frame.class_name,
            frame.rms,
            frame.transient,
            frame.bass_hz,
            note_name(frame.bass_hz),
            frame.bass_amp,
            frame.lead_hz,
            note_name(frame.lead_hz),
            frame.lead_amp,
            frame.noise_amp,
            comma,
        ));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    std::fs::write(path, out)
}

fn analyse_wav(args: &Args) -> io::Result<()> {
    let input = args
        .rom
        .as_deref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing WAV path"))?;
    let wav = read_wav_mono(input)?;
    let frames = analyse_samples(&wav);
    let runtime_events = build_runtime_events(&frames, wav.sample_rate);
    let out = args
        .out
        .clone()
        .unwrap_or_else(|| default_analysis_path(input));
    let bundle_dir = out.parent().unwrap_or_else(|| Path::new("."));
    write_analysis_json(&out, input, &wav, &frames)?;
    write_runtime_binary(
        &bundle_dir.join("events.vandbin"),
        wav.sample_rate,
        &runtime_events,
    )?;
    write_sgdk_audio(
        &bundle_dir.join("sgdk_audio.h"),
        &bundle_dir.join("sgdk_audio.c"),
        &runtime_events,
    )?;
    if args.install_sgdk {
        write_sgdk_audio(
            Path::new("src/generated_audio.h"),
            Path::new("src/generated_audio.c"),
            &runtime_events,
        )?;
    }
    let active = frames
        .iter()
        .filter(|frame| frame.class_name != "silence")
        .count();
    println!(
        "analysed {} | {} frames, {} active, {} runtime events | wrote {}",
        input.display(),
        frames.len(),
        active,
        runtime_events.len(),
        out.display()
    );
    Ok(())
}

fn run() -> io::Result<()> {
    let args = parse_args()?;
    match args.command {
        CommandMode::AnalyseWav => analyse_wav(&args),
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
