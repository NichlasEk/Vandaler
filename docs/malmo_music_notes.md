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

## Current level implementation

The first direct YM/DAC pass was technically active but too quiet in-game. The
current level fix is in `src/vand_audio.c`:

- generated FM bass intent is mirrored to PSG channel 1
- generated FM lead intent is mirrored to PSG channel 2
- PSG levels are boosted by `+7` and clamped to `15`
- the last PSG tone is held for `10` update ticks when an analysis frame has no
  usable pitch, which makes the converted line less hesitant
- PSG noise still uses channel 3 for generated noise/drum/sample intent

This is deliberately a playback/mapping boost, not a replacement composition.
If the result still sounds wrong, improve the analyser/exporter so
`malmoMusicEvents` carries better pitch, rhythm and channel intent.
