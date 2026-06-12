# Vandaler

En SGDK/Mega Drive-prototyp inspirerad av stadsförstörelse-arcade: tre valbara monster, svenska städer, skada på byggnader och enkla PSG-ljudeffekter.

## Bygg

```sh
make
```

ROM: `out/rom.bin`

## Kontroller

- D-pad: gå och klättra
- A: hoppa
- B/C: slå
- Start: välj / starta / gå vidare

Svenska `ÅÄÖ` i speltext ritas med en liten UTF-8-wrapper ovanpå SGDK:s ASCII-font.

Original-ROM:en `rampage.sms` ignoreras av git och används inte av koden.
