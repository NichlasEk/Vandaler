#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SPEC = ROOT / "art/source/backgrounds/malmo_stage_tiles.json"
DEFAULT_HEADER = ROOT / "src/stage_tiles.h"
DEFAULT_PREVIEW = ROOT / "art/source/backgrounds/malmo_stage_tiles_preview.png"
DEFAULT_REPORT = ROOT / "art/source/backgrounds/malmo_stage_tiles_report.json"


def rgb333_to_rgb(rgb: list[int]) -> tuple[int, int, int]:
    if len(rgb) != 3 or any(channel < 0 or channel > 7 for channel in rgb):
        raise ValueError(f"invalid RGB333 color: {rgb}")
    return tuple(round((channel / 7) * 255) for channel in rgb)  # type: ignore[return-value]


def validate_tile(name: str, rows: list[str]) -> None:
    if len(rows) != 8:
        raise ValueError(f"{name}: expected 8 rows, got {len(rows)}")
    for row in rows:
        if len(row) != 8:
            raise ValueError(f"{name}: expected 8 columns, got {len(row)} in {row!r}")
        for ch in row:
            if ch not in "0123456789ABCDEF":
                raise ValueError(f"{name}: invalid palette index {ch!r}")


def tile_rows_to_u32(rows: list[str]) -> list[str]:
    return [f"0x{row}" for row in rows]


def write_header(spec: dict[str, Any], out: Path) -> None:
    palette = spec["palette"]
    tiles: dict[str, list[str]] = spec["tiles"]
    out.parent.mkdir(parents=True, exist_ok=True)
    lines: list[str] = [
        "#ifndef VANDALER_STAGE_TILES_H",
        "#define VANDALER_STAGE_TILES_H",
        "",
        "static const u16 palette0[16] =",
        "{",
    ]
    for entry in palette:
        r, g, b = entry["rgb333"]
        lines.append(f"    RGB3_3_3_TO_VDPCOLOR({r},{g},{b}),")
    lines.extend(["};", ""])
    for name, rows in tiles.items():
        validate_tile(name, rows)
        lines.append(f"static const u32 {name}[8] = {{{','.join(tile_rows_to_u32(rows))}}};")
    lines.extend(["", "#endif"])
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")


def draw_preview(spec: dict[str, Any], out: Path) -> None:
    palette = [rgb333_to_rgb(entry["rgb333"]) for entry in spec["palette"]]
    tiles: dict[str, list[str]] = spec["tiles"]
    scale = 6
    label_h = 12
    cols = 5
    cell_w = (8 * scale) + 18
    cell_h = (8 * scale) + label_h + 14
    rows = (len(tiles) + cols - 1) // cols
    canvas = Image.new("RGBA", (cols * cell_w + 10, rows * cell_h + 10), (24, 24, 32, 255))
    draw = ImageDraw.Draw(canvas)
    for i, (name, tile_rows) in enumerate(tiles.items()):
        validate_tile(name, tile_rows)
        ox = 6 + (i % cols) * cell_w
        oy = 6 + (i // cols) * cell_h
        draw.text((ox, oy), name.replace("tile", ""), fill=(240, 240, 240, 255))
        for y, row in enumerate(tile_rows):
            for x, ch in enumerate(row):
                color = palette[int(ch, 16)]
                draw.rectangle(
                    (
                        ox + x * scale,
                        oy + label_h + y * scale,
                        ox + (x + 1) * scale - 1,
                        oy + label_h + (y + 1) * scale - 1,
                    ),
                    fill=color + (255,),
                )
        draw.rectangle((ox, oy + label_h, ox + 8 * scale, oy + label_h + 8 * scale), outline=(80, 80, 96, 255))
    out.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(out)


def write_report(spec: dict[str, Any], out: Path) -> None:
    tiles: dict[str, list[str]] = spec["tiles"]
    report = {
        "palette_colors": len(spec["palette"]),
        "tile_count": len(tiles),
        "unique_tile_count": len({tuple(rows) for rows in tiles.values()}),
        "tiles": [
            {
                "name": name,
                "used_indices": sorted({int(ch, 16) for row in rows for ch in row}),
            }
            for name, rows in tiles.items()
        ],
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Build deterministic stage tiles from JSON art spec.")
    parser.add_argument("--spec", type=Path, default=DEFAULT_SPEC)
    parser.add_argument("--header", type=Path, default=DEFAULT_HEADER)
    parser.add_argument("--preview", type=Path, default=DEFAULT_PREVIEW)
    parser.add_argument("--report", type=Path, default=DEFAULT_REPORT)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    spec = json.loads(args.spec.read_text(encoding="utf-8"))
    if len(spec["palette"]) != 16:
        raise SystemExit("stage palette must contain exactly 16 colors")
    write_header(spec, args.header)
    draw_preview(spec, args.preview)
    write_report(spec, args.report)
    print(f"wrote {args.header}")
    print(f"wrote {args.preview}")
    print(f"wrote {args.report}")


if __name__ == "__main__":
    main()
