# Fab path examples (S11+)

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

# Printer dry-run (no network) — prints sha256 for the start gate
cargo run -p cadre-cli -- printer dry-run examples/fab/sample.gcode --json

# Gates only (still no network — missing --live)
cargo run -p cadre-cli -- printer start examples/fab/sample.gcode \
  --sha256 <from dry-run> --confirm START --allowlist bambu:x1c-01 --json

# LIVE start (YOU opt in): FTPS upload + MQTT after all gates
# Needs: curl, mosquitto_pub, LAN access code, printer serial
export CADRE_BAMBU_ACCESS_CODE=xxxxxxxx
export CADRE_BAMBU_SERIAL=01P00A000000000
cargo run -p cadre-cli -- printer start examples/fab/sample.gcode \
  --sha256 <from dry-run> --confirm START --allowlist bambu:x1c-01 \
  --host 192.168.1.50 --live --json
```

## Safety gates (all required before any network)

| Gate | How you open it |
|------|-----------------|
| **allowlist** | `--allowlist bambu:x1c-01` (your printer id) |
| **sha256** | must match file; copy from `printer dry-run` |
| **confirm** | exactly `--confirm START` (case-sensitive) |
| **gcode-check** | static validation must pass |
| **`--live`** | second consent: without it, gates may pass but **no sockets** |
| **credentials** | `--access-code` / `CADRE_BAMBU_ACCESS_CODE` + `--serial` / `CADRE_BAMBU_SERIAL` |

Live transport shells to `curl` (FTPS, `-k` for self-signed) and `mosquitto_pub` (MQTT 8883).
Community LAN protocol — label accordingly; printer firmware drift is possible.
