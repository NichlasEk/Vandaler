# Vandaler Art Pipeline

This directory holds high-resolution source art used to generate Mega Drive
assets for SGDK.

## Current Flow

1. Generate or edit isolated high-resolution pose/object source PNGs.
2. Save monster pose sources under `art/source/monsters/<monster>/`.
3. Save object sources under `art/source/objects/`.
4. Run:

   ```sh
   tools/assets.sh preview
   ```

5. Inspect `art/source/vandaler_converted_preview.png`.
6. Adjust `art/assets.json`, source PNGs, or `tools/build_sprites.py` cleanup settings if needed.
7. Run:

   ```sh
   tools/assets.sh build
   make
   ```

The generated runtime sprites are written to `res/images/` and consumed by
`res/resources.res`.

The Makefile also exposes `make assets-preview` and `make assets` as shortcuts.

## Instructions For Future Codex

The stable path is isolated source art per pose, not rectangle-cropping a big
busy sheet. Do not spend time trying to make one giant sheet perfectly crop.
Use that only for bootstrapping.

Preferred generation loop:

1. Generate one horizontal pose sheet for one monster at a time.
2. Use a perfectly flat chroma-key background:
   - `#00ff00` for Jonny/Conny.
   - `#ff00ff` for Bettan or any green-heavy source.
3. Prompt for exactly eight isolated full-body poses in this order:
   `idle`, `walk_a`, `walk_b`, `climb`, `punch_forward`, `punch_up`,
   `punch_diag_up`, `punch_diag_down`.
4. Require: no wall, no pipe, no building pieces, no props, no impact flashes,
   no text, no labels, no shadows, no floor, no neighboring characters.
5. Save the generated sheet under `art/source/generated/`.
6. Import it:

   ```sh
   python tools/import_pose_sheet.py art/source/generated/jonny_pose_sheet_XX.png --monster jonny --key 00ff00
   python tools/import_pose_sheet.py art/source/generated/conny_pose_sheet_XX.png --monster conny --key 00ff00
   python tools/import_pose_sheet.py art/source/generated/bettan_pose_sheet_XX.png --monster bettan --key ff00ff
   ```

7. Build and preview:

   ```sh
   tools/assets.sh preview
   make
   ```

What to inspect:

- If a pose has a loose fragment, fix/regenerate that single pose source rather
  than weakening the global lasso too much.
- Monster poses are normalized per character in `tools/build_sprites.py`: idle
  and walk poses set the standing scale, then all poses for that monster reuse
  that scale and are alpha-bottom anchored. This avoids climb/punch poses being
  resized independently and prevents transparent crop padding from making the
  player float.
- If a pose touches the frame edge, lower scale/margin or edit the source.
- If dark body pixels become transparent, check that index `0` is still reserved
  only for true alpha in `tools/convert_source_sheet.py`.
- If colors look wrong in a debug preview, remember monster sprites use role
  indices and are recolored by PAL1/PAL2/PAL3 in-game; PAL0 previews can look
  misleading.
- Keep `48x56` monster frames unless code constants and collision assumptions
  are deliberately updated together.

Current known rough edges:

- Some AI-generated poses can include tiny detached fragments. The pipeline can
  remove obvious fragments, but high-quality source poses are still better.
- Punch poses may vary in silhouette more than hand-drawn sprite sheets. Prefer
  regenerating only the bad pose source when this happens.
- Object sprites are still bootstrapped from the large source sheet and should
  eventually get their own isolated source sheets too.

## Bootstrap From A Sheet

The first separated pose files were bootstrapped from
`art/source/vandaler_source_sheet_01.png`.

To regenerate those initial files:

```sh
tools/assets.sh extract
```

This is a convenience step only. The stable workflow is to replace individual
pose files directly, for example:

```text
art/source/monsters/jonny/climb.png
art/source/monsters/jonny/punch_forward.png
art/source/monsters/conny/walk_a.png
```

Those isolated pose files should use transparent or flat chroma-key backgrounds,
with no wall, props, impact flashes, neighboring frames, text, or shadows baked
into the source unless that detail is part of the sprite itself.

## Constraints

- Sprites are indexed 4bpp PNGs for SGDK.
- Monster frames must stay `48x56`, 14 frames per monster, 3 monsters.
- Enemy car and tank frames are `24x16`.
- Helicopter frames are `32x16`.
- Civilians are `8x16`.
- Projectile frames are `8x8`.
- Explosion frames are `16x16`.

## Toolchain Files

- `art/assets.json` maps separated source files to runtime frame order.
- `tools/extract_initial_sources.py` bootstraps separated sources from the first large sheet.
- `tools/build_sprites.py` builds SGDK indexed PNGs from separated sources.
- `tools/convert_source_sheet.py` contains shared cleanup, lasso, palette, and quantization helpers.

## Current Source Prompt

The first source sheet was generated with the built-in image generation tool
using this prompt:

```text
Create an original high-quality 16-bit pixel art source sheet for a
monster-destruction game set in a Swedish city. It should feel like premium
late-era Mega Drive arcade graphics: dense brickwork, crisp outlines, dramatic
shadows, rich but limited palette, readable silhouettes, no modern effects.

Include three original selectable monsters: a hulking brown ape, a gray
wolf-man, and a green lizard kaiju. Include each as full-body sprite poses:
idle, walk, climb, punch forward, punch upward, diagonal punch, jump/tail-bounce
style pose. Include small enemy assets: tiny civilians, police car, tank,
helicopter, red projectiles, explosion puffs.

Use a clean sprite and tile source sheet layout, assets separated with padding,
flat dark neutral background behind loose sprites, one compact city background
sample strip. No logos, no copyrighted characters, no readable text, no
watermark.
```
