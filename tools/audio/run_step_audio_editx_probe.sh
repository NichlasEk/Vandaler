#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  tools/audio/run_step_audio_editx_probe.sh INPUT_AUDIO [OUTPUT_BUNDLE_DIR]

Environment overrides:
  STEP_AUDIO_CODE=/home/nichlas/Step-Audio-EditX
  STEP_AUDIO_MODEL=/home/nichlas/models/Step-Audio-EditX
  STEP_AUDIO_CONDA_ENV=foundation1
  STEP_AUDIO_START=0
  STEP_AUDIO_SECONDS=3
  STEP_AUDIO_EDIT_TYPE=denoise
  STEP_AUDIO_PROMPT_TEXT="Instrumental TV theme music"
  STEP_AUDIO_GPU_MEMORY=18GiB
  STEP_AUDIO_CPU_MEMORY=48GiB
  STEP_AUDIO_MAX_TOKENS=512
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" || $# -lt 1 ]]; then
  usage
  exit 0
fi

INPUT_AUDIO=$1
if [[ ! -f "$INPUT_AUDIO" ]]; then
  echo "input audio not found: $INPUT_AUDIO" >&2
  exit 1
fi

STEP_AUDIO_CODE=${STEP_AUDIO_CODE:-/home/nichlas/Step-Audio-EditX}
STEP_AUDIO_MODEL=${STEP_AUDIO_MODEL:-/home/nichlas/models/Step-Audio-EditX}
STEP_AUDIO_CONDA_ENV=${STEP_AUDIO_CONDA_ENV:-foundation1}
STEP_AUDIO_START=${STEP_AUDIO_START:-0}
STEP_AUDIO_SECONDS=${STEP_AUDIO_SECONDS:-3}
STEP_AUDIO_EDIT_TYPE=${STEP_AUDIO_EDIT_TYPE:-denoise}
STEP_AUDIO_PROMPT_TEXT=${STEP_AUDIO_PROMPT_TEXT:-Instrumental TV theme music}
STEP_AUDIO_GPU_MEMORY=${STEP_AUDIO_GPU_MEMORY:-18GiB}
STEP_AUDIO_CPU_MEMORY=${STEP_AUDIO_CPU_MEMORY:-48GiB}
STEP_AUDIO_MAX_TOKENS=${STEP_AUDIO_MAX_TOKENS:-512}

if [[ ! -d "$STEP_AUDIO_CODE" ]]; then
  echo "Step-Audio-EditX code directory not found: $STEP_AUDIO_CODE" >&2
  exit 1
fi
if [[ ! -d "$STEP_AUDIO_MODEL/Step-Audio-EditX" || ! -d "$STEP_AUDIO_MODEL/Step-Audio-Tokenizer" ]]; then
  echo "Step-Audio-EditX model root must contain Step-Audio-EditX and Step-Audio-Tokenizer: $STEP_AUDIO_MODEL" >&2
  exit 1
fi

CONDA=${CONDA:-/opt/miniconda3/condabin/conda}
if [[ ! -x "$CONDA" ]]; then
  echo "conda not found or not executable: $CONDA" >&2
  exit 1
fi

stem=$(basename "$INPUT_AUDIO")
stem=${stem%.*}
if [[ $# -ge 2 ]]; then
  bundle_dir=$2
else
  bundle_dir="audio/converted/${stem}.vand-audio"
fi

out_dir="$bundle_dir/ai/step_audio_editx"
out_dir=$(realpath -m "$out_dir")
work_dir="/tmp/vandaler-step-audio-editx-probe"
probe_code="$work_dir/code"
probe_audio="$work_dir/${stem}-step-audio-probe.wav"
offload_dir="$work_dir/offload"
hf_cache="$work_dir/hf-cache"
numba_cache="$work_dir/numba-cache"
log_file="$out_dir/step_audio_editx.log"

mkdir -p "$out_dir" "$work_dir" "$offload_dir" "$hf_cache" "$numba_cache"
rm -rf "$probe_code"
mkdir -p "$probe_code"

cp -a \
  "$STEP_AUDIO_CODE/app.py" \
  "$STEP_AUDIO_CODE/config" \
  "$STEP_AUDIO_CODE/funasr_detach" \
  "$STEP_AUDIO_CODE/model_loader.py" \
  "$STEP_AUDIO_CODE/quantization" \
  "$STEP_AUDIO_CODE/stepvocoder" \
  "$STEP_AUDIO_CODE/tokenizer.py" \
  "$STEP_AUDIO_CODE/tts.py" \
  "$STEP_AUDIO_CODE/tts_infer.py" \
  "$STEP_AUDIO_CODE/utils.py" \
  "$STEP_AUDIO_CODE/whisper_wrapper.py" \
  "$probe_code/"

python - "$probe_code/tts.py" "$probe_code/model_loader.py" "$STEP_AUDIO_MAX_TOKENS" "$STEP_AUDIO_GPU_MEMORY" "$STEP_AUDIO_CPU_MEMORY" <<'PY'
from pathlib import Path
import sys

tts_path = Path(sys.argv[1])
loader_path = Path(sys.argv[2])
max_tokens = sys.argv[3]
gpu_memory = sys.argv[4]
cpu_memory = sys.argv[5]

tts = tts_path.read_text()
tts = tts.replace(
    "max_length=8192,",
    f"max_length=min(1024, len(token_ids) + {max_tokens}),",
    1,
)
tts = tts.replace(
    "max_length=8192,",
    f"max_length=min(1024, len(prompt_tokens) + {max_tokens}),",
    1,
)
tts_path.write_text(tts)

loader = loader_path.read_text()
needle = '''                load_kwargs = {
                    "device_map": kwargs.get("device_map", "auto"),
                    "trust_remote_code": True,
                    "local_files_only": True
                }
'''
replacement = needle + f'''                if load_kwargs["device_map"] == "auto":
                    load_kwargs["max_memory"] = {{0: "{gpu_memory}", "cpu": "{cpu_memory}"}}
                    load_kwargs["offload_folder"] = "{Path("/tmp/vandaler-step-audio-editx-probe/offload")}"
'''
if needle not in loader:
    raise SystemExit("could not patch model_loader.py: load_kwargs block not found")
loader = loader.replace(needle, replacement, 1)
loader_path.write_text(loader)
PY

ffmpeg -hide_banner -y \
  -ss "$STEP_AUDIO_START" \
  -t "$STEP_AUDIO_SECONDS" \
  -i "$INPUT_AUDIO" \
  -ar 16000 \
  -ac 1 \
  "$probe_audio"

run_cmd=(
  "$CONDA" run -n "$STEP_AUDIO_CONDA_ENV"
  python "$probe_code/tts_infer.py"
  --model-path "$STEP_AUDIO_MODEL"
  --model-source local
  --output-dir "$out_dir"
  --prompt-text "$STEP_AUDIO_PROMPT_TEXT"
  --prompt-audio-path "$probe_audio"
  --edit-type "$STEP_AUDIO_EDIT_TYPE"
  --torch-dtype float16
  --device-map auto
)

{
  echo "input=$INPUT_AUDIO"
  echo "probe_audio=$probe_audio"
  echo "output_dir=$out_dir"
  echo "step_audio_code=$STEP_AUDIO_CODE"
  echo "step_audio_model=$STEP_AUDIO_MODEL"
  echo "conda_env=$STEP_AUDIO_CONDA_ENV"
  echo "start=$STEP_AUDIO_START seconds=$STEP_AUDIO_SECONDS edit_type=$STEP_AUDIO_EDIT_TYPE"
  echo "gpu_memory=$STEP_AUDIO_GPU_MEMORY cpu_memory=$STEP_AUDIO_CPU_MEMORY max_tokens=$STEP_AUDIO_MAX_TOKENS"
  echo "command=${run_cmd[*]}"
} > "$log_file"

(
  cd "$probe_code"
  PROTOCOL_BUFFERS_PYTHON_IMPLEMENTATION=python \
  HF_HOME="$hf_cache" \
  TRANSFORMERS_CACHE="$hf_cache" \
  NUMBA_DISABLE_CACHE=1 \
  NUMBA_CACHE_DIR="$numba_cache" \
  "${run_cmd[@]}"
) 2>&1 | tee -a "$log_file"

output_wav=$(find "$out_dir" -maxdepth 1 -type f -name '*_edited_iter1.wav' -printf '%T@ %p\n' | sort -n | tail -1 | cut -d' ' -f2-)
if [[ -z "$output_wav" || ! -f "$output_wav" ]]; then
  echo "Step-Audio-EditX did not produce an edited wav; see $log_file" >&2
  exit 1
fi

manifest="$out_dir/manifest.json"
cat > "$manifest" <<EOF
{
  "format": "vandaler-step-audio-editx-probe-v0",
  "source_audio": "$INPUT_AUDIO",
  "probe_audio": "$probe_audio",
  "output_wav": "$output_wav",
  "step_audio_code": "$STEP_AUDIO_CODE",
  "step_audio_model": "$STEP_AUDIO_MODEL",
  "conda_env": "$STEP_AUDIO_CONDA_ENV",
  "start_seconds": $STEP_AUDIO_START,
  "duration_seconds": $STEP_AUDIO_SECONDS,
  "edit_type": "$STEP_AUDIO_EDIT_TYPE",
  "prompt_text": "$STEP_AUDIO_PROMPT_TEXT",
  "gpu_memory": "$STEP_AUDIO_GPU_MEMORY",
  "cpu_memory": "$STEP_AUDIO_CPU_MEMORY",
  "max_tokens": $STEP_AUDIO_MAX_TOKENS,
  "log": "$log_file"
}
EOF

echo "wrote Step-Audio-EditX guide wav: $output_wav"
echo "wrote manifest: $manifest"
