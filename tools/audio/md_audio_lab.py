#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import math
import shutil
import subprocess
import sys
import tempfile
import wave
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

import numpy as np


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_OUT = ROOT / "audio" / "converted"
ANALYSIS_RATE = 22050
PREVIEW_RATE = 44100
VGM_RATE = 44100
RUNTIME_RATE = 60
YM2612_CLOCK = 7_670_454
FRAME = 2048
HOP = 512
CLASS_IDS = {
    "silence": 0,
    "ambience": 1,
    "noise": 2,
    "sample": 3,
    "drum": 4,
    "bass": 5,
    "tonal": 6,
}


@dataclass
class FrameEvent:
    t: float
    bass_hz: float
    lead_hz: float
    bass_amp: float
    lead_amp: float
    noise_amp: float
    transient: float
    kind: str


def require_tool(name: str) -> str:
    path = shutil.which(name)
    if not path:
        raise SystemExit(f"Missing required tool: {name}")
    return path


def run(cmd: list[str]) -> None:
    subprocess.run(cmd, check=True)


def decode_audio(path: Path, sample_rate: int = ANALYSIS_RATE) -> np.ndarray:
    require_tool("ffmpeg")
    with tempfile.NamedTemporaryFile(suffix=".wav") as tmp:
        run([
            "ffmpeg", "-hide_banner", "-loglevel", "error", "-y",
            "-i", str(path), "-ac", "1", "-ar", str(sample_rate),
            "-sample_fmt", "s16", tmp.name,
        ])
        return read_wav_mono(Path(tmp.name)).astype(np.float32)


def read_wav_mono(path: Path) -> np.ndarray:
    with wave.open(str(path), "rb") as wf:
        channels = wf.getnchannels()
        width = wf.getsampwidth()
        frames = wf.getnframes()
        raw = wf.readframes(frames)
    if width != 2:
        raise ValueError(f"Expected 16-bit wav, got {width * 8}-bit")
    data = np.frombuffer(raw, dtype="<i2").astype(np.float32) / 32768.0
    if channels > 1:
        data = data.reshape((-1, channels)).mean(axis=1)
    return data


def write_wav_mono(path: Path, samples: np.ndarray, sample_rate: int) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    clipped = np.clip(samples, -1.0, 1.0)
    pcm = (clipped * 32767.0).astype("<i2")
    with wave.open(str(path), "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(sample_rate)
        wf.writeframes(pcm.tobytes())


def median_filter_axis(values: np.ndarray, size: int, axis: int) -> np.ndarray:
    radius = size // 2
    moved = np.moveaxis(values, axis, 0)
    padded = np.pad(moved, [(radius, radius)] + [(0, 0)] * (moved.ndim - 1), mode="edge")
    out = np.empty_like(moved)
    for i in range(moved.shape[0]):
        out[i] = np.median(padded[i:i + size], axis=0)
    return np.moveaxis(out, 0, axis)


def stft(samples: np.ndarray) -> tuple[np.ndarray, int]:
    if len(samples) < FRAME:
        padded_len = FRAME
    else:
        frame_count = 1 + math.ceil((len(samples) - FRAME) / HOP)
        padded_len = (frame_count - 1) * HOP + FRAME
    padded = np.pad(samples, (0, padded_len - len(samples)))
    window = np.hanning(FRAME).astype(np.float32)
    frames = []
    for start in range(0, padded_len - FRAME + 1, HOP):
        frames.append(np.fft.rfft(padded[start:start + FRAME] * window))
    return np.asarray(frames), padded_len


def istft(spec: np.ndarray, padded_len: int, original_len: int) -> np.ndarray:
    window = np.hanning(FRAME).astype(np.float32)
    out = np.zeros(padded_len, dtype=np.float32)
    norm = np.zeros(padded_len, dtype=np.float32)
    for i, frame_spec in enumerate(spec):
        start = i * HOP
        frame = np.fft.irfft(frame_spec, n=FRAME).astype(np.float32)
        out[start:start + FRAME] += frame * window
        norm[start:start + FRAME] += window * window
    out /= np.maximum(norm, 1e-6)
    return out[:original_len]


def separate_stems(samples: np.ndarray) -> dict[str, np.ndarray]:
    spec, padded_len = stft(samples)
    mag = np.abs(spec).astype(np.float32)
    harmonic_ref = median_filter_axis(mag, 17, axis=0)
    percussive_ref = median_filter_axis(mag, 17, axis=1)
    denom = harmonic_ref * harmonic_ref + percussive_ref * percussive_ref + 1e-8
    harmonic_mask = (harmonic_ref * harmonic_ref) / denom
    percussive_mask = (percussive_ref * percussive_ref) / denom

    harmonic = istft(spec * harmonic_mask, padded_len, len(samples))
    percussive = istft(spec * percussive_mask, padded_len, len(samples))
    residual = samples - harmonic - percussive
    return {
        "harmonic": harmonic,
        "percussive": percussive,
        "residual": residual,
    }


def export_stems(bundle: Path, samples: np.ndarray) -> dict[str, str]:
    stem_dir = bundle / "stems"
    stem_dir.mkdir(parents=True, exist_ok=True)
    paths: dict[str, str] = {}
    for name, audio in separate_stems(samples).items():
        path = stem_dir / f"{name}.wav"
        write_wav_mono(path, audio, ANALYSIS_RATE)
        paths[name] = str(path.relative_to(bundle))
    return paths


def hz_to_midi(hz: float) -> float:
    if hz <= 0:
        return 0.0
    return 69.0 + 12.0 * math.log2(hz / 440.0)


def midi_to_hz(midi: float) -> float:
    return 440.0 * (2.0 ** ((midi - 69.0) / 12.0))


def quantize_hz(hz: float) -> float:
    if hz <= 0:
        return 0.0
    return midi_to_hz(round(hz_to_midi(hz)))


def note_name(hz: float) -> str:
    if hz <= 0:
        return "---"
    names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]
    midi = int(round(hz_to_midi(hz)))
    return f"{names[midi % 12]}{(midi // 12) - 1}"


def peak_in_band(freqs: np.ndarray, mag: np.ndarray, lo: float, hi: float) -> tuple[float, float]:
    mask = (freqs >= lo) & (freqs <= hi)
    if not np.any(mask):
        return 0.0, 0.0
    band_freqs = freqs[mask]
    band_mag = mag[mask]
    idx = int(np.argmax(band_mag))
    amp = float(band_mag[idx])
    if amp <= 1e-6:
        return 0.0, 0.0
    return float(band_freqs[idx]), amp


def smooth_track(values: list[float], max_jump_semitones: float = 9.0) -> list[float]:
    out: list[float] = []
    last = 0.0
    for hz in values:
        if hz > 0 and last > 0:
            jump = abs(hz_to_midi(hz) - hz_to_midi(last))
            if jump > max_jump_semitones:
                hz = last
        if hz > 0:
            last = hz
        out.append(hz)
    return out


def classify_event(bass_amp: float, lead_amp: float, noise_amp: float, transient: float, bass_hz: float, lead_hz: float) -> str:
    if transient > 0.55 and noise_amp > 0.25:
        return "drum"
    if bass_hz > 0 and bass_amp >= max(lead_amp, noise_amp) * 0.75 and bass_amp > 0.10:
        return "bass"
    if lead_hz > 0 and lead_amp > 0.12:
        return "tonal"
    if noise_amp > 0.22:
        return "noise"
    if transient > 0.30:
        return "sample"
    if max(bass_amp, lead_amp, noise_amp) > 0.08:
        return "ambience"
    return "silence"


def analyse(samples: np.ndarray, sample_rate: int = ANALYSIS_RATE) -> list[FrameEvent]:
    if len(samples) < FRAME:
        samples = np.pad(samples, (0, FRAME - len(samples)))
    window = np.hanning(FRAME).astype(np.float32)
    freqs = np.fft.rfftfreq(FRAME, 1.0 / sample_rate)
    events: list[FrameEvent] = []
    bass_raw: list[float] = []
    lead_raw: list[float] = []
    mags: list[np.ndarray] = []
    rms_values: list[float] = []

    for start in range(0, len(samples) - FRAME + 1, HOP):
        frame = samples[start:start + FRAME] * window
        spec = np.fft.rfft(frame)
        mag = np.abs(spec)
        mags.append(mag)
        rms_values.append(float(np.sqrt(np.mean(frame * frame))))
        bass_hz, _ = peak_in_band(freqs, mag, 38.0, 220.0)
        lead_hz, _ = peak_in_band(freqs, mag, 220.0, 1800.0)
        bass_raw.append(quantize_hz(bass_hz))
        lead_raw.append(quantize_hz(lead_hz))

    bass_track = smooth_track(bass_raw, 7.0)
    lead_track = smooth_track(lead_raw, 12.0)
    max_rms = max(max(rms_values), 1e-6)
    prev_mag = np.zeros_like(mags[0])

    for i, mag in enumerate(mags):
        t = (i * HOP) / sample_rate
        bass_hz = bass_track[i]
        lead_hz = lead_track[i]
        low_energy = float(np.sum(mag[(freqs >= 38.0) & (freqs <= 220.0)]))
        mid_energy = float(np.sum(mag[(freqs > 220.0) & (freqs <= 1800.0)]))
        high_energy = float(np.sum(mag[(freqs > 1800.0) & (freqs <= 8000.0)]))
        total = max(low_energy + mid_energy + high_energy, 1e-6)
        flux = float(np.sum(np.maximum(mag - prev_mag, 0.0)) / (np.sum(mag) + 1e-6))
        prev_mag = mag
        rms = rms_values[i] / max_rms
        bass_amp = min(1.0, rms * (low_energy / total) * 2.5)
        lead_amp = min(1.0, rms * (mid_energy / total) * 2.0)
        noise_amp = min(1.0, rms * (high_energy / total) * 2.8)
        transient = min(1.0, flux * 5.0)
        bass_hz = bass_hz if low_energy / total > 0.08 else 0.0
        lead_hz = lead_hz if mid_energy / total > 0.08 else 0.0
        events.append(FrameEvent(
            t=t,
            bass_hz=bass_hz,
            lead_hz=lead_hz,
            bass_amp=bass_amp,
            lead_amp=lead_amp,
            noise_amp=noise_amp,
            transient=transient,
            kind=classify_event(bass_amp, lead_amp, noise_amp, transient, bass_hz, lead_hz),
        ))
    return events


def compress_events(events: list[FrameEvent]) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    last_key: tuple[int, int, int, int, int] | None = None
    for ev in events:
        key = (
            int(round(hz_to_midi(ev.bass_hz))) if ev.bass_hz > 0 else 0,
            int(round(hz_to_midi(ev.lead_hz))) if ev.lead_hz > 0 else 0,
            int(round(ev.bass_amp * 15)),
            int(round(ev.lead_amp * 15)),
            int(round(max(ev.noise_amp, ev.transient) * 15)),
            CLASS_IDS.get(ev.kind, 0),
        )
        if key == last_key:
            rows[-1]["frames"] = int(rows[-1]["frames"]) + 1
            continue
        last_key = key
        rows.append({
            "t": round(ev.t, 4),
            "frames": 1,
            "bass_hz": round(ev.bass_hz, 2),
            "bass_note": note_name(ev.bass_hz),
            "lead_hz": round(ev.lead_hz, 2),
            "lead_note": note_name(ev.lead_hz),
            "bass_level": key[2],
            "lead_level": key[3],
            "dac_noise_level": key[4],
            "class": ev.kind,
        })
    return rows


def fm_voice(freq: float, amp: float, phase: np.ndarray, ratio: float, index: float) -> np.ndarray:
    if freq <= 0 or amp <= 0:
        return np.zeros_like(phase)
    mod = np.sin(phase * ratio) * index
    return np.sin(phase + mod) * amp


def synth_preview(events: list[FrameEvent], duration: float, out_rate: int = PREVIEW_RATE) -> np.ndarray:
    total = max(1, int(duration * out_rate))
    out = np.zeros(total, dtype=np.float32)
    frame_samples = max(1, int((HOP / ANALYSIS_RATE) * out_rate))
    rng = np.random.default_rng(0x56414e44)

    bass_phase = 0.0
    lead_phase = 0.0
    for i, ev in enumerate(events):
        start = i * frame_samples
        end = min(total, start + frame_samples)
        if start >= total:
            break
        n = end - start
        env = np.linspace(0.85, 1.0, n, dtype=np.float32)
        if ev.bass_hz > 0:
            inc = 2.0 * math.pi * ev.bass_hz / out_rate
            phase = bass_phase + np.arange(n, dtype=np.float32) * inc
            out[start:end] += fm_voice(ev.bass_hz, ev.bass_amp * 0.45, phase, 0.5, 2.4) * env
            bass_phase = float((phase[-1] + inc) % (2.0 * math.pi))
        if ev.lead_hz > 0:
            inc = 2.0 * math.pi * ev.lead_hz / out_rate
            phase = lead_phase + np.arange(n, dtype=np.float32) * inc
            out[start:end] += fm_voice(ev.lead_hz, ev.lead_amp * 0.32, phase, 2.0, 3.2) * env
            lead_phase = float((phase[-1] + inc) % (2.0 * math.pi))
        if ev.noise_amp > 0.05 or ev.transient > 0.1:
            noise = rng.uniform(-1.0, 1.0, n).astype(np.float32)
            decay = np.exp(-np.linspace(0.0, 4.0 if ev.transient > 0.25 else 1.0, n)).astype(np.float32)
            out[start:end] += noise * max(ev.noise_amp * 0.10, ev.transient * 0.20) * decay

    peak = max(float(np.max(np.abs(out))), 1e-6)
    return out / peak * 0.85


def contiguous_chunks(events: list[FrameEvent], predicate) -> list[dict[str, object]]:
    chunks: list[dict[str, object]] = []
    start: FrameEvent | None = None
    frames = 0
    peak = 0.0
    for ev in events:
        if predicate(ev):
            if start is None:
                start = ev
                frames = 0
                peak = 0.0
            frames += 1
            peak = max(peak, ev.noise_amp, ev.transient)
            continue
        if start is not None:
            chunks.append({
                "t": round(start.t, 4),
                "frames": frames,
                "duration": round(frames * HOP / ANALYSIS_RATE, 4),
                "peak": round(peak, 4),
            })
            start = None
    if start is not None:
        chunks.append({
            "t": round(start.t, 4),
            "frames": frames,
            "duration": round(frames * HOP / ANALYSIS_RATE, 4),
            "peak": round(peak, 4),
        })
    return chunks


def channel_events(events: list[FrameEvent], field: str) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    last: tuple[int, int] | None = None
    for ev in events:
        hz = ev.bass_hz if field == "bass" else ev.lead_hz
        amp = ev.bass_amp if field == "bass" else ev.lead_amp
        key = (int(round(hz_to_midi(hz))) if hz > 0 else 0, int(round(amp * 15)))
        if key == last:
            rows[-1]["frames"] = int(rows[-1]["frames"]) + 1
            continue
        last = key
        rows.append({
            "t": round(ev.t, 4),
            "frames": 1,
            "hz": round(hz, 2),
            "note": note_name(hz),
            "level": key[1],
        })
    return rows


def psg_noise_events(events: list[FrameEvent]) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    last: tuple[int, int] | None = None
    for ev in events:
        level = int(round(max(ev.noise_amp, ev.transient) * 15))
        mode = 1 if ev.kind in {"drum", "sample"} else 0
        key = (level, mode)
        if key == last:
            rows[-1]["frames"] = int(rows[-1]["frames"]) + 1
            continue
        last = key
        rows.append({"t": round(ev.t, 4), "frames": 1, "level": level, "white_noise": bool(mode)})
    return rows


def vgm_wait(stream: bytearray, samples: int) -> None:
    while samples > 0:
        chunk = min(samples, 65535)
        stream.extend((0x61, chunk & 0xFF, (chunk >> 8) & 0xFF))
        samples -= chunk


def ym_write(stream: bytearray, port: int, reg: int, value: int) -> None:
    stream.extend((0x52 if port == 0 else 0x53, reg & 0xFF, value & 0xFF))


def ym_channel_freq(hz: float) -> tuple[int, int]:
    if hz <= 0:
        return 0, 0
    best_block = 4
    best_fnum = 0
    best_err = float("inf")
    for block in range(1, 8):
        fnum = int(round(hz * (1 << 20) / (YM2612_CLOCK / 144.0) / (1 << (block - 1))))
        if 0x100 <= fnum <= 0x7FF:
            err = abs(fnum - 0x400)
            if err < best_err:
                best_err = err
                best_block = block
                best_fnum = fnum
    if best_fnum == 0:
        best_block = 1 if hz < 80 else 7
        best_fnum = max(0x100, min(0x7FF, int(round(hz * (1 << 20) / (YM2612_CLOCK / 144.0) / (1 << (best_block - 1))))))
    return best_block, best_fnum


def runtime_ticks(analysis_frames: int) -> int:
    seconds = max(1, analysis_frames) * HOP / ANALYSIS_RATE
    return max(1, int(round(seconds * RUNTIME_RATE)))


def ym_set_freq(stream: bytearray, channel: int, hz: float) -> None:
    block, fnum = ym_channel_freq(hz)
    ym_write(stream, 0, 0xA4 + channel, ((block & 7) << 3) | ((fnum >> 8) & 7))
    ym_write(stream, 0, 0xA0 + channel, fnum & 0xFF)


def ym_key(stream: bytearray, channel: int, on: bool) -> None:
    ym_write(stream, 0, 0x28, (0xF0 if on else 0x00) | channel)


def ym_set_level(stream: bytearray, channel: int, level: int, carrier_only: bool = True) -> None:
    base = [0x40, 0x44, 0x48, 0x4C]
    total_level = 127 - max(0, min(15, level)) * 7
    for op, reg in enumerate(base):
        value = total_level if (not carrier_only or op == 3) else 112
        ym_write(stream, 0, reg + channel, value)


def ym_init_voice(stream: bytearray, channel: int, *, bass: bool) -> None:
    ym_write(stream, 0, 0xB0 + channel, 0x32 if bass else 0x31)
    ym_write(stream, 0, 0xB4 + channel, 0xC0)
    detune_mul = (0x01, 0x02, 0x01, 0x01) if bass else (0x02, 0x03, 0x01, 0x01)
    rates = (0x1F, 0x18, 0x14, 0x16) if bass else (0x1F, 0x12, 0x10, 0x14)
    sustain = (0x08, 0x0A, 0x0C, 0x08)
    release = (0x0F, 0x0F, 0x0F, 0x0F)
    for op, offset in enumerate((0x00, 0x04, 0x08, 0x0C)):
        ym_write(stream, 0, 0x30 + offset + channel, detune_mul[op])
        ym_write(stream, 0, 0x50 + offset + channel, rates[op])
        ym_write(stream, 0, 0x60 + offset + channel, 0x05)
        ym_write(stream, 0, 0x70 + offset + channel, 0x02)
        ym_write(stream, 0, 0x80 + offset + channel, release[op])
        ym_write(stream, 0, 0x90 + offset + channel, sustain[op])
    ym_set_level(stream, channel, 0)


def export_vgm(path: Path, events: list[FrameEvent]) -> None:
    stream = bytearray()
    ym_write(stream, 0, 0x22, 0x00)
    ym_write(stream, 0, 0x27, 0x00)
    ym_write(stream, 0, 0x2B, 0x00)
    ym_init_voice(stream, 0, bass=True)
    ym_init_voice(stream, 1, bass=False)

    last_bass: tuple[int, int] = (0, 0)
    last_lead: tuple[int, int] = (0, 0)
    samples_per_event = int(round((HOP / ANALYSIS_RATE) * VGM_RATE))
    total_samples = 0
    for ev in events:
        bass_key = (int(round(hz_to_midi(ev.bass_hz))) if ev.bass_hz > 0 else 0, int(round(ev.bass_amp * 15)))
        lead_key = (int(round(hz_to_midi(ev.lead_hz))) if ev.lead_hz > 0 else 0, int(round(ev.lead_amp * 15)))
        if bass_key != last_bass:
            ym_key(stream, 0, False)
            if ev.bass_hz > 0 and bass_key[1] > 0:
                ym_set_freq(stream, 0, ev.bass_hz)
                ym_set_level(stream, 0, bass_key[1])
                ym_key(stream, 0, True)
            last_bass = bass_key
        if lead_key != last_lead:
            ym_key(stream, 1, False)
            if ev.lead_hz > 0 and lead_key[1] > 0:
                ym_set_freq(stream, 1, ev.lead_hz)
                ym_set_level(stream, 1, lead_key[1])
                ym_key(stream, 1, True)
            last_lead = lead_key
        vgm_wait(stream, samples_per_event)
        total_samples += samples_per_event
    ym_key(stream, 0, False)
    ym_key(stream, 1, False)
    stream.append(0x66)

    header = bytearray(0x100)
    header[0:4] = b"Vgm "
    eof_offset = len(header) + len(stream) - 4
    header[0x04:0x08] = eof_offset.to_bytes(4, "little")
    header[0x08:0x0C] = (0x00000170).to_bytes(4, "little")
    header[0x18:0x1C] = total_samples.to_bytes(4, "little")
    header[0x24:0x28] = VGM_RATE.to_bytes(4, "little")
    header[0x2C:0x30] = YM2612_CLOCK.to_bytes(4, "little")
    header[0x34:0x38] = (0x100 - 0x34).to_bytes(4, "little")
    path.write_bytes(header + stream)


def c_ident(name: str) -> str:
    out = []
    for ch in name.lower():
        if ch.isalnum():
            out.append(ch)
        else:
            out.append("_")
    ident = "".join(out).strip("_")
    if not ident:
        ident = "audio"
    if ident[0].isdigit():
        ident = f"audio_{ident}"
    while "__" in ident:
        ident = ident.replace("__", "_")
    return ident


def export_sgdk_tables(bundle: Path, stem: str, events: list[FrameEvent]) -> tuple[Path, Path]:
    name = c_ident(stem)
    symbol = f"vand_audio_{name}"
    header_path = bundle / "sgdk_audio.h"
    source_path = bundle / "sgdk_audio.c"
    rows = compress_events(events)

    header = f"""#ifndef VAND_AUDIO_{name.upper()}_H
#define VAND_AUDIO_{name.upper()}_H

#include "vand_audio.h"

extern const VandAudioEvent {symbol}_events[];
extern const u16 {symbol}_event_count;

#endif
"""

    lines = [
        f'#include "sgdk_audio.h"',
        "",
        f"const VandAudioEvent {symbol}_events[] =",
        "{",
    ]
    for row in rows:
        fm0_block, fm0_fnum = ym_channel_freq(float(row["bass_hz"]))
        fm1_block, fm1_fnum = ym_channel_freq(float(row["lead_hz"]))
        kind = CLASS_IDS.get(str(row["class"]), 0)
        lines.append(
            "    {"
            f"{runtime_ticks(int(row['frames']))}, "
            f"0x{fm0_fnum:03X}, {fm0_block}, {int(row['bass_level'])}, "
            f"0x{fm1_fnum:03X}, {fm1_block}, {int(row['lead_level'])}, "
            f"{int(row['dac_noise_level'])}, {kind}"
            "},"
        )
    lines.extend([
        "};",
        "",
        f"const u16 {symbol}_event_count = sizeof({symbol}_events) / sizeof({symbol}_events[0]);",
        "",
    ])

    header_path.write_text(header, encoding="utf-8")
    source_path.write_text("\n".join(lines), encoding="utf-8")
    return header_path, source_path


def export_vand_bin(bundle: Path, events: list[FrameEvent]) -> Path:
    path = bundle / "events.vandbin"
    rows = compress_events(events)
    data = bytearray()
    data.extend(b"VADB")
    data.extend((0).to_bytes(2, "little"))  # format version
    data.extend(len(rows).to_bytes(2, "little"))
    data.extend(HOP.to_bytes(2, "little"))
    data.extend(ANALYSIS_RATE.to_bytes(2, "little"))
    for row in rows:
        fm0_block, fm0_fnum = ym_channel_freq(float(row["bass_hz"]))
        fm1_block, fm1_fnum = ym_channel_freq(float(row["lead_hz"]))
        data.extend(min(65535, int(row["frames"])).to_bytes(2, "little"))
        data.extend(fm0_fnum.to_bytes(2, "little"))
        data.append(fm0_block & 0xFF)
        data.append(int(row["bass_level"]) & 0xFF)
        data.extend(fm1_fnum.to_bytes(2, "little"))
        data.append(fm1_block & 0xFF)
        data.append(int(row["lead_level"]) & 0xFF)
        data.append(int(row["dac_noise_level"]) & 0xFF)
        data.append(CLASS_IDS.get(str(row["class"]), 0) & 0xFF)
    path.write_bytes(data)
    return path


def write_debug_visualizer(path: Path, events: list[FrameEvent]) -> None:
    try:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
    except Exception:
        return

    times = [ev.t for ev in events]
    bass = [hz_to_midi(ev.bass_hz) if ev.bass_hz > 0 else np.nan for ev in events]
    lead = [hz_to_midi(ev.lead_hz) if ev.lead_hz > 0 else np.nan for ev in events]
    noise = [ev.noise_amp for ev in events]
    transient = [ev.transient for ev in events]
    classes = ["silence", "ambience", "noise", "sample", "drum", "bass", "tonal"]
    class_y = [classes.index(ev.kind) if ev.kind in classes else 0 for ev in events]

    fig, axes = plt.subplots(3, 1, figsize=(12, 7), sharex=True)
    axes[0].plot(times, bass, label="bass midi", color="#ff9d00")
    axes[0].plot(times, lead, label="lead midi", color="#55aaff")
    axes[0].set_ylabel("pitch")
    axes[0].legend(loc="upper right")
    axes[1].plot(times, noise, label="noise", color="#cccccc")
    axes[1].plot(times, transient, label="transient", color="#ff5555")
    axes[1].set_ylabel("energy")
    axes[1].legend(loc="upper right")
    axes[2].step(times, class_y, where="post", color="#88ff88")
    axes[2].set_yticks(range(len(classes)), classes)
    axes[2].set_xlabel("seconds")
    axes[2].set_ylabel("class")
    fig.tight_layout()
    path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(path)
    plt.close(fig)


def export_plan(source: Path, events: list[FrameEvent], out_dir: Path, duration: float, samples: np.ndarray) -> tuple[Path, Path]:
    out_dir.mkdir(parents=True, exist_ok=True)
    stem = source.stem.replace(" ", "_")
    bundle = out_dir / f"{stem}.vand-audio"
    bundle.mkdir(parents=True, exist_ok=True)
    plan_path = bundle / "arrangement.vand-audio.json"
    wav_path = bundle / "preview.wav"
    debug_path = bundle / "debug.png"
    vgm_path = bundle / "debug.vgm"
    sgdk_header, sgdk_source = bundle / "sgdk_audio.h", bundle / "sgdk_audio.c"
    vandbin_path = bundle / "events.vandbin"
    rows = compress_events(events)
    stem_exports = export_stems(bundle, samples)
    plan = {
        "format": "vandaler-vand-audio-v0",
        "source": str(source),
        "analysis_rate": ANALYSIS_RATE,
        "hop": HOP,
        "sgdk_runtime_rate": RUNTIME_RATE,
        "duration": round(duration, 4),
        "pipeline": [
            "pipe_decode_to_pcm",
            "source_separation_hpss_v0",
            "transient_detection",
            "pitch_tracking",
            "frame_classification",
            "fm_psg_dac_mapping",
        ],
        "mapping": {
            "fm": [
                {"channel": 0, "role": "bass", "file": "fm_bass.json"},
                {"channel": 1, "role": "lead", "file": "fm_lead.json"}
            ],
            "psg": [{"channel": "noise", "role": "brus_trumma", "file": "psg_noise.json"}],
            "dac": {"role": "chunks_for_transients_samples_ambience", "file": "dac_chunks.json"},
            "fallback_pcm": {"enabled": False, "reason": "keep first prototype as arranger not sample player"},
        },
        "debug_exports": {"vgm": "debug.vgm"},
        "stem_exports": stem_exports,
        "sgdk_exports": {"header": sgdk_header.name, "source": sgdk_source.name},
        "binary_exports": {"events": vandbin_path.name},
        "loop": {"start": 0.0, "end": round(duration, 4), "detected": False},
        "events": rows,
    }
    plan_path.write_text(json.dumps(plan, indent=2), encoding="utf-8")
    (bundle / "fm_bass.json").write_text(json.dumps(channel_events(events, "bass"), indent=2), encoding="utf-8")
    (bundle / "fm_lead.json").write_text(json.dumps(channel_events(events, "lead"), indent=2), encoding="utf-8")
    (bundle / "psg_noise.json").write_text(json.dumps(psg_noise_events(events), indent=2), encoding="utf-8")
    (bundle / "dac_chunks.json").write_text(json.dumps(contiguous_chunks(events, lambda ev: ev.kind in {"drum", "sample", "noise"}), indent=2), encoding="utf-8")
    preview = synth_preview(events, duration, PREVIEW_RATE)
    write_wav_mono(wav_path, preview, PREVIEW_RATE)
    export_vgm(vgm_path, events)
    export_sgdk_tables(bundle, stem, events)
    export_vand_bin(bundle, events)
    write_debug_visualizer(debug_path, events)
    return plan_path, wav_path


def analyse_file(path: Path, out_dir: Path = DEFAULT_OUT) -> tuple[Path, Path]:
    samples = decode_audio(path)
    duration = len(samples) / ANALYSIS_RATE
    events = analyse(samples)
    return export_plan(path, events, out_dir, duration, samples)


def play(path: Path) -> None:
    player = shutil.which("ffplay") or shutil.which("pw-play")
    if not player:
        raise SystemExit("No ffplay or pw-play found for playback")
    if Path(player).name == "ffplay":
        subprocess.Popen([player, "-hide_banner", "-loglevel", "error", "-autoexit", str(path)])
    else:
        subprocess.Popen([player, str(path)])


def launch_gui() -> None:
    import tkinter as tk
    from tkinter import filedialog, messagebox

    root = tk.Tk()
    root.title("Vandaler Mega Drive Audio Lab")
    root.geometry("720x360")
    selected = tk.StringVar()
    status = tk.StringVar(value="Open an MP3/OGG/WAV to build a YM/DAC arrangement preview.")
    last_preview: dict[str, Path] = {}

    def open_file() -> None:
        path = filedialog.askopenfilename(filetypes=[
            ("Audio", "*.mp3 *.ogg *.wav *.flac *.aiff *.aif"),
            ("All files", "*.*"),
        ])
        if path:
            selected.set(path)
            status.set("Ready.")

    def run_analysis() -> None:
        if not selected.get():
            messagebox.showwarning("No file", "Choose an audio file first.")
            return
        try:
            status.set("Analysing tonal bass/lead and DAC residual...")
            root.update_idletasks()
            plan, wav = analyse_file(Path(selected.get()))
            last_preview["plan"] = plan
            last_preview["wav"] = wav
            status.set(f"Wrote {plan.parent.name}: plan, split channels, debug VGM, preview WAV and visualizer")
        except Exception as exc:
            messagebox.showerror("Analysis failed", str(exc))
            status.set("Failed.")

    def play_original() -> None:
        if selected.get():
            play(Path(selected.get()))

    def play_preview() -> None:
        wav = last_preview.get("wav")
        if wav:
            play(wav)
        else:
            messagebox.showwarning("No preview", "Run analysis first.")

    tk.Label(root, text="Vandaler Mega Drive Audio Lab", font=("Sans", 18, "bold")).pack(pady=12)
    row = tk.Frame(root)
    row.pack(fill="x", padx=16)
    tk.Entry(row, textvariable=selected).pack(side="left", fill="x", expand=True)
    tk.Button(row, text="Open", command=open_file).pack(side="left", padx=6)

    buttons = tk.Frame(root)
    buttons.pack(pady=18)
    tk.Button(buttons, text="Play Original", command=play_original, width=16).pack(side="left", padx=6)
    tk.Button(buttons, text="Analyse/Export", command=run_analysis, width=16).pack(side="left", padx=6)
    tk.Button(buttons, text="Play MD Preview", command=play_preview, width=16).pack(side="left", padx=6)

    tk.Label(root, textvariable=status, wraplength=660, justify="left").pack(fill="x", padx=20, pady=8)
    tk.Label(root, text="v0 extracts rough FM bass/lead notes plus DAC-noise intent and exports a debug VGM. It is deliberately a transcriber, not a sampler.").pack(pady=10)
    root.mainloop()


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Prototype MP3/OGG/WAV to Mega Drive YM/DAC arrangement lab.")
    parser.add_argument("input", nargs="?", type=Path, help="audio file to analyse")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT, help="output directory")
    parser.add_argument("--gui", action="store_true", help="launch Tkinter GUI")
    parser.add_argument("--play", action="store_true", help="play generated preview")
    args = parser.parse_args(list(argv) if argv is not None else None)

    if args.gui:
        launch_gui()
        return 0
    if not args.input:
        parser.error("input is required unless --gui is used")
    plan, wav = analyse_file(args.input, args.out)
    print(plan)
    print(wav)
    if args.play:
        play(wav)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
