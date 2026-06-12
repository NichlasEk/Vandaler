# Sprite Lab

`tools/sprite_lab.py` is a non-destructive gate for AI sprite candidates. It audits
candidate sheets before anything is copied into `art/source/monsters` or
`art/source/objects`.

Current baseline:

```sh
python tools/sprite_lab.py audit-current
```

Audit a generated monster grid:

```sh
python tools/sprite_lab.py audit-grid \
  --sheet art/source/generated/new_monster_walks.png \
  --kind monsters \
  --rows 3 \
  --cols 4 \
  --labels jonny,conny,bettan \
  --key ff00ff \
  --boost
```

Audit a generated object grid:

```sh
python tools/sprite_lab.py audit-grid \
  --sheet art/source/generated/new_object_walks.png \
  --kind objects \
  --rows 3 \
  --cols 4 \
  --labels enemy,person,tank \
  --key ff00ff \
  --boost
```

The tool rejects frames for clipping, bad alpha area, color drift, low
saturation, and flat contrast. Red frames should not be imported.
