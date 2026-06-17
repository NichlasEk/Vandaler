#!/usr/bin/env sh
set -eu

AI_HOME="${AI_HOME:-/home/nichlas/ai}"
VENV_DIR="${VAND_AI_LISM_VENV:-$AI_HOME/vand-ai-lism}"

find_python() {
  for candidate in "${PYTHON:-}" python3.11 python3.10 python3.9; do
    if [ -n "$candidate" ] && command -v "$candidate" >/dev/null 2>&1; then
      if "$candidate" - <<'PY'
import sys
raise SystemExit(0 if (3, 9) <= sys.version_info[:2] <= (3, 11) else 1)
PY
      then
        command -v "$candidate"
        return 0
      fi
    fi
  done
  return 1
}

PYTHON_BIN="$(find_python || true)"

mkdir -p "$AI_HOME/cache" "$AI_HOME/huggingface" "$AI_HOME/torch" "$AI_HOME/uv-cache"

if [ -n "$PYTHON_BIN" ]; then
  "$PYTHON_BIN" -m venv "$VENV_DIR"
elif command -v uv >/dev/null 2>&1; then
  UV_CACHE_DIR="$AI_HOME/uv-cache" uv venv --python 3.11 --seed --allow-existing "$VENV_DIR"
else
  echo "No supported Python found. Basic Pitch supports Python 3.7-3.11; install python3.11, set PYTHON=/path/to/python, or install uv." >&2
  exit 1
fi

XDG_CACHE_HOME="$AI_HOME/cache" \
HF_HOME="$AI_HOME/huggingface" \
TORCH_HOME="$AI_HOME/torch" \
  "$VENV_DIR/bin/python" -m pip install --upgrade pip setuptools wheel

XDG_CACHE_HOME="$AI_HOME/cache" \
HF_HOME="$AI_HOME/huggingface" \
TORCH_HOME="$AI_HOME/torch" \
  "$VENV_DIR/bin/python" -m pip install basic-pitch demucs

cat <<EOF
Vand-AI-lism AI tools installed.

Add this to tools/audio/vand_ai_lism/settings.toml:

ai_home = "$AI_HOME"
ai_tool_dir = "$VENV_DIR/bin"

Available commands:
  $VENV_DIR/bin/basic-pitch
  $VENV_DIR/bin/demucs
EOF
