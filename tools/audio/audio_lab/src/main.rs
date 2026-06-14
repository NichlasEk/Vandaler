use euther_oxide::Emulator;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_SECONDS: u32 = 5;
const DEFAULT_RATE: u32 = 44_100;

struct Args {
    command: CommandMode,
    rom: Option<PathBuf>,
    wav: PathBuf,
    report: Option<PathBuf>,
    seconds: u32,
    sample_rate: u32,
    play: bool,
}

#[derive(Clone, Copy)]
enum CommandMode {
    RenderRom,
    TestRom,
}

fn usage() {
    eprintln!(
        "usage:\n  vandaler-audio-lab render-rom ROM [--wav out.wav] [--report report.json] [--seconds N] [--rate HZ] [--play]\n  vandaler-audio-lab test-rom [--wav out/audio-test.wav] [--report out/audio-test-report.json] [--seconds N] [--rate HZ] [--play]"
    );
}

fn parse_args() -> io::Result<Args> {
    let mut iter = env::args().skip(1);
    let mut command = None;
    let mut rom = None;
    let mut wav = PathBuf::from("out/audio-lab-render.wav");
    let mut report = None;
    let mut seconds = DEFAULT_SECONDS;
    let mut sample_rate = DEFAULT_RATE;
    let mut play = false;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "render-rom" if command.is_none() => command = Some(CommandMode::RenderRom),
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
    if matches!(command, CommandMode::RenderRom) && rom.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "missing ROM path",
        ));
    }
    Ok(Args {
        command,
        rom,
        wav,
        report,
        seconds,
        sample_rate,
        play,
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

fn run() -> io::Result<()> {
    let args = parse_args()?;
    match args.command {
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
