# Research Notes

## Rampage SMS ROM

The local `rampage.sms` file is treated as private reference material and is ignored by git.

- File size: 262656 bytes.
- This is 256 KiB plus a 512 byte copier/header prefix.
- The Master System `TMR SEGA` header appears at file offset `0x81f0`, which maps to ROM offset `0x7ff0` after skipping the first `0x200` bytes.
- Z80 code appears to start at file offset `0x200`.
- A visible string table includes `SAN FRANCISCO` at file offset `0xbff4`.
- HUD strings near file offset `0xbfaa`: `PLAYER`, `DAMAGE`, `SCORE     0`, `HIGH SCORE`.
- The city-name pointer table starts at ROM offset `0xbde0` / file offset `0xbfe0`.
- The SMS city table has ten entries:
  - `SAN FRANCISCO-`
  - `LOS ANGELES-`
  - `LAS VEGAS-`
  - `DALLAS-`
  - `ST.LOUIS-`
  - `CHICAGO-`
  - `DETROIT-`
  - `BALTIMORE-`
  - `PHILADELPHIA-`
  - `NEW YORK-`
- Code around ROM offset `0x765b` increments a day/progress counter and compares it with `0x32` decimal 50. Combined with ten city names, this suggests a 50-day route, likely five days per city.
- Original HUD emphasizes building `DAMAGE` alongside score/high score. Vandaler should probably track city/building destruction explicitly, not only player health.

Vandaler city route proposal:

- `MALMÖ`
- `LINKÖPING`
- `ÖREBRO`
- `VÄSTERÅS`
- `STOCKHOLM`
- `UPPSALA`
- `GÄVLE`
- `SUNDSVALL`
- `UMEÅ`
- `LULEÅ`

Allowed use for Vandaler:

- Study game rules, state flow, pacing, collision behavior, scoring ranges, and level structure.
- Recreate mechanics in original SGDK code.

Avoid:

- Copying original game code.
- Copying original graphics, tile data, music, or sound effects.
- Shipping `rampage.sms` or generated disassembly output.
