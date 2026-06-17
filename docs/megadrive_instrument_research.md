# Mega Drive Instrument Research

Vand-AI-lism should build its own curated instrument bank instead of copying a
tracker runtime. The useful external work is mostly data formats and patch
libraries.

## Good First Source: Furnace

Furnace has chip-organized instruments:

- `instruments/OPN` maps to the Mega Drive YM2612/OPN FM side.
- `instruments/SN7` maps to the Mega Drive SN76489 PSG side.

The Furnace codebase is GPL, so do not vendor or port runtime code into
Vandaler. The instrument collection is still useful because its instrument
README asks contributors to submit instruments that are free of restrictions
and usable in any project. Treat imported files as third-party data: keep source
paths, attribution, and a license-review note with every curated patch.

Relevant links:

- https://github.com/tildearrow/furnace
- https://raw.githubusercontent.com/tildearrow/furnace/master/instruments/README.md
- https://github.com/tildearrow/furnace/tree/master/instruments/OPN
- https://github.com/tildearrow/furnace/tree/master/instruments/SN7
- https://github.com/tildearrow/furnace/blob/master/papers/newIns.md

## Other Useful Format References

`deflemask-preset-viewer` is a small MIT-licensed reference for reading DMP,
WOPN and TFI FM preset files. It is useful for cross-checking fields and
operator order, but the Vandaler tool should keep its own Rust importer.

`dmp2mml` is public domain and can help validate DefleMask DMP parsing.

`libOPNMIDI` and `DMXOPN2` contain larger WOPN banks. They are useful for
audition and comparison, but banks need per-file license review before any data
is committed. Some banks are derived from historic VGM/GEMS or MIDI-to-VGM
sources, so they should be treated as experimental until proven clean.

Relevant links:

- https://github.com/rhargreaves/deflemask-preset-viewer
- https://github.com/SilSinn9801/dmp2mml
- https://github.com/Wohlstand/libOPNMIDI
- https://github.com/sneakernets/DMXOPN2

## Import Strategy

The lab owns a neutral JSON schema:

- `format`: `vandaler-instrument-v0`
- `chip`: `ym2612`, `sn76489`, or `dac`
- `category`: `bass`, `lead`, `pad`, `noise`, `drum`, `sfx`, or `uncurated`
- `role`: provisional playback role such as `bass`, `lead`, `hook`, `pad`,
  `chord`, `drum`, or `psg`
- `tags`: searchable traits such as `low`, `aggressive`, `melodic`, `short`,
  `bright`, `sustained`, `chord`, and `percussive`
- chip-specific patch data
- source and license metadata

First milestone:

```sh
cargo run --manifest-path tools/audio/audio_lab/Cargo.toml -- inspect-instrument /tmp/patch.fui
cargo run --manifest-path tools/audio/audio_lab/Cargo.toml -- import-instrument /tmp/patch.fui
cargo run --manifest-path tools/audio/audio_lab/Cargo.toml -- import-instrument-dir /tmp/furnace/instruments/OPN/bass --out /tmp/vand-bass-bank
```

This only imports one Furnace `.fui` at a time and marks it
`pending_curated_import`. `import-instrument-dir` can scan a local Furnace
folder, classify patches from folder names, and write a temporary
`bank.vand-instruments.json` manifest. A later curator command should download
known Furnace OPN/SN7 subsets, write attribution, and emit a stable bank under
`audio/instruments/`.

## Curated Core Bank

The first stable bank lives at
`audio/instruments/vand_furnace_core/bank.vand-instruments.json`. It contains
six converted Furnace `.fui` patches:

- YM2612 bass: `growl_bass_wobbly`, `neatguitarmultialg_alt1`
- YM2612 lead: `fm_grinder`, `growl`
- SN76489/PSG: `psg_echo_warble`, `octave_arp`

Each generated instrument JSON keeps the original raw Furnace URL and the bank
has an `ATTRIBUTION.md` file next to it. The current automatic arrangement pass
uses `growl_bass_wobbly` for bass frames, `fm_grinder` for lead frames, and
`psg_echo_warble` for noise/transient frames.

## Audio Pipeline Impact

The current analyser maps detected bass and lead notes to generic runtime
voices. That is why output can sound arbitrary even when pitch tracking is
working. The next musical step is to let analysis events carry an instrument id:

- bass frames choose a curated YM2612 bass patch.
- tonal frames choose a YM2612 lead/pad patch.
- transient/noise frames choose SN76489 noise macros or DAC chunks.
- ambiguous ambience falls back to low-volume FM/PSG texture.

The GUI should then preview the generated arrangement through the same event
stream and instrument bank that the ROM uses. DAC preview alone is only a sample
extract preview and will not match the in-game arrangement.

Status: the JSON exports now carry `instrument_bank` plus per-frame
`bass_instrument_id`, `lead_instrument_id`, `noise_instrument_id`, and split
track `instrument_id` fields. The SGDK binary/runtime event format still uses
the old hardcoded voices; the next runtime step is to load these IDs into a
deterministic YM/PSG patch switcher.

Imported Furnace instruments are now assigned provisional `role` and `tags`
from folder names, patch names and basic FM parameters. The Tauri previewer uses
those fields when a requested instrument id is missing or unsuitable for a track,
so larger temporary banks can be auditioned without hand-picking every event.
Older curated instruments without these fields still load through defaults; a
future curation pass should regenerate the committed core bank with explicit
roles and tags after license review.
