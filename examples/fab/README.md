# Fab path examples (S11)

```sh
# DXF plate + holes
cargo run -p cadre-cli -- fab dxf --width 100 --height 50 \
  --hole 25,25,6 --hole 75,25,6 -o /tmp/plate.dxf --json

# Face → DXF from a part (largest +Z face outline)
cargo run -p cadre-cli -- fab dxf-face parity/parts/01_calibration_block/part.cad.star \
  --normal 0,0,1 -o /tmp/face.dxf --json

# DFM preflight (bundled SendCutSend-style profile)
cargo run -p cadre-cli -- fab check --part-json examples/fab/plate.flat.json --json

# Slicer discovery + command preview
cargo run -p cadre-cli -- fab slicers --json
cargo run -p cadre-cli -- fab slice mesh.stl --json

# G-code static check
cargo run -p cadre-cli -- fab gcode-check examples/fab/sample.gcode --json

# Printer dry-run (no network) + gated start
cargo run -p cadre-cli -- printer dry-run examples/fab/sample.gcode --json
# start always refused for live print in S11; gates still run:
cargo run -p cadre-cli -- printer start examples/fab/sample.gcode \
  --sha256 <from dry-run> --confirm START --allowlist bambu:x1c-01 --json
```

Safety: `printer start` requires allow-list + hash match + confirm=START, and S11 still
refuses live MQTT start by design.
