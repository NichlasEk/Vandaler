# AI Music Model Inventory

Checked: 2026-06-17

Purpose: choose local models/tools for Vand-AI-lism, where source audio is
converted into Mega Drive-friendly note, stem and timing intent. Keep downloaded
models and caches under `/home/nichlas/ai`.

## Recommended First Stack

Use Basic Pitch plus Demucs first.

Basic Pitch is the best first transcription backend for Vand-AI-lism because it
is lightweight, installable as a Python package, has a CLI, accepts normal audio
formats, emits MIDI, and is instrument-agnostic. Its own README says it works
best on one instrument at a time, so it pairs naturally with stem separation.

Demucs is the best first separator because it is established, has a simple CLI,
and separates a mastered track into broad stems. For our use, the stems are not
the final game audio; they are analysis inputs that can make pitch/drum/bass
extraction less confused.

Local layout:

```text
/home/nichlas/ai/
  vand-ai-lism/       # Python venv for Basic Pitch/Demucs tools
  cache/              # XDG_CACHE_HOME
  huggingface/        # HF_HOME
  torch/              # TORCH_HOME
```

TOML settings:

```toml
ai_backend = "basic-pitch"
ai_home = "/home/nichlas/ai"
ai_tool_dir = "/home/nichlas/ai/vand-ai-lism/bin"
```

Setup helper:

```sh
sh tools/audio/setup_vand_ai_lism_ai.sh
```

The helper creates `/home/nichlas/ai/vand-ai-lism` and installs `basic-pitch`
plus `demucs` there. It requires Python 3.9-3.11; the current system
`python` may be newer than Basic Pitch supports.

## Candidates

| Tool/model | Use | Fit now | Notes |
| --- | --- | --- | --- |
| Basic Pitch | Audio-to-MIDI / note events | High | First choice for MIDI-side notes. CLI shape is `basic-pitch <output-dir> <input-audio>`. Upstream lists Python 3.7-3.11 support, so do not create this venv with system Python 3.14. |
| Demucs | Stem separation | High | First choice for optional `demucs-basic-pitch`. Use as sidecar stems first; later feed bass/drums/other into different arrangers. |
| Omnizart | Notes, drums, chords, beat | Medium later | Broader MIR toolbox with checkpoints, but heavier and more dependency-sensitive. Interesting after Basic Pitch/Demucs is stable. |
| MT3 | Multi-instrument transcription | Medium later | Strong research direction for multi-track transcription, but heavier than the first stack and likely needs more glue around checkpoints/runtime. |
| ByteDance piano_transcription | Piano-specific transcription | Low for Vandaler | Useful reference for piano material, but too narrow for mixed arcade soundtrack conversion. Repo was archived on 2025-12-08. |
| note-seq | MIDI/NoteSequence utilities | Medium later | Useful if we ingest MIDI/CSV outputs into a richer symbolic representation before mapping to YM/PSG/DAC. Not itself an audio model. |
| Essentia models | MIR classifiers/extractors | Medium later | Useful for key/BPM/instrument classifiers, but not the first path for note extraction. |

## Integration Plan

1. Keep the deterministic Rust analyser as the always-available baseline.
2. Let `basic-pitch` write MIDI/CSV/NPZ sidecars under
   `audio/converted/<name>.vand-audio/ai/basic_pitch/`.
3. Let `demucs-basic-pitch` write stems under
   `audio/converted/<name>.vand-audio/ai/stems/`, then run Basic Pitch on the
   source or selected stems.
4. Convert Basic Pitch note CSV/MIDI into Vand-AI-lism note events.
5. Compare AI note events against the existing Goertzel notes in the debug view.
6. Only after that, evaluate Omnizart or MT3 for chord/drum/multitrack
   improvement.

## Sources

- Basic Pitch: https://github.com/spotify/basic-pitch
- Demucs: https://github.com/facebookresearch/demucs
- MT3: https://github.com/magenta/mt3
- Omnizart: https://github.com/Music-and-Culture-Technology-Lab/omnizart
- ByteDance piano transcription: https://github.com/bytedance/piano_transcription
- note-seq: https://github.com/magenta/note-seq
- Essentia models: https://essentia.upf.edu/models.html
