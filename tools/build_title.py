#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageEnhance, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
TITLE_W = 320
TITLE_H = 224


def snap_rgb333(im: Image.Image) -> Image.Image:
    src = im.convert("RGB")
    out = Image.new("RGB", src.size)
    in_px = src.load()
    out_px = out.load()
    for y in range(src.height):
        for x in range(src.width):
            r, g, b = in_px[x, y]
            out_px[x, y] = (
                round(r / 36.4286) * 36,
                round(g / 36.4286) * 36,
                round(b / 36.4286) * 36,
            )
    return out


def crop_to_aspect(im: Image.Image, width: int, height: int) -> Image.Image:
    src_w, src_h = im.size
    target = width / height
    current = src_w / src_h
    if current > target:
        new_w = int(src_h * target)
        left = (src_w - new_w) // 2
        return im.crop((left, 0, left + new_w, src_h))

    new_h = int(src_w / target)
    top = (src_h - new_h) // 2
    return im.crop((0, top, src_w, top + new_h))


def restore_select_accents(im: Image.Image) -> Image.Image:
    out = im.copy()
    px = out.load()
    for x in (242, 249):
        for dy in range(2):
            for dx in range(2):
                px[x + dx, 47 + dy] = (232, 232, 232)
                px[x + dx + 1, 49 + dy] = (72, 72, 112)
    return out


def remove_select_prompt(im: Image.Image) -> Image.Image:
    out = im.copy()
    draw = ImageDraw.Draw(out)
    draw.rectangle((28, 190, 292, 223), fill=(0, 8, 36))
    draw.rectangle((32, 194, 288, 219), outline=(16, 32, 96))
    draw.rectangle((36, 198, 284, 215), outline=(0, 0, 24))
    return out


def build_screen(source: Path, out: Path, *, select_screen: bool = False) -> None:
    im = Image.open(source).convert("RGB")
    im = crop_to_aspect(im, TITLE_W, TITLE_H)
    im = im.resize((TITLE_W, TITLE_H), Image.Resampling.LANCZOS)
    im = ImageEnhance.Color(im).enhance(1.35 if select_screen else 1.22)
    im = ImageEnhance.Contrast(im).enhance(1.12)
    im = ImageEnhance.Brightness(im).enhance(1.08 if select_screen else 1.04)
    im = im.filter(ImageFilter.UnsharpMask(radius=0.6, percent=120, threshold=2))
    if select_screen:
        im = remove_select_prompt(im)
        im = restore_select_accents(im)
    im = snap_rgb333(im)
    indexed = im.quantize(colors=16, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.FLOYDSTEINBERG)
    palette = indexed.getpalette()[:48]
    md_palette = []
    for i in range(0, len(palette), 3):
        md_palette.extend((round(palette[i] / 36.4286) * 36, round(palette[i + 1] / 36.4286) * 36, round(palette[i + 2] / 36.4286) * 36))
    indexed.putpalette(md_palette + [0] * (768 - len(md_palette)))
    indexed = indexed.convert("RGB").quantize(palette=indexed, dither=Image.Dither.FLOYDSTEINBERG)
    palette = indexed.getpalette()[:48]
    indexed.putpalette(palette + [0] * (768 - len(palette)))
    out.parent.mkdir(parents=True, exist_ok=True)
    indexed.save(out)


def main() -> None:
    build_screen(ROOT / "art/source/title/vandaler_title_source_01.png", ROOT / "res/images/title.png")
    build_screen(ROOT / "art/source/title/vandaler_select_source_01.png", ROOT / "res/images/select.png", select_screen=True)


if __name__ == "__main__":
    main()
