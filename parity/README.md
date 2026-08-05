# Parity suite

Deterministic geometry fixtures for Cadre-RS (PRD §12 Parity-10).

## Parts 1–4 (M1 / S6)

| Id | Part | Notes |
|----|------|--------|
| `01_calibration_block` | Plate + 2×2 holes | mock volume uses hole height as authored |
| `02_bolt_circle_flange` | Disc + bore + 6 bolts | |
| `03_l_bracket` | L + gusset + 2-dir clearances | vertical hole = slot (no rotate yet) |
| `04_stepped_shaft` | 3-step shaft + keyway | |

Each directory:

```
part.cad.star   # reference model
expect.json     # volume/bbox/ops/params/measures
```

## Run

```sh
cargo test -p cadre-bench
cargo run -p cadre-cli -- bench run --suite parts1-4 --json
```

Volumes are calibrated against **MockKernel** analytic booleans so default CI stays OCCT-free.
OCCT golden re-calibration is a follow-up when `parity-geom` lands.
