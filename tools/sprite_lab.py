#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable

from PIL import Image, ImageDraw, ImageEnhance, ImageFilter, ImageOps

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUT = ROOT / "art/source/sprite_lab"
MONSTER_FRAME_W = 48
MONSTER_FRAME_H = 56
OBJECT_SPECS = {
    "enemy": (24, 16),
    "tank": (24, 16),
    "person": (8, 16),
}


@dataclass
class FrameStats:
    bbox: tuple[int, int, int, int] | None
    alpha_area: int
    alpha_ratio: float
    mean_rgb: tuple[float, float, float]
    saturation: float
    luma_span: float
    centroid: tuple[float, float]


@dataclass
class Decision:
    name: str
    accepted: bool
    reasons: list[str]
    stats: FrameStats
    reference: FrameStats | None


def load_rgba(path: Path) -> Image.Image:
    return Image.open(path).convert("RGBA")


def alpha_bbox(im: Image.Image) -> tuple[int, int, int, int] | None:
    return im.getchannel("A").getbbox()


def remove_chroma(im: Image.Image, key: str = "auto") -> Image.Image:
    rgba = im.convert("RGBA")
    if key == "auto":
        samples = []
        px = rgba.load()
        for x, y in (
            (0, 0),
            (rgba.width - 1, 0),
            (0, rgba.height - 1),
            (rgba.width - 1, rgba.height - 1),
            (rgba.width // 2, 0),
            (rgba.width // 2, rgba.height - 1),
        ):
            samples.append(px[x, y][:3])
        kr, kg, kb = max(set(samples), key=samples.count)
    else:
        key = key.lstrip("#")
        kr, kg, kb = int(key[0:2], 16), int(key[2:4], 16), int(key[4:6], 16)

    px = rgba.load()
    for y in range(rgba.height):
        for x in range(rgba.width):
            r, g, b, a = px[x, y]
            dist = math.sqrt((r - kr) ** 2 + (g - kg) ** 2 + (b - kb) ** 2)
            if dist < 42:
                px[x, y] = (0, 0, 0, 0)
            else:
                px[x, y] = (r, g, b, a)
    alpha = rgba.getchannel("A").filter(ImageFilter.MinFilter(3)).filter(ImageFilter.MaxFilter(3))
    rgba.putalpha(alpha)
    return rgba


def connected_components(alpha: Image.Image, threshold: int = 128) -> list[list[tuple[int, int]]]:
    mask = alpha.load()
    w, h = alpha.size
    seen: set[tuple[int, int]] = set()
    comps: list[list[tuple[int, int]]] = []
    for y in range(h):
        for x in range(w):
            if mask[x, y] < threshold or (x, y) in seen:
                continue
            queue = [(x, y)]
            seen.add((x, y))
            pixels: list[tuple[int, int]] = []
            while queue:
                qx, qy = queue.pop()
                pixels.append((qx, qy))
                for nx, ny in ((qx + 1, qy), (qx - 1, qy), (qx, qy + 1), (qx, qy - 1)):
                    if nx < 0 or ny < 0 or nx >= w or ny >= h or (nx, ny) in seen:
                        continue
                    if mask[nx, ny] >= threshold:
                        seen.add((nx, ny))
                        queue.append((nx, ny))
            comps.append(pixels)
    return comps


def keep_main_components(im: Image.Image, min_ratio: float = 0.04) -> Image.Image:
    rgba = im.convert("RGBA")
    comps = connected_components(rgba.getchannel("A"))
    if not comps:
        return rgba
    max_area = max(len(comp) for comp in comps)
    out_alpha = Image.new("L", rgba.size, 0)
    out = out_alpha.load()
    for comp in comps:
        if len(comp) < max_area * min_ratio:
            continue
        for x, y in comp:
            out[x, y] = 255
    rgba.putalpha(out_alpha)
    return rgba


def trim(im: Image.Image, padding: int = 4) -> Image.Image:
    bbox = alpha_bbox(im)
    if bbox is None:
        return im
    return im.crop(
        (
            max(0, bbox[0] - padding),
            max(0, bbox[1] - padding),
            min(im.width, bbox[2] + padding),
            min(im.height, bbox[3] + padding),
        )
    )


def frame_stats(im: Image.Image) -> FrameStats:
    rgba = im.convert("RGBA")
    alpha = rgba.getchannel("A")
    bbox = alpha.getbbox()
    px = rgba.load()
    alpha_px = alpha.load()
    area = 0
    sum_r = sum_g = sum_b = 0
    sum_sat = 0.0
    min_luma = 255.0
    max_luma = 0.0
    sum_x = sum_y = 0
    for y in range(rgba.height):
        for x in range(rgba.width):
            a = alpha_px[x, y]
            if a < 128:
                continue
            r, g, b, _ = px[x, y]
            max_c = max(r, g, b)
            min_c = min(r, g, b)
            luma = 0.2126 * r + 0.7152 * g + 0.0722 * b
            area += 1
            sum_r += r
            sum_g += g
            sum_b += b
            sum_sat += 0.0 if max_c == 0 else (max_c - min_c) / max_c
            min_luma = min(min_luma, luma)
            max_luma = max(max_luma, luma)
            sum_x += x
            sum_y += y
    if area == 0:
        return FrameStats(bbox, 0, 0.0, (0.0, 0.0, 0.0), 0.0, 0.0, (0.0, 0.0))
    return FrameStats(
        bbox=bbox,
        alpha_area=area,
        alpha_ratio=area / (rgba.width * rgba.height),
        mean_rgb=(sum_r / area, sum_g / area, sum_b / area),
        saturation=sum_sat / area,
        luma_span=max_luma - min_luma,
        centroid=(sum_x / area, sum_y / area),
    )


def color_distance(a: tuple[float, float, float], b: tuple[float, float, float]) -> float:
    return math.sqrt(sum((aa - bb) ** 2 for aa, bb in zip(a, b)))


def fit_to_frame(src: Image.Image, frame_w: int, frame_h: int, ref: FrameStats | None) -> Image.Image:
    src = trim(keep_main_components(src), padding=2)
    bbox = alpha_bbox(src)
    if bbox is None:
        return Image.new("RGBA", (frame_w, frame_h), (0, 0, 0, 0))

    max_w = frame_w - 2
    max_h = frame_h - 2
    if ref and ref.bbox:
        ref_w = ref.bbox[2] - ref.bbox[0]
        ref_h = ref.bbox[3] - ref.bbox[1]
        src_w = bbox[2] - bbox[0]
        src_h = bbox[3] - bbox[1]
        scale = min(ref_w / max(1, src_w), ref_h / max(1, src_h), max_w / max(1, src_w), max_h / max(1, src_h))
        fitted = src.resize((max(1, round(src.width * scale)), max(1, round(src.height * scale))), Image.Resampling.LANCZOS)
    else:
        fitted = ImageOps.contain(src, (max_w, max_h), Image.Resampling.LANCZOS)

    out = Image.new("RGBA", (frame_w, frame_h), (0, 0, 0, 0))
    fb = alpha_bbox(fitted)
    if fb is None:
        return out
    center_x = (fb[0] + fb[2]) // 2
    x = frame_w // 2 - center_x
    y = frame_h - fb[3] - 1
    out.alpha_composite(fitted, (x, y))
    return out


def boost_frame(im: Image.Image, saturation: float, contrast: float, brightness: float) -> Image.Image:
    rgba = im.convert("RGBA")
    alpha = rgba.getchannel("A")
    rgb = rgba.convert("RGB")
    rgb = ImageEnhance.Color(rgb).enhance(saturation)
    rgb = ImageEnhance.Contrast(rgb).enhance(contrast)
    rgb = ImageEnhance.Brightness(rgb).enhance(brightness)
    out = rgb.convert("RGBA")
    out.putalpha(alpha)
    return out


def split_sheet(sheet: Image.Image, rows: int, cols: int, key: str) -> list[list[Image.Image]]:
    cell_w = sheet.width // cols
    cell_h = sheet.height // rows
    frames: list[list[Image.Image]] = []
    for row in range(rows):
        row_frames: list[Image.Image] = []
        for col in range(cols):
            cell = sheet.crop((col * cell_w, row * cell_h, (col + 1) * cell_w, (row + 1) * cell_h))
            row_frames.append(trim(keep_main_components(remove_chroma(cell, key=key)), padding=4))
        frames.append(row_frames)
    return frames


def load_object_reference(name: str) -> list[Image.Image]:
    path = ROOT / "art/source/objects" / f"{name}.png"
    frame_w, frame_h = OBJECT_SPECS[name]
    im = load_rgba(path)
    if im.size[0] % frame_w == 0 and im.size[1] == frame_h:
        return [im.crop((x, 0, x + frame_w, frame_h)) for x in range(0, im.width, frame_w)]
    return [fit_to_frame(im, frame_w, frame_h, None)]


def load_monster_reference(name: str) -> list[Image.Image]:
    root = ROOT / "art/source/monsters" / name
    return [load_rgba(root / "walk_a.png"), load_rgba(root / "walk_b.png")]


def decide(name: str, frame: Image.Image, ref_stats: list[FrameStats], *, allow_touch_bottom: bool) -> Decision:
    stats = frame_stats(frame)
    ref = ref_stats[min(len(ref_stats) - 1, 0)] if ref_stats else None
    reasons: list[str] = []
    if stats.bbox is None:
        reasons.append("empty frame")
    else:
        left, top, right, bottom = stats.bbox
        if left == 0 or right == frame.width:
            reasons.append(f"touches horizontal edge bbox={stats.bbox}")
        if top == 0:
            reasons.append(f"touches top edge bbox={stats.bbox}")
        if bottom == frame.height and not allow_touch_bottom:
            reasons.append(f"touches bottom edge bbox={stats.bbox}")
    if ref_stats and stats.alpha_area:
        areas = [s.alpha_area for s in ref_stats if s.alpha_area > 0]
        colors = [s.mean_rgb for s in ref_stats if s.alpha_area > 0]
        if areas:
            ref_area = sum(areas) / len(areas)
            ratio = stats.alpha_area / ref_area
            if ratio < 0.55 or ratio > 1.65:
                reasons.append(f"alpha area drift {ratio:.2f}x")
        if colors:
            nearest = min(color_distance(stats.mean_rgb, color) for color in colors)
            if nearest > 95:
                reasons.append(f"color drift {nearest:.1f}")
        ref_sat = sum(s.saturation for s in ref_stats) / len(ref_stats)
        ref_span = sum(s.luma_span for s in ref_stats) / len(ref_stats)
        if stats.saturation < max(0.12, ref_sat * 0.72):
            reasons.append(f"low saturation {stats.saturation:.2f} ref={ref_sat:.2f}")
        if stats.luma_span < max(35.0, ref_span * 0.65):
            reasons.append(f"flat contrast {stats.luma_span:.1f} ref={ref_span:.1f}")
    return Decision(name=name, accepted=not reasons, reasons=reasons, stats=stats, reference=ref)


def contact_sheet(rows: list[tuple[str, list[Image.Image], list[Decision]]], out_path: Path) -> None:
    scale = 4
    label_h = 14
    row_h = max((frames[0].height * scale + label_h for _, frames, _ in rows if frames), default=32)
    width = max((len(frames) * frames[0].width * scale for _, frames, _ in rows if frames), default=64) + 8
    height = len(rows) * row_h + 8
    canvas = Image.new("RGBA", (width, height), (26, 26, 34, 255))
    draw = ImageDraw.Draw(canvas)
    y = 4
    for label, frames, decisions in rows:
        draw.text((4, y), label, fill=(255, 255, 255, 255))
        x = 4
        for frame, decision in zip(frames, decisions):
            preview = frame.resize((frame.width * scale, frame.height * scale), Image.Resampling.NEAREST)
            canvas.alpha_composite(preview, (x, y + label_h))
            color = (80, 240, 120, 255) if decision.accepted else (255, 70, 70, 255)
            draw.rectangle((x, y + label_h, x + preview.width - 1, y + label_h + preview.height - 1), outline=color)
            x += preview.width
        y += row_h
    out_path.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(out_path)


def write_report(path: Path, decisions: Iterable[Decision]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps([asdict(d) for d in decisions], indent=2), encoding="utf-8")


def audit_current(args: argparse.Namespace) -> None:
    rows: list[tuple[str, list[Image.Image], list[Decision]]] = []
    all_decisions: list[Decision] = []
    for name in ("jonny", "conny", "bettan"):
        refs = load_monster_reference(name)
        stats = [frame_stats(fit_to_frame(frame, MONSTER_FRAME_W, MONSTER_FRAME_H, None)) for frame in refs]
        frames = [fit_to_frame(frame, MONSTER_FRAME_W, MONSTER_FRAME_H, None) for frame in refs]
        decisions = [decide(f"{name}:walk_{i}", frame, stats, allow_touch_bottom=True) for i, frame in enumerate(frames)]
        rows.append((name, frames, decisions))
        all_decisions.extend(decisions)
    for name in ("enemy", "tank", "person"):
        frames = load_object_reference(name)
        stats = [frame_stats(frame) for frame in frames]
        decisions = [decide(f"{name}:{i}", frame, stats, allow_touch_bottom=True) for i, frame in enumerate(frames)]
        rows.append((name, frames, decisions))
        all_decisions.extend(decisions)
    contact_sheet(rows, args.out / "audit_current.png")
    write_report(args.out / "audit_current.json", all_decisions)
    print(f"wrote {args.out / 'audit_current.png'}")
    print(f"wrote {args.out / 'audit_current.json'}")


def audit_grid(args: argparse.Namespace) -> None:
    sheet = load_rgba(args.sheet)
    rows = split_sheet(sheet, args.rows, args.cols, args.key)
    labels = args.labels.split(",")
    if len(labels) != args.rows:
        raise SystemExit("--labels count must match --rows")

    preview_rows: list[tuple[str, list[Image.Image], list[Decision]]] = []
    all_decisions: list[Decision] = []
    for row_index, label in enumerate(labels):
        if args.kind == "monsters":
            frame_w, frame_h = MONSTER_FRAME_W, MONSTER_FRAME_H
            refs = [fit_to_frame(frame, frame_w, frame_h, None) for frame in load_monster_reference(label)]
        else:
            frame_w, frame_h = OBJECT_SPECS[label]
            refs = load_object_reference(label)
        ref_stats = [frame_stats(ref) for ref in refs]
        fitted = [fit_to_frame(frame, frame_w, frame_h, ref_stats[col % len(ref_stats)] if ref_stats else None) for col, frame in enumerate(rows[row_index])]
        if args.boost:
            fitted = [boost_frame(frame, args.saturation, args.contrast, args.brightness) for frame in fitted]
        decisions = [
            decide(f"{label}:{col}", frame, ref_stats, allow_touch_bottom=True)
            for col, frame in enumerate(fitted)
        ]
        preview_rows.append((label, fitted, decisions))
        all_decisions.extend(decisions)
    stem = args.sheet.stem
    contact_sheet(preview_rows, args.out / f"{stem}_{args.kind}_audit.png")
    write_report(args.out / f"{stem}_{args.kind}_audit.json", all_decisions)
    failed = [d for d in all_decisions if not d.accepted]
    print(f"wrote {args.out / f'{stem}_{args.kind}_audit.png'}")
    print(f"wrote {args.out / f'{stem}_{args.kind}_audit.json'}")
    if failed:
        for decision in failed:
            print(f"reject: {decision.name}: {', '.join(decision.reasons)}")
        raise SystemExit(1)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Audit AI sprite candidates before they can enter the SGDK asset pipeline.")
    sub = parser.add_subparsers(dest="cmd", required=True)

    current = sub.add_parser("audit-current", help="write a baseline report for current source sprites")
    current.add_argument("--out", type=Path, default=DEFAULT_OUT)
    current.set_defaults(func=audit_current)

    grid = sub.add_parser("audit-grid", help="audit a chroma-key grid sheet without modifying game assets")
    grid.add_argument("--sheet", type=Path, required=True)
    grid.add_argument("--kind", choices=("monsters", "objects"), required=True)
    grid.add_argument("--rows", type=int, required=True)
    grid.add_argument("--cols", type=int, required=True)
    grid.add_argument("--labels", required=True, help="comma-separated row names, e.g. jonny,conny,bettan")
    grid.add_argument("--key", default="auto", help="auto or hex RGB such as ff00ff")
    grid.add_argument("--out", type=Path, default=DEFAULT_OUT)
    grid.add_argument("--boost", action="store_true", help="preview candidates with saturation/contrast boost before audit")
    grid.add_argument("--saturation", type=float, default=1.35)
    grid.add_argument("--contrast", type=float, default=1.14)
    grid.add_argument("--brightness", type=float, default=1.03)
    grid.set_defaults(func=audit_grid)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
