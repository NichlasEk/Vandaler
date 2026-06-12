#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageOps

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "art/source/objects"
AI_SHEET = ROOT / "art/source/objects_ai_sheet_01.png"
WALK_SHEET = ROOT / "art/source/walk_ai_sheet_01.png"


def remove_magenta(im: Image.Image) -> Image.Image:
    rgba = im.convert("RGBA")
    px = rgba.load()
    for y in range(rgba.height):
        for x in range(rgba.width):
            r, g, b, a = px[x, y]
            if r > 150 and b > 150 and g < 120 and r > (g * 2) and b > (g * 2):
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


def repair_tiny_frames(frames: list[Image.Image], min_area: int) -> list[Image.Image]:
    out: list[Image.Image] = []
    for i, frame in enumerate(frames):
        bbox = frame.getchannel("A").getbbox()
        area = 0 if bbox is None else (bbox[2] - bbox[0]) * (bbox[3] - bbox[1])
        if area < min_area and out:
            frame = ImageChops.offset(out[-1], 1 if (i & 1) else -1, 0)
        out.append(frame)
    return out


def offset_no_wrap(frame: Image.Image, dx: int, dy: int = 0) -> Image.Image:
    out = Image.new("RGBA", frame.size, (0, 0, 0, 0))
    out.alpha_composite(frame, (dx, dy))
    return out


def write_monster_walks(walk_sheet: Image.Image) -> None:
    rows = {
        "jonny": [
            (85, 35, 340, 218),
            (425, 35, 690, 218),
            (750, 35, 1015, 218),
            (1070, 35, 1335, 218),
        ],
        "conny": [
            (80, 230, 350, 405),
            (420, 230, 695, 405),
            (740, 230, 1020, 405),
            (1060, 230, 1335, 405),
        ],
        "bettan": [
            (80, 410, 365, 595),
            (420, 410, 710, 595),
            (735, 410, 1035, 595),
            (1060, 410, 1350, 595),
        ],
    }
    poses = ("walk_a", "walk_b", "walk_c", "walk_d")
    for monster, boxes in rows.items():
        out_dir = ROOT / "art/source/monsters" / monster
        out_dir.mkdir(parents=True, exist_ok=True)
        for pose, box in zip(poses, boxes):
            frame = trim(crop(walk_sheet, box), padding=8)
            frame = frame.resize((max(1, int(frame.width * 0.50)), frame.height), Image.Resampling.BICUBIC)
            scale = 430 / frame.height
            frame = frame.resize((max(1, int(frame.width * scale)), 430), Image.Resampling.BICUBIC)
            frame.save(out_dir / f"{pose}.png")


def main() -> None:
    if not AI_SHEET.exists():
        raise SystemExit(f"Missing AI sprite sheet: {AI_SHEET}")
    if not WALK_SHEET.exists():
        raise SystemExit(f"Missing AI walk sheet: {WALK_SHEET}")

    OUT.mkdir(parents=True, exist_ok=True)
    sheet = Image.open(AI_SHEET).convert("RGBA")
    walk_sheet = Image.open(WALK_SHEET).convert("RGBA")
    write_monster_walks(walk_sheet)

    enemy_boxes = [
        (110, 575, 285, 720),
        (450, 575, 635, 720),
        (780, 575, 975, 720),
    ]
    enemy = repair_tiny_frames([fit(crop(walk_sheet, box), 24, 16, margin=1) for box in enemy_boxes], min_area=70)
    enemy.append(offset_no_wrap(enemy[1], 1))
    sheet_from_frames(enemy, 24, 16).save(OUT / "enemy.png")

    tank_boxes = [
        (70, 855, 385, 1010),
        (405, 855, 725, 1010),
        (740, 855, 1065, 1010),
    ]
    tank = repair_tiny_frames([fit(crop(walk_sheet, box), 24, 16, margin=0) for box in tank_boxes], min_area=120)
    tank.append(offset_no_wrap(tank[1], 1))
    sheet_from_frames(tank, 24, 16).save(OUT / "tank.png")

    heli_boxes = [
        (50, 590, 370, 780),
        (420, 590, 745, 780),
        (750, 585, 1115, 785),
        (1160, 585, 1510, 780),
    ]
    heli = [fit(crop(sheet, box), 32, 16, margin=0) for box in heli_boxes]
    sheet_from_frames(heli, 32, 16).save(OUT / "helicopter.png")

    person_boxes = [
        (120, 680, 305, 825),
        (455, 680, 640, 825),
        (780, 680, 970, 825),
        (1110, 680, 1305, 825),
    ]
    person = repair_tiny_frames([fit(crop(walk_sheet, box), 8, 16, margin=0) for box in person_boxes], min_area=30)
    sheet_from_frames(person, 8, 16).save(OUT / "person.png")

    shot = fit(crop(sheet, (420, 830, 625, 925)), 8, 8, margin=0)
    shot.save(OUT / "shot.png")

    explosion_a = fit(crop(sheet, (760, 790, 1035, 980)), 16, 16, margin=0)
    explosion_b = fit(crop(sheet, (1130, 770, 1480, 995)), 16, 16, margin=0)
    sheet_from_frames([explosion_a, explosion_b], 16, 16).save(OUT / "explosion.png")


if __name__ == "__main__":
    main()
