#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from PIL import Image

from convert_source_sheet import lasso_subject, transparent_border, trim_alpha

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "art/source/vandaler_source_sheet_01.png"

MONSTERS = [
    ("jonny", (14, 186)),
    ("conny", (205, 365)),
    ("bettan", (382, 536)),
]

POSES = {
    "idle": (0, 145),
    "walk_a": (150, 275),
    "walk_b": (285, 410),
    "climb": (392, 492),
    "punch_forward": (535, 665),
    "punch_up": (675, 765),
    "punch_diag_up": (775, 895),
    "punch_diag_down": (905, 1030),
}

OBJECTS = {
    "enemy": (8, 742, 220, 815),
    "tank": (255, 736, 450, 815),
    "helicopter": (490, 727, 724, 822),
    "person": (12, 546, 80, 625),
    "shot": (14, 826, 62, 862),
    "explosion": (20, 870, 360, 1005),
}


def clean_crop(src: Image.Image, box: tuple[int, int, int, int], padding: int, min_area: int) -> Image.Image:
    crop = transparent_border(src.crop(box))
    crop = lasso_subject(crop, min_area)
    return trim_alpha(crop, padding)


def main() -> None:
    if not SOURCE.exists():
        raise SystemExit(f"Missing source sheet: {SOURCE}")

    src = Image.open(SOURCE).convert("RGBA")
    for monster, ybox in MONSTERS:
        out_dir = ROOT / "art/source/monsters" / monster
        out_dir.mkdir(parents=True, exist_ok=True)
        for pose, xbox in POSES.items():
            box = (xbox[0], ybox[0], xbox[1], ybox[1])
            clean_crop(src, box, padding=12, min_area=18).save(out_dir / f"{pose}.png")

    obj_dir = ROOT / "art/source/objects"
    obj_dir.mkdir(parents=True, exist_ok=True)
    for name, box in OBJECTS.items():
        clean_crop(src, box, padding=4, min_area=10).save(obj_dir / f"{name}.png")


if __name__ == "__main__":
    main()
