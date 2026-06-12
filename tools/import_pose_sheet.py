#!/usr/bin/env python3
from __future__ import annotations

import argparse
from collections import deque
from pathlib import Path

from PIL import Image, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
POSE_ORDER = [
    "idle",
    "walk_a",
    "walk_b",
    "climb",
    "punch_forward",
    "punch_up",
    "punch_diag_up",
    "punch_diag_down",
]


def remove_chroma(im: Image.Image, key: tuple[int, int, int] | None, tolerance: int) -> Image.Image:
    rgba = im.convert("RGBA")
    px = rgba.load()
    if key is None:
        key = px[0, 0][:3]
    for y in range(rgba.height):
        for x in range(rgba.width):
            r, g, b, a = px[x, y]
            if a == 0:
                continue
            if abs(r - key[0]) <= tolerance and abs(g - key[1]) <= tolerance and abs(b - key[2]) <= tolerance:
                px[x, y] = (0, 0, 0, 0)
    return rgba


def components(im: Image.Image, min_area: int) -> list[dict[str, object]]:
    alpha = im.getchannel("A").filter(ImageFilter.MaxFilter(3))
    mask = alpha.load()
    w, h = im.size
    seen: set[tuple[int, int]] = set()
    found: list[dict[str, object]] = []
    for y in range(h):
        for x in range(w):
            if mask[x, y] < 128 or (x, y) in seen:
                continue
            queue: deque[tuple[int, int]] = deque([(x, y)])
            seen.add((x, y))
            pixels: list[tuple[int, int]] = []
            left = right = x
            top = bottom = y
            while queue:
                qx, qy = queue.popleft()
                pixels.append((qx, qy))
                left = min(left, qx)
                right = max(right, qx)
                top = min(top, qy)
                bottom = max(bottom, qy)
                for nx, ny in ((qx + 1, qy), (qx - 1, qy), (qx, qy + 1), (qx, qy - 1)):
                    if nx < 0 or ny < 0 or nx >= w or ny >= h or (nx, ny) in seen:
                        continue
                    if mask[nx, ny] >= 128:
                        seen.add((nx, ny))
                        queue.append((nx, ny))
            if len(pixels) >= min_area:
                found.append(
                    {
                        "area": len(pixels),
                        "bbox": (left, top, right + 1, bottom + 1),
                    }
                )
    return sorted(found, key=lambda c: (c["bbox"][0], c["bbox"][1]))  # type: ignore[index]


def crop_component(im: Image.Image, bbox: tuple[int, int, int, int], padding: int) -> Image.Image:
    left = max(0, bbox[0] - padding)
    top = max(0, bbox[1] - padding)
    right = min(im.width, bbox[2] + padding)
    bottom = min(im.height, bbox[3] + padding)
    return im.crop((left, top, right, bottom))


def parse_key(value: str | None) -> tuple[int, int, int] | None:
    if value is None:
        return None
    value = value.strip().lstrip("#")
    if len(value) != 6:
        raise argparse.ArgumentTypeError("key must be RRGGBB")
    return (int(value[0:2], 16), int(value[2:4], 16), int(value[4:6], 16))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Import a chroma-key horizontal monster pose sheet into separated source poses.")
    parser.add_argument("source", type=Path)
    parser.add_argument("--monster", required=True, choices=["jonny", "conny", "bettan"])
    parser.add_argument("--key", type=parse_key, default="#00ff00")
    parser.add_argument("--tolerance", type=int, default=36)
    parser.add_argument("--min-area", type=int, default=250)
    parser.add_argument("--padding", type=int, default=20)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    source = args.source.resolve()
    if not source.exists():
        raise SystemExit(f"Missing source sheet: {source}")
    rgba = remove_chroma(Image.open(source), args.key, args.tolerance)
    comps = components(rgba, args.min_area)
    if len(comps) < len(POSE_ORDER):
        raise SystemExit(f"Expected at least {len(POSE_ORDER)} poses, found {len(comps)} in {source}")
    if len(comps) > len(POSE_ORDER):
        comps = sorted(comps, key=lambda c: int(c["area"]), reverse=True)[: len(POSE_ORDER)]
        comps = sorted(comps, key=lambda c: (c["bbox"][0], c["bbox"][1]))  # type: ignore[index]

    out_dir = ROOT / "art/source/monsters" / args.monster
    out_dir.mkdir(parents=True, exist_ok=True)
    for pose, comp in zip(POSE_ORDER, comps):
        crop_component(rgba, comp["bbox"], args.padding).save(out_dir / f"{pose}.png")  # type: ignore[arg-type]
        print(f"{args.monster}/{pose}: bbox={comp['bbox']} area={comp['area']}")


if __name__ == "__main__":
    main()
