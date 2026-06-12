# Vandaler

En SGDK/Mega Drive-prototyp inspirerad av stadsförstörelse-arcade: tre valbara monster, svenska städer, skada på byggnader och enkla PSG-ljudeffekter.

## Bygg

```sh
make
```

ROM: `out/rom.bin`

## Kontroller

- D-pad: gå och klättra när monstret står vid sidan av en byggnad
- A: hoppa
- B/C: slå
- Start: välj / starta / gå vidare

Polisbilar patrullerar gatan och skjuter mot monstret. Slå dem eller deras skott för bonuspoäng; tar hälsan slut visas game over.

Svenska `ÅÄÖ` i speltext ritas med en liten UTF-8-wrapper ovanpå SGDK:s ASCII-font.
HUD:en visar score, liv och aktuell stad på en mörk panel högst upp.
Spelarmonstret körs via SGDK:s sprite engine från `res/images/monsters.png`.

Original-ROM:en `rampage.sms` ignoreras av git och används inte av koden.
