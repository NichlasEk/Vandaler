# Vand-AI-lism AI Toolchain Plan

Checkpoint date: 2026-06-17

This is the restart plan for the Vand-AI-lism AI music toolchain if the machine
freezes again.

## Current State

- The deterministic Rust audio analyser still works and remains the baseline.
- The Tauri app has an `AI analysis` selector with `Off`, `Basic Pitch`, and
  `Demucs + Basic Pitch`.
- AI settings are persisted in `tools/audio/vand_ai_lism/settings.toml`.
- The intended local model/tool root is `/home/nichlas/ai`.
- Step-Audio-EditX is being evaluated as an optional guide/reference pass from
  local model files under `/home/nichlas/models/Step-Audio-EditX`.
- The app now separates:
  - `ai_home`: cache/model root, default `/home/nichlas/ai`
  - `ai_tool_dir`: explicit executable directory, recommended
    `/home/nichlas/ai/vand-ai-lism/bin`
- Online model inventory is in `docs/ai_music_model_inventory.md`.

## Current Local Environment

- `/home/nichlas/ai` exists.
- `uv` exists at `/home/nichlas/.local/bin/uv`.
- System `python` is Python 3.14.
- No `python3.11`, `python3.10`, or `python3.12` was found on PATH.
- Basic Pitch upstream lists Python 3.7-3.11 support, so do not build the AI
  venv with Python 3.14.
- `/home/nichlas/ai/vand-ai-lism` has been created with uv-managed CPython
  3.11.14 and contains `basic-pitch` plus `demucs`.
- Current installed size after first setup: about 6.7 GB for the venv and 3.3 GB
  for `/home/nichlas/ai/cache`.

## Target Layout

```text
/home/nichlas/ai/
  vand-ai-lism/       # Python venv with basic-pitch and demucs
  cache/              # XDG_CACHE_HOME
  huggingface/        # HF_HOME
  torch/              # TORCH_HOME
```

Recommended settings:

```toml
ai_backend = "off"
ai_home = "/home/nichlas/ai"
ai_tool_dir = "/home/nichlas/ai/vand-ai-lism/bin"
```

Switch `ai_backend` to `basic-pitch` or `demucs-basic-pitch` after the venv is
installed.

## Implementation Plan

1. Done: make `tools/audio/setup_vand_ai_lism_ai.sh` able to use `uv` to provision a
   supported Python, preferably 3.11, when no suitable Python exists on PATH.
2. Done: run the setup helper so it creates `/home/nichlas/ai/vand-ai-lism` and
   installs `basic-pitch` plus `demucs`.
3. Done: verify the installed tools:

   ```sh
   /home/nichlas/ai/vand-ai-lism/bin/basic-pitch --version
   /home/nichlas/ai/vand-ai-lism/bin/demucs --help
   ```

   Note: `basic-pitch --version` is not a supported option, but the command
   starts and prints usage. Practical smoke passed with:

   ```sh
   /home/nichlas/ai/vand-ai-lism/bin/basic-pitch --save-midi --save-note-events /tmp/vandaler-basic-pitch-smoke audio/source/endless_cyber_runner.mp3
   ```

   Output:

   ```text
   /tmp/vandaler-basic-pitch-smoke/endless_cyber_runner_basic_pitch.mid
   /tmp/vandaler-basic-pitch-smoke/endless_cyber_runner_basic_pitch.csv
   ```

4. In progress: run Vand-AI-lism analysis with `ai_backend = "basic-pitch"` on a known local
   source track and confirm it writes:

   ```text
   audio/converted/<name>.vand-audio/ai/manifest.json
   audio/converted/<name>.vand-audio/ai/basic_pitch/
   ```

5. Run `demucs-basic-pitch` once and confirm it writes `ai/stems/` plus the
   Basic Pitch sidecar output.
6. In progress: parse Basic Pitch note output into Vand-AI-lism AI note
   sidecar JSON. The parser writes `ai/basic_pitch/ai_notes.vand-ai-notes.json`
   from Basic Pitch `*_basic_pitch.csv`.
7. In progress: show AI notes alongside deterministic Goertzel notes in the
   debug view. The UI now draws `ai_basic_pitch` as an extra debug track and
   includes AI note counts in debug metadata.
8. In progress: render Basic Pitch sidecar notes as a separate AI Preview player.
   It converts the AI note JSON into a conservative FM arrangement with separate
   bass, pad, chord, lead, and counter roles instead of one lead-only line. It
   also synthesizes drum events from low-note/onset accents plus a 16th-note
   grid so AI Preview is no longer rhythmically empty. The preview renderer
   keeps PSG noise low and mixes deterministic PCM-style kick/snare/hat fallback
   samples when no DAC chunk is available. When `dac_chunks.json` exists in the
   bundle, AI and Hybrid previews classify those chunks into a small kick/snare/
   hat palette and attach real DAC sample paths to generated drum events.
9. In progress: use `Preview source` in the UI to choose `Rust`, `AI`, or
   `Hybrid`. `Hybrid` keeps Rust bass/drums/PSG and replaces Rust lead-style
   notes with selected Basic Pitch pad/chord/lead/counter notes. Hybrid also
   merges in the AI PSG drum layer and a dense drum grid on top of Rust drums.
10. In progress: make Basic Pitch notes Mega Drive-friendly before rendering.
    The current selector picks limited bass, pad, chord, lead, and counter roles
    in short windows. It prefers melodic notes for lead, strong low notes for
    bass, sustained mid-register notes for pad/chord, and high notes that do not
    closely duplicate the lead for counter.
11. Only after the Basic Pitch/Demucs path is stable, evaluate MT3 or Omnizart.
12. In progress: evaluate Step-Audio-EditX as a source-guided reference pass
    for short original-theme excerpts. This is not a runtime renderer. It is a
    listening and extraction aid for understanding the target sound, especially
    drum/body/timbre, before mapping the result back into YM2612/PSG/DAC rules.
    The local probe runner is:

   ```sh
   tools/audio/run_step_audio_editx_probe.sh 'audio/source/MacGyver - 1985.mp3'
   ```

   It copies the local Step-Audio-EditX code to `/tmp`, caps generation length,
   enables CPU offload, and writes ignored artifacts under:

   ```text
   audio/converted/<name>.vand-audio/ai/step_audio_editx/
   ```

## Verification Commands

```sh
sh -n tools/audio/setup_vand_ai_lism_ai.sh
cargo check --manifest-path tools/audio/audio_lab/Cargo.toml
cargo check --manifest-path tools/audio/vand_ai_lism/src-tauri/Cargo.toml
cargo test --manifest-path tools/audio/vand_ai_lism/src-tauri/Cargo.toml
cargo run --manifest-path tools/audio/audio_lab/Cargo.toml -- analyse-audio audio/source/endless_cyber_runner.mp3 --out /tmp/vandaler-ai-smoke/arrangement.vand-audio.json
tools/audio/run_step_audio_editx_probe.sh 'audio/source/MacGyver - 1985.mp3'
```

## Notes

- Keep downloaded models and caches out of the repo under `/home/nichlas/ai`.
- Keep generated `.vand-audio` bundles ignored unless a specific fixture is
  intentionally added.
- Do not replace the deterministic analyser with AI output immediately. Use AI
  sidecars first, compare in the debug UI, then feed selected notes into the
  arranger once the output is explainable.
- Step-Audio-EditX currently needs a CUDA-visible Python environment with
  `torch`, `funasr`, `onnxruntime`, `omegaconf`, `hyperpyyaml`, `diffusers`,
  `openai-whisper`, and `sox`. On this machine the tested environment is the
  `foundation1` conda env, but a dedicated clean env is preferable later because
  these dependencies can conflict with unrelated audio packages.
