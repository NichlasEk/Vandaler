# Research Notes

## Rampage SMS ROM

The local `rampage.sms` file is treated as private reference material and is ignored by git.

- File size: 262656 bytes.
- This is 256 KiB plus a 512 byte copier/header prefix.
- The Master System `TMR SEGA` header appears at file offset `0x81f0`, which maps to ROM offset `0x7ff0` after skipping the first `0x200` bytes.
- Z80 code appears to start at file offset `0x200`.
- A visible string table includes `SAN FRANCISCO` at file offset `0xbff4`.

Allowed use for Vandaler:

- Study game rules, state flow, pacing, collision behavior, scoring ranges, and level structure.
- Recreate mechanics in original SGDK code.

Avoid:

- Copying original game code.
- Copying original graphics, tile data, music, or sound effects.
- Shipping `rampage.sms` or generated disassembly output.

