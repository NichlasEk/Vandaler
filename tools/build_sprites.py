#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import statistics
from pathlib import Path
from typing import Any

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageOps

from convert_source_sheet import PAL_IMG, quantize, remap_monster_indices, transparent_border, lasso_subject

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CONFIG = ROOT / "art/assets.json"
DEFAULT_PREVIEW = ROOT / "art/source/vandaler_converted_preview.png"
MONSTER_FRAME_W = 48
MONSTER_FRAME_H = 56
MONSTER_TARGET_STANDING_H = 54


def clear_alpha_rect(im: Image.Image, box: tuple[int, int, int, int]) -> None:
    alpha = im.getchannel("A")
    draw = ImageDraw.Draw(alpha)
    draw.rectangle(box, fill=0)
    im.putalpha(alpha)


def clean_source(im: Image.Image, min_area: int, trim_padding: int) -> Image.Image:
    rgba = transparent_border(im)
    rgba = lasso_subject(rgba, min_area)
    rgba = keep_major_components(rgba, min_ratio=0.12)
    alpha = rgba.getchannel("A").filter(ImageFilter.MaxFilter(3)).filter(ImageFilter.MinFilter(3))
    rgba.putalpha(alpha)
    bbox = rgba.getchannel("A").getbbox()
    if bbox is None:
        return rgba
    left = max(0, bbox[0] - trim_padding)
    top = max(0, bbox[1] - trim_padding)
    right = min(rgba.width, bbox[2] + trim_padding)
    bottom = min(rgba.height, bbox[3] + trim_padding)
    return rgba.crop((left, top, right, bottom))


def keep_major_components(im: Image.Image, min_ratio: float) -> Image.Image:
    rgba = im.convert("RGBA")
    alpha = rgba.getchannel("A")
    mask = alpha.load()
    w, h = rgba.size
    seen: set[tuple[int, int]] = set()
    comps: list[list[tuple[int, int]]] = []
    for y in range(h):
        for x in range(w):
            if mask[x, y] < 128 or (x, y) in seen:
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
                    if mask[nx, ny] >= 128:
                        seen.add((nx, ny))
                        queue.append((nx, ny))
            comps.append(pixels)
    if not comps:
        return rgba
    max_area = max(len(c) for c in comps)
    keep = [c for c in comps if len(c) >= max_area * min_ratio]
    out_alpha = Image.new("L", (w, h), 0)
    out = out_alpha.load()
    for comp in keep:
        for x, y in comp:
            out[x, y] = 255
    rgba.putalpha(out_alpha)
    return rgba


def alpha_bbox(im: Image.Image) -> tuple[int, int, int, int] | None:
    return im.convert("RGBA").getchannel("A").getbbox()


def bbox_size(bbox: tuple[int, int, int, int] | None) -> tuple[int, int]:
    if bbox is None:
        return 1, 1
    return max(1, bbox[2] - bbox[0]), max(1, bbox[3] - bbox[1])


def fit_to_frame(src: Image.Image, frame_w: int, frame_h: int, *, margin: int, offset: tuple[int, int]) -> Image.Image:
    max_w = max(1, frame_w - margin * 2)
    max_h = max(1, frame_h - margin * 2)
    fitted = ImageOps.contain(src.convert("RGBA"), (max_w, max_h), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (frame_w, frame_h), (0, 0, 0, 0))
    bbox = alpha_bbox(fitted)
    if bbox is None:
        return canvas
    alpha_center = (bbox[0] + bbox[2]) // 2
    x = (frame_w // 2) - alpha_center + offset[0]
    y = frame_h - bbox[3] - margin + offset[1]
    canvas.alpha_composite(fitted, (x, y))
    return canvas


def fit_to_frame_at_scale(src: Image.Image, frame_w: int, frame_h: int, *, scale: float, offset: tuple[int, int]) -> Image.Image:
    rgba = src.convert("RGBA")
    fitted_w = max(1, int(round(rgba.width * scale)))
    fitted_h = max(1, int(round(rgba.height * scale)))
    fitted = rgba.resize((fitted_w, fitted_h), Image.Resampling.LANCZOS)
    bbox = alpha_bbox(fitted)
    canvas = Image.new("RGBA", (frame_w, frame_h), (0, 0, 0, 0))
    if bbox is None:
        return canvas

    alpha_center = (bbox[0] + bbox[2]) // 2
    x = (frame_w // 2) - alpha_center + offset[0]
    y = frame_h - bbox[3] + offset[1]
    canvas.alpha_composite(fitted, (x, y))
    return canvas


def monster_normalized_scale(cleaned_frames: dict[str, Image.Image]) -> float:
    reference_poses = ("idle", "walk_a", "walk_b")
    reference_heights: list[int] = []
    max_allowed_scale = 1000.0

    for pose, frame in cleaned_frames.items():
        bbox = alpha_bbox(frame)
        width, height = bbox_size(bbox)
        max_allowed_scale = min(
            max_allowed_scale,
            MONSTER_FRAME_W / width,
            MONSTER_FRAME_H / height,
        )
        if pose in reference_poses:
            reference_heights.append(height)

    if not reference_heights:
        reference_heights = [bbox_size(alpha_bbox(frame))[1] for frame in cleaned_frames.values()]

    standing_h = max(1, int(statistics.median(reference_heights)))
    target_scale = MONSTER_TARGET_STANDING_H / standing_h
    return min(target_scale, max_allowed_scale)


def validate_frame(name: str, frame: Image.Image) -> list[str]:
    alpha = frame.getchannel("A")
    bbox = alpha.getbbox()
    if bbox is None:
        return [f"{name}: empty frame"]
    warnings: list[str] = []
    if bbox[0] == 0 or bbox[1] == 0 or bbox[2] == frame.width or bbox[3] == frame.height:
        warnings.append(f"{name}: touches frame edge bbox={bbox}")
    return warnings


def build_monsters(config: dict[str, Any], warnings: list[str]) -> None:
    frames_per_monster = max(len(monster["frames"]) for monster in config["monsters"])
    sheet = Image.new("P", (MONSTER_FRAME_W * frames_per_monster * len(config["monsters"]), MONSTER_FRAME_H), 0)
    sheet.putpalette(PAL_IMG.getpalette())

    for monster_index, monster in enumerate(config["monsters"]):
        monster_name = monster["name"]
        unique_poses = sorted(set(monster["frames"]))
        cleaned_frames: dict[str, Image.Image] = {}
        for pose in unique_poses:
            src_path = ROOT / "art/source/monsters" / monster_name / f"{pose}.png"
            if not src_path.exists():
                raise SystemExit(f"Missing monster source: {src_path}")

            source = Image.open(src_path).convert("RGBA")
            cleaned_frames[pose] = clean_source(source, min_area=18, trim_padding=8)

        normalized_scale = monster_normalized_scale(cleaned_frames)
        for frame_index, pose in enumerate(monster["frames"]):
            cleaned = cleaned_frames[pose]
            frame = fit_to_frame_at_scale(cleaned, MONSTER_FRAME_W, MONSTER_FRAME_H, scale=normalized_scale, offset=(0, 0))
            if pose == "climb":
                clear_alpha_rect(frame, (0, 0, MONSTER_FRAME_W - 1, 6))
            if frame_index in (6, 13):
                frame = ImageChops.offset(frame, 0, 1)

            warnings.extend(validate_frame(f"{monster_name}:{frame_index}:{pose}", frame))
            q = remap_monster_indices(quantize(frame, opaque_zero_index=2))
            sheet.paste(q, ((monster_index * frames_per_monster + frame_index) * MONSTER_FRAME_W, 0))

    out = ROOT / "res/images/monsters.png"
    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)


def build_object(obj: dict[str, Any], warnings: list[str]) -> None:
    frame_w = int(obj["frame_width"])
    frame_h = int(obj["frame_height"])
    frames = int(obj["frames"])
    source_path = ROOT / obj["source"]
    out_path = ROOT / obj["out"]
    if not source_path.exists():
        raise SystemExit(f"Missing object source: {source_path}")

    source = Image.open(source_path).convert("RGBA")
    sheet = Image.new("P", (frame_w * frames, frame_h), 0)
    sheet.putpalette(PAL_IMG.getpalette())

    def object_quantize(frame: Image.Image) -> Image.Image:
        q = quantize(frame, opaque_zero_index=11)
        if obj["name"] in {"enemy", "tank", "helicopter"}:
            remap = {7: 5, 10: 5, 13: 4}
            src = q.load()
            for y in range(q.height):
                for x in range(q.width):
                    src[x, y] = remap.get(src[x, y], src[x, y])
        return q

    if source.size == (frame_w * frames, frame_h):
        for frame_index in range(frames):
            frame = source.crop((frame_index * frame_w, 0, (frame_index + 1) * frame_w, frame_h))
            warnings.extend(validate_frame(f"{obj['name']}:{frame_index}", frame))
            sheet.paste(object_quantize(frame), (frame_index * frame_w, 0))
    else:
        cleaned = clean_source(source, min_area=10, trim_padding=4)
        base = fit_to_frame(cleaned, frame_w, frame_h, margin=1, offset=(0, 0))
        for frame_index in range(frames):
            frame = base
            if frame_index:
                frame = ImageChops.offset(frame, frame_index % 2, 0)
            warnings.extend(validate_frame(f"{obj['name']}:{frame_index}", frame))
            sheet.paste(object_quantize(frame), (frame_index * frame_w, 0))

    out_path.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out_path)


def build_objects(config: dict[str, Any], warnings: list[str]) -> None:
    for obj in config["objects"]:
        build_object(obj, warnings)


def write_preview(preview: Path) -> None:
    rows: list[tuple[str, Image.Image]] = []
    for name in ["monsters.png", "enemy.png", "tank.png", "helicopter.png", "person.png", "shot.png", "explosion.png"]:
        im = Image.open(ROOT / "res/images" / name).convert("RGBA")
        if name == "monsters.png":
            im = im.crop((0, 0, im.width, MONSTER_FRAME_H))
        scale = 1 if name == "monsters.png" else 3
        rows.append((name, im.resize((im.width * scale, im.height * scale), Image.Resampling.NEAREST)))

    width = max(im.width for _, im in rows) + 18
    height = sum(im.height + 22 for _, im in rows) + 10
    canvas = Image.new("RGBA", (width, height), (28, 28, 38, 255))
    draw = ImageDraw.Draw(canvas)
    y = 6
    for name, im in rows:
        draw.text((8, y), name, fill=(255, 255, 255, 255))
        y += 11
        canvas.alpha_composite(im, (8, y))
        y += im.height + 11
    preview.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(preview)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Build SGDK sprite sheets from separated Vandaler source pose files.")
    parser.add_argument("--config", type=Path, default=DEFAULT_CONFIG)
    parser.add_argument("--preview", type=Path, default=None)
    parser.add_argument("--strict", action="store_true", help="Fail on validation warnings.")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    config = json.loads(args.config.read_text(encoding="utf-8"))
    warnings: list[str] = []
    build_monsters(config, warnings)
    build_objects(config, warnings)
    if args.preview is not None:
        write_preview(args.preview.resolve())
    for warning in warnings:
        print(f"warning: {warning}")
    if args.strict and warnings:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
