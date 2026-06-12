#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageOps

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "art/source/objects"
AI_SHEET = ROOT / "art/source/objects_ai_sheet_01.png"


def remove_magenta(im: Image.Image) -> Image.Image:
    rgba = im.convert("RGBA")
    px = rgba.load()
    for y in range(rgba.height):
        for x in range(rgba.width):
            r, g, b, a = px[x, y]
            if r > 205 and g < 85 and b > 185:
                px[x, y] = (0, 0, 0, 0)
            else:
                px[x, y] = (r, g, b, a)
    alpha = rgba.getchannel("A").filter(ImageFilter.MinFilter(3)).filter(ImageFilter.MaxFilter(3))
    rgba.putalpha(alpha)
    return rgba


def trim(im: Image.Image, padding: int = 4) -> Image.Image:
    bbox = im.getchannel("A").getbbox()
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


def fit(src: Image.Image, frame_w: int, frame_h: int, *, margin: int = 0, y_offset: int = 0) -> Image.Image:
    src = trim(src)
    max_w = max(1, frame_w - margin * 2)
    max_h = max(1, frame_h - margin * 2)
    fitted = ImageOps.contain(src, (max_w, max_h), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (frame_w, frame_h), (0, 0, 0, 0))
    bbox = fitted.getchannel("A").getbbox()
    if bbox is None:
        return canvas
    x = (frame_w - fitted.width) // 2
    y = frame_h - bbox[3] - margin + y_offset
    canvas.alpha_composite(fitted, (x, y))
    return canvas


def crop(sheet: Image.Image, box: tuple[int, int, int, int]) -> Image.Image:
    return remove_magenta(sheet.crop(box))


def sheet_from_frames(frames: list[Image.Image], frame_w: int, frame_h: int) -> Image.Image:
    out = Image.new("RGBA", (frame_w * len(frames), frame_h), (0, 0, 0, 0))
    for i, frame in enumerate(frames):
        out.alpha_composite(frame, (i * frame_w, 0))
    return out


def add_muzzle_flash(frame: Image.Image, x: int, y: int) -> Image.Image:
    out = frame.copy()
    draw = ImageDraw.Draw(out)
    draw.polygon([(x, y), (x + 5, y - 2), (x + 9, y), (x + 5, y + 2)], fill=(255, 196, 0, 255))
    draw.point((x + 8, y), fill=(255, 32, 0, 255))
    draw.point((x + 4, y), fill=(255, 255, 255, 255))
    return out


def main() -> None:
    if not AI_SHEET.exists():
        raise SystemExit(f"Missing AI sprite sheet: {AI_SHEET}")

    OUT.mkdir(parents=True, exist_ok=True)
    sheet = Image.open(AI_SHEET).convert("RGBA")

    enemy_boxes = [
        (84, 62, 255, 276),
        (440, 70, 650, 276),
        (795, 65, 1070, 255),
    ]
    enemy = [fit(crop(sheet, box), 24, 16, margin=1) for box in enemy_boxes]
    sheet_from_frames(enemy, 24, 16).save(OUT / "enemy.png")

    tank_boxes = [
        (55, 330, 410, 535),
        (525, 330, 880, 535),
        (980, 330, 1395, 535),
    ]
    tank = [fit(crop(sheet, box), 24, 16, margin=0) for box in tank_boxes]
    sheet_from_frames(tank, 24, 16).save(OUT / "tank.png")

    heli_boxes = [
        (50, 590, 370, 780),
        (420, 590, 745, 780),
        (750, 585, 1115, 785),
        (1160, 585, 1510, 780),
    ]
    heli = [fit(crop(sheet, box), 32, 16, margin=0) for box in heli_boxes]
    sheet_from_frames(heli, 32, 16).save(OUT / "helicopter.png")

    person = fit(crop(sheet, (155, 825, 260, 950)), 8, 16, margin=0)
    person.save(OUT / "person.png")

    shot = fit(crop(sheet, (420, 830, 625, 925)), 8, 8, margin=0)
    shot.save(OUT / "shot.png")

    explosion_a = fit(crop(sheet, (760, 790, 1035, 980)), 16, 16, margin=0)
    explosion_b = fit(crop(sheet, (1130, 770, 1480, 995)), 16, 16, margin=0)
    sheet_from_frames([explosion_a, explosion_b], 16, 16).save(OUT / "explosion.png")


if __name__ == "__main__":
    main()
