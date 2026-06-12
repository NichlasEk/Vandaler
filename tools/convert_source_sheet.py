#!/usr/bin/env python3
from __future__ import annotations

import argparse
from collections import deque
from pathlib import Path
from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageOps

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = ROOT / "art/source/vandaler_source_sheet_01.png"
DEFAULT_OUT = ROOT / "res/images"
DEFAULT_PREVIEW = ROOT / "art/source/vandaler_converted_preview.png"

PALETTE = [
    (0, 0, 0),        # 0 transparent / black key
    (0, 36, 182),     # 1 night sky blue
    (0, 0, 0),        # 2 outline
    (109, 109, 146),  # 3 cool concrete grey
    (0, 210, 50),     # 4 green
    (0, 109, 36),     # 5 dark green
    (109, 182, 255),  # 6 glass blue
    (182, 109, 36),   # 7 brick brown
    (255, 30, 0),     # 8 red
    (255, 255, 255),  # 9 white
    (109, 36, 0),     # 10 brown shadow
    (36, 36, 109),    # 11 deep shadow
    (0, 0, 109),      # 12 deep blue
    (255, 182, 73),   # 13 warm highlight
    (0, 0, 0),        # 14 spare black
    (255, 255, 255),  # 15 spare white
]


def palette_image() -> Image.Image:
    pal = []
    for rgb in PALETTE:
        pal.extend(rgb)
    pal.extend([0, 0, 0] * (256 - len(PALETTE)))
    im = Image.new("P", (1, 1), 0)
    im.putpalette(pal)
    return im


PAL_IMG = palette_image()


def quantize(im: Image.Image, opaque_zero_index: int = 2) -> Image.Image:
    rgba = im.convert("RGBA")
    rgb = Image.new("RGB", rgba.size, (0, 0, 0))
    rgb.paste(rgba.convert("RGB"), mask=rgba.getchannel("A"))
    out = rgb.quantize(palette=PAL_IMG, dither=Image.Dither.NONE)
    alpha = rgba.getchannel("A")
    src_alpha = alpha.load()
    dst = out.load()
    for y in range(out.height):
        for x in range(out.width):
            if src_alpha[x, y] < 128:
                dst[x, y] = 0
            elif dst[x, y] == 0:
                dst[x, y] = opaque_zero_index
    return out


def remap_monster_indices(im: Image.Image) -> Image.Image:
    # Monster sprites are drawn with PAL1/PAL2/PAL3 at runtime, so their pixels
    # must use the shared monster role indices instead of the background palette.
    remap = {
        0: 0,
        1: 3,
        2: 7,
        3: 2,
        4: 3,
        5: 2,
        6: 4,
        7: 3,
        8: 6,
        9: 5,
        10: 2,
        11: 7,
        12: 2,
        13: 4,
        14: 7,
        15: 5,
    }
    out = Image.new("P", im.size, 0)
    out.putpalette(PAL_IMG.getpalette())
    src = im.load()
    dst = out.load()
    for y in range(im.height):
        for x in range(im.width):
            dst[x, y] = remap.get(src[x, y], 3)
    return out


def transparent_border(im: Image.Image, threshold: int = 42) -> Image.Image:
    rgba = im.convert("RGBA")
    px = rgba.load()
    key = px[0, 0][:3]
    seen = set()
    queue: deque[tuple[int, int]] = deque()
    for x in range(rgba.width):
        queue.append((x, 0))
        queue.append((x, rgba.height - 1))
    for y in range(rgba.height):
        queue.append((0, y))
        queue.append((rgba.width - 1, y))

    def close_to_key(rgb: tuple[int, int, int]) -> bool:
        diffs = [abs(rgb[i] - key[i]) for i in range(3)]
        return sum(diffs) <= threshold and max(diffs) <= 22

    while queue:
        x, y = queue.popleft()
        if x < 0 or y < 0 or x >= rgba.width or y >= rgba.height or (x, y) in seen:
            continue
        seen.add((x, y))
        r, g, b, a = px[x, y]
        if a != 0 and close_to_key((r, g, b)):
            px[x, y] = (0, 0, 0, 0)
            queue.append((x + 1, y))
            queue.append((x - 1, y))
            queue.append((x, y + 1))
            queue.append((x, y - 1))
    return rgba


def trim_alpha(im: Image.Image, padding: int) -> Image.Image:
    bbox = im.getchannel("A").getbbox()
    if bbox is None:
        return im
    left = max(0, bbox[0] - padding)
    top = max(0, bbox[1] - padding)
    right = min(im.width, bbox[2] + padding)
    bottom = min(im.height, bbox[3] + padding)
    return im.crop((left, top, right, bottom))


def lasso_subject(im: Image.Image, min_area: int, bridge_passes: int = 1) -> Image.Image:
    rgba = im.convert("RGBA")
    alpha = rgba.getchannel("A")
    for _ in range(bridge_passes):
        alpha = alpha.filter(ImageFilter.MaxFilter(3))

    mask = alpha.load()
    w, h = rgba.size
    seen: set[tuple[int, int]] = set()
    components: list[dict[str, object]] = []

    for y in range(h):
        for x in range(w):
            if mask[x, y] < 128 or (x, y) in seen:
                continue
            queue: deque[tuple[int, int]] = deque([(x, y)])
            seen.add((x, y))
            area = 0
            sx = 0
            sy = 0
            pixels: list[tuple[int, int]] = []
            left = x
            top = y
            right = x
            bottom = y

            while queue:
                qx, qy = queue.popleft()
                pixels.append((qx, qy))
                area += 1
                sx += qx
                sy += qy
                left = min(left, qx)
                top = min(top, qy)
                right = max(right, qx)
                bottom = max(bottom, qy)
                for nx, ny in ((qx + 1, qy), (qx - 1, qy), (qx, qy + 1), (qx, qy - 1)):
                    if nx < 0 or ny < 0 or nx >= w or ny >= h or (nx, ny) in seen:
                        continue
                    if mask[nx, ny] >= 128:
                        seen.add((nx, ny))
                        queue.append((nx, ny))

            components.append(
                {
                    "area": area,
                    "left": left,
                    "top": top,
                    "right": right + 1,
                    "bottom": bottom + 1,
                    "cx": sx / area,
                    "cy": sy / area,
                    "pixels": pixels,
                }
            )

    if not components:
        return rgba

    components.sort(reverse=True, key=lambda c: int(c["area"]))
    main = components[0]
    keep: list[dict[str, object]] = []
    main_cx = float(main["cx"])
    main_cy = float(main["cy"])
    main_area = int(main["area"])

    for comp in components:
        area = int(comp["area"])
        left = int(comp["left"])
        top = int(comp["top"])
        right = int(comp["right"])
        bottom = int(comp["bottom"])
        cx = float(comp["cx"])
        cy = float(comp["cy"])
        bw = right - left
        bh = bottom - top
        long_thin = bh > bw * 4 and area < main_area * 0.35
        flat_thin = bw > bh * 6 and area < main_area * 0.25
        border_fragment = (top <= 1 or bottom >= h - 1 or left <= 1 or right >= w - 1) and area < main_area * 0.08
        far_from_main = abs(cx - main_cx) > w * 0.42 or abs(cy - main_cy) > h * 0.48
        if comp is main or (area >= min_area and not border_fragment and not ((long_thin or flat_thin) and far_from_main)):
            keep.append(comp)

    out_alpha = Image.new("L", (w, h), 0)
    out_px = out_alpha.load()
    for comp in keep:
        for x, y in comp["pixels"]:  # type: ignore[index]
            out_px[x, y] = 255

    # Close tiny source-sheet pinholes but keep the final edge crisp.
    out_alpha = out_alpha.filter(ImageFilter.MaxFilter(3)).filter(ImageFilter.MinFilter(3))
    rgba.putalpha(out_alpha)
    return rgba


def grow_alpha(im: Image.Image, passes: int) -> Image.Image:
    if passes <= 0:
        return im
    rgba = im.convert("RGBA")
    alpha = rgba.getchannel("A")
    for _ in range(passes):
        alpha = alpha.filter(ImageFilter.MaxFilter(3))
    rgba.putalpha(alpha)
    return rgba


def sprite_from_crop(
    src: Image.Image,
    box: tuple[int, int, int, int],
    size: tuple[int, int],
    padding: int = 4,
    alpha_grow: int = 0,
    lasso: bool = False,
    min_component_area: int = 24,
) -> Image.Image:
    crop = transparent_border(src.crop(box))
    if lasso:
        crop = lasso_subject(crop, min_component_area)
    crop = grow_alpha(crop, alpha_grow)
    crop = trim_alpha(crop, padding)
    crop = ImageOps.contain(crop, size, Image.Resampling.LANCZOS)
    crop = crop.convert("RGBA")
    canvas = Image.new("RGBA", size, (0, 0, 0, 0))
    canvas.alpha_composite(crop, ((size[0] - crop.width) // 2, size[1] - crop.height))
    return canvas


def make_monsters(src: Image.Image, out: Path) -> None:
    # Coordinates on the generated source sheet, mapped to the game's pose constants.
    ape_y = (14, 186)
    wolf_y = (205, 365)
    lizard_y = (382, 536)
    pose_x = [
        (0, 145),
        (150, 275),
        (285, 410),
        (392, 492),
        (535, 665),
        (675, 765),
        (775, 895),
        (905, 1030),
    ]
    pose_map = [0, 3, 4, 5, 6, 6, 3, 3, 5, 6, 6, 5, 1, 2]
    rows = [ape_y, wolf_y, lizard_y]
    poses = 14
    frame_w, frame_h = 48, 56
    sheet = Image.new("P", (frame_w * poses * 3, frame_h), 0)
    sheet.putpalette(PAL_IMG.getpalette())

    for monster, ybox in enumerate(rows):
        for pose in range(poses):
            sx = pose_x[pose_map[pose]]
            crop = (sx[0], ybox[0], sx[1], ybox[1])
            frame = sprite_from_crop(
                src,
                crop,
                (frame_w, frame_h),
                padding=12,
                alpha_grow=1,
                lasso=True,
                min_component_area=18,
            )
            if pose in (1, 6, 12, 13):
                frame = ImageChops.offset(frame, 0, 1 if pose in (1, 13) else -1)
            if pose in (1, 6):
                clear_alpha_rect(frame, (0, 0, frame_w - 1, 7))
            q = remap_monster_indices(quantize(frame, opaque_zero_index=2))
            sheet.paste(q, ((monster * poses + pose) * frame_w, 0))

    sheet.save(out / "monsters.png")


def draw_attack_hint(frame: Image.Image, pose: int) -> None:
    d = ImageDraw.Draw(frame)
    if pose in (3, 8):
        d.rectangle((32, 3, 40, 13), fill=(255, 220, 70, 255))
    elif pose in (4, 9):
        d.line((31, 18, 44, 8), fill=(255, 220, 70, 255), width=5)
    elif pose in (5, 10):
        d.line((31, 34, 44, 47), fill=(255, 220, 70, 255), width=5)
    else:
        d.rectangle((34, 24, 47, 32), fill=(255, 220, 70, 255))


def clear_alpha_rect(im: Image.Image, box: tuple[int, int, int, int]) -> None:
    alpha = im.getchannel("A")
    draw = ImageDraw.Draw(alpha)
    draw.rectangle(box, fill=0)
    im.putalpha(alpha)


def make_objects(src: Image.Image, out: Path) -> None:
    specs = [
        ("enemy.png", (8, 742, 220, 815), (24, 16), 3),
        ("tank.png", (255, 736, 450, 815), (24, 16), 3),
        ("helicopter.png", (490, 727, 724, 822), (32, 16), 4),
        ("person.png", (12, 546, 80, 625), (8, 16), 1),
        ("shot.png", (14, 826, 62, 862), (8, 8), 1),
        ("explosion.png", (20, 870, 360, 1005), (16, 16), 2),
    ]
    for filename, crop, frame_size, frames in specs:
        base = sprite_from_crop(
            src,
            crop,
            frame_size,
            padding=3,
            alpha_grow=1,
            lasso=True,
            min_component_area=10,
        )
        sheet = Image.new("P", (frame_size[0] * frames, frame_size[1]), 0)
        sheet.putpalette(PAL_IMG.getpalette())
        for frame in range(frames):
            im = base.convert("RGBA")
            if frame:
                im = ImageChops.offset(im, frame % 2, 0)
            if filename == "explosion.png" and frame == 1:
                im = im.resize((frame_size[0] + 4, frame_size[1] + 4), Image.Resampling.NEAREST)
                fixed = Image.new("RGBA", frame_size, (0, 0, 0, 0))
                fixed.alpha_composite(im, (-2, -2))
                im = fixed
            sheet.paste(quantize(im, opaque_zero_index=11), (frame * frame_size[0], 0))
        sheet.save(out / filename)


def write_preview(out: Path, preview: Path) -> None:
    rows: list[tuple[str, Image.Image]] = []
    for name in ["monsters.png", "enemy.png", "tank.png", "helicopter.png", "person.png", "shot.png", "explosion.png"]:
        im = Image.open(out / name).convert("RGBA")
        if name == "monsters.png":
            im = im.crop((0, 0, 48 * 14 * 3, 56))
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
    parser = argparse.ArgumentParser(description="Convert high-res Vandaler source art into SGDK indexed sprite sheets.")
    parser.add_argument("--source", type=Path, default=DEFAULT_SOURCE, help="High-resolution source sheet.")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT, help="Output directory for SGDK sprite PNGs.")
    parser.add_argument("--preview", type=Path, default=None, help="Optional preview PNG path.")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    source = args.source.resolve()
    out = args.out.resolve()
    if not source.exists():
        raise SystemExit(f"Missing source sheet: {source}")
    out.mkdir(parents=True, exist_ok=True)
    src = Image.open(source)
    make_monsters(src, out)
    make_objects(src, out)
    if args.preview is not None:
        write_preview(out, args.preview.resolve())


if __name__ == "__main__":
    main()
