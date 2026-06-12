#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageEnhance

ROOT = Path(__file__).resolve().parents[1]
MONSTER_ROOT = ROOT / "art/source/monsters"
OBJECT_ROOT = ROOT / "art/source/objects"


def rgba(path: Path) -> Image.Image:
    return Image.open(path).convert("RGBA")


def boost(im: Image.Image, *, color: float = 1.18, contrast: float = 1.08, brightness: float = 1.02) -> Image.Image:
    alpha = im.getchannel("A")
    rgb = im.convert("RGB")
    rgb = ImageEnhance.Color(rgb).enhance(color)
    rgb = ImageEnhance.Contrast(rgb).enhance(contrast)
    rgb = ImageEnhance.Brightness(rgb).enhance(brightness)
    out = rgb.convert("RGBA")
    out.putalpha(alpha)
    return out


def offset_no_wrap(im: Image.Image, dx: int = 0, dy: int = 0) -> Image.Image:
    out = Image.new("RGBA", im.size, (0, 0, 0, 0))
    out.alpha_composite(im, (dx, dy))
    return out


def alpha_bbox(im: Image.Image) -> tuple[int, int, int, int] | None:
    return im.getchannel("A").getbbox()


def stable_bob(im: Image.Image, *, dx: int, dy: int, split: float) -> Image.Image:
    """Move the upper body slightly while keeping feet planted.

    This creates a distinct in-between frame without changing identity, size, or
    palette source. It deliberately avoids wraparound so no edge artifacts appear.
    """
    bbox = alpha_bbox(im)
    if bbox is None:
        return im.copy()
    split_y = int(round(bbox[1] + (bbox[3] - bbox[1]) * split))
    upper = im.crop((0, 0, im.width, split_y))
    lower = im.crop((0, split_y, im.width, im.height))
    out = Image.new("RGBA", im.size, (0, 0, 0, 0))
    out.alpha_composite(upper, (dx, dy))
    out.alpha_composite(lower, (0, split_y))
    return out


def split_sheet(im: Image.Image, frame_w: int, frame_h: int) -> list[Image.Image]:
    if im.height != frame_h or im.width % frame_w != 0:
        raise SystemExit(f"Expected exact sheet size Nx{frame_w} x {frame_h}, got {im.size}")
    return [im.crop((x, 0, x + frame_w, frame_h)) for x in range(0, im.width, frame_w)]


def sheet(frames: list[Image.Image], frame_w: int, frame_h: int) -> Image.Image:
    out = Image.new("RGBA", (frame_w * len(frames), frame_h), (0, 0, 0, 0))
    for i, frame in enumerate(frames):
        out.alpha_composite(frame, (i * frame_w, 0))
    return out


def write_monster_cycles() -> None:
    for monster in ("jonny", "conny", "bettan"):
        root = MONSTER_ROOT / monster
        walk_a = boost(rgba(root / "walk_a.png"))
        walk_b = boost(rgba(root / "walk_b.png"))
        # Keep the two trusted poses, then add conservative in-betweens.
        walk_c = stable_bob(walk_a, dx=1, dy=-1, split=0.72)
        walk_d = stable_bob(walk_b, dx=-1, dy=-1, split=0.72)
        walk_a.save(root / "walk_a.png")
        walk_b.save(root / "walk_b.png")
        walk_c.save(root / "walk_c.png")
        walk_d.save(root / "walk_d.png")


def write_object_cycles() -> None:
    enemy_frames = [boost(frame, color=1.25, contrast=1.12) for frame in split_sheet(rgba(OBJECT_ROOT / "enemy.png"), 24, 16)]
    if len(enemy_frames) < 3:
        raise SystemExit("enemy.png needs at least 3 source frames")
    enemy = [enemy_frames[0], enemy_frames[1], enemy_frames[2], offset_no_wrap(enemy_frames[1], dx=-1)]
    sheet(enemy, 24, 16).save(OBJECT_ROOT / "enemy.png")

    tank_frames = [boost(frame, color=1.22, contrast=1.12) for frame in split_sheet(rgba(OBJECT_ROOT / "tank.png"), 24, 16)]
    if len(tank_frames) < 3:
        raise SystemExit("tank.png needs at least 3 source frames")
    tank = [tank_frames[0], tank_frames[1], tank_frames[2], tank_frames[1]]
    sheet(tank, 24, 16).save(OBJECT_ROOT / "tank.png")

    person_frames = split_sheet(rgba(OBJECT_ROOT / "person.png"), 8, 16)
    base = boost(person_frames[0], color=1.3, contrast=1.15)
    person = [
        base,
        stable_bob(base, dx=1, dy=-1, split=0.58),
        base,
        stable_bob(base, dx=-1, dy=-1, split=0.58),
    ]
    sheet(person, 8, 16).save(OBJECT_ROOT / "person.png")


def main() -> None:
    write_monster_cycles()
    write_object_cycles()


if __name__ == "__main__":
    main()
