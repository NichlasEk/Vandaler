# Mega Drive Audio Lab

Goal: build a Vandaler-specific audio transcriber that turns ordinary source
audio into a Mega Drive arrangement instead of only storing it as a sample.

The hard path is intentional:

```text
pipe/decode to PCM
  -> source separation / transient detection / pitch tracking
  -> classify: tonal / bass / noise / drum / sample / ambience
  -> map to FM channels, PSG square/noise, DAC chunks, fallback PCM
  -> export .ym/.vgm or .vand-audio plus split instrument files
  -> debug visualizer
```

The prototype starts with the `.vand-audio` path because it can preserve our
analysis decisions before we commit to a final SGDK playback format.

Current prototype:

```sh
python tools/audio/md_audio_lab.py --gui
python tools/audio/md_audio_lab.py path/to/song.mp3 --play
make audio-test
make audio-lab
cargo run --manifest-path tools/audio/oxide_probe/Cargo.toml -- render-rom out/audio-test.bin --wav out/audio-test.wav --seconds 5
```

It writes a bundle:

- `audio/converted/<name>.vand-audio/arrangement.vand-audio.json`
- `audio/converted/<name>.vand-audio/fm_bass.json`
- `audio/converted/<name>.vand-audio/fm_lead.json`
- `audio/converted/<name>.vand-audio/psg_noise.json`
- `audio/converted/<name>.vand-audio/dac_chunks.json`
- `audio/converted/<name>.vand-audio/debug.vgm`
- `audio/converted/<name>.vand-audio/events.vandbin`
- `audio/converted/<name>.vand-audio/sgdk_audio.h`
- `audio/converted/<name>.vand-audio/sgdk_audio.c`
- `audio/converted/<name>.vand-audio/stems/harmonic.wav`
- `audio/converted/<name>.vand-audio/stems/percussive.wav`
- `audio/converted/<name>.vand-audio/stems/residual.wav`
- `audio/converted/<name>.vand-audio/preview.wav`
- `audio/converted/<name>.vand-audio/debug.png`

The generated SGDK header uses `src/vand_audio.h`. Copy or include the
generated `sgdk_audio.c/.h` in a test target, call `VandAudio_start()` with the
exported table, then call `VandAudio_update()` once per game tick. Generated C
tables convert analysis frames to 60 Hz runtime ticks.

The JSON format is deliberately not final. Version `vandaler-vand-audio-v0`
describes frame runs with classification, FM note intent, PSG noise intent, DAC
chunk candidates and loop metadata. The preview synth is a rough FM-style
approximation so we can evaluate transcription quality before writing a real
YM2612/SN76489/DAC renderer.

The first separator is an offline HPSS-style pass built from numpy STFT masks:
stable horizontal spectral content becomes `stems/harmonic.wav`, transient
vertical content becomes `stems/percussive.wav`, and leftovers become
`stems/residual.wav`. It is intentionally dependency-light so the tool can run
before we decide whether to add a heavier local model.

Next milestones:

1. Add licensed source asset import and attribution metadata.
2. Improve the YM2612 VGM exporter with real voice search and noise/DAC events.
3. Add `.ym` export for debug and comparison.
4. Add a tiny audio test ROM around the SGDK runtime player.
5. Add optional heavy stem separation when a local model is available.
6. Add DAC sample extraction and playback for drums/bass transients.

`debug.vgm` is intentionally a listening/debug artifact. It writes YM2612
registers for two channels from the bass and lead tracks so the transcription can
be auditioned in VGM tools before a game runtime exists.

`events.vandbin` is the compact runtime candidate. It starts with `VADB`, format
version, event count, hop size and analysis rate, followed by packed per-event
FM/PSG/DAC intent rows.

`src/vand_audio.c` is the first runtime side of this: it consumes the generated
`VandAudioEvent` rows and writes YM2612 bass/lead plus PSG noise intent.

The Rust lab under `tools/audio/oxide_probe` builds the `vandaler-audio-lab`
binary and uses the local `/home/nichlas/EutherOxide` Rust Mega Drive core as
its first host backend. `render-rom` loads a ROM, runs frames, renders stereo
i16 audio, writes a WAV, and can hand it to PipeWire with `--play`.
