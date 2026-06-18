# Malmo Music Notes

## Source

- Track: "Endless Cyber Runner" / "Endless Cyber Runner (Looping)"
- Author: Eric Matyas
- Local source: `audio/source/endless_cyber_runner.mp3`
- Runtime export: `src/malmo_music.c` / `src/malmo_music.h`
- Runtime data source: generated `VandAudioEvent` rows from `tools/audio/audio_lab`

The in-game music must come from the generated event stream. Do not add a
hand-written stage melody in `src/main.c`; it becomes audible as a separate,
made-up loop instead of the converted cyberpunk track.

## Runtime hookup

`startCityMusic()` starts the generated Malmö arrangement only for
`currentCity == 0`. It installs the generated DAC bank with
`VandAudio_setDacBank()`, starts `malmoMusicEvents` with looping enabled, and
`updateCityMusic()` calls `VandAudio_update()` once per game tick while the
state is `STATE_PLAY`.

`stopTone()` leaves PSG channels 1 and 2 alone while `cityMusicPlaying` is true,
so short SFX cleanup does not silence the music carrier channels.

## Current runtime implementation

The first direct YM/DAC pass was technically active but too quiet in-game
because DAC bytes were pumped from the 60 Hz game update. Current runtime
playback uses SGDK's Z80 `SND_PCM4` driver for generated drum/sample chunks:

- `src/vand_audio.c` loads `SND_PCM4` during `VandAudio_init()`
- generated DAC chunks are exported as signed 8-bit, 16 kHz, 256-byte-aligned
  PCM4 samples
- `VandAudioEvent.dac_chunk` starts PCM4 playback on rotating PCM channels
- PSG noise remains only a light transient layer instead of carrying the drum
  body

This is deliberately a playback/mapping boost, not a replacement composition.
If the result still sounds wrong, improve the analyser/exporter so
`malmoMusicEvents` carries better pitch, rhythm and channel intent.
