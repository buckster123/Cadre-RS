# H8 — build123d → Cadre skeleton migrator (clean-room)

## Scope

Best-effort **structure + params** only. Not full semantic parity with build123d.

Input is treated as untrusted text. Refuse: `exec`/`eval`/`subprocess`/`open`/…

Shaped from **public** build123d-style APIs only — never third-party private sources.

## CLI

```sh
cargo run -p cadre-cli -- migrate fixtures/migrate/01_simple_box.py --json
cargo run -p cadre-cli -- migrate fixtures/migrate/02_plate_hole.py -o /tmp/plate.cad.star --json
cargo run -p cadre-cli -- build /tmp/plate.cad.star --json
```

## Fixtures

| File | Intent |
|------|--------|
| `fixtures/migrate/01_simple_box.py` | Box + params |
| `fixtures/migrate/02_plate_hole.py` | Box + Cylinder |
| `fixtures/migrate/03_kwargs_sphere.py` | kwargs Box + Sphere |

## Honesty

- Placements / Locations / fillets / workplanes not reconstructed
- Multiple solids → union (or cut if subtract markers seen)
- Always review the skeleton before fab
