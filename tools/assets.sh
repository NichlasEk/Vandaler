#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."

case "${1:-build}" in
    extract)
        python tools/extract_initial_sources.py
        ;;
    build)
        python tools/build_sprites.py
        ;;
    preview)
        python tools/build_sprites.py --preview art/source/vandaler_converted_preview.png
        ;;
    *)
        echo "usage: tools/assets.sh [extract|build|preview]" >&2
        exit 2
        ;;
esac
