# H5 — Viewer depth (G-code scrub + URDF jog)

Loopback `cadre view` stays a tiny HTTP server (no new crate).

## Targets

| Path | Kind | Artifact |
|------|------|----------|
| `.snap/` / `.cad.star` | snap | multi-view PNG + GIF (unchanged) |
| `.gcode` / `.gco` / `.nc` | gcode | `{stem}.view/path.json` + canvas layer scrub |
| `.robot.json` | robot | `{stem}.view/robot.json` + joint sliders (2D FK alpha) |
| `.urdf` | — | validates only; jog needs `.robot.json` |

## Commands

```sh
# CI-friendly prepare
cargo run -p cadre-cli -- view examples/fab/sample.gcode --once --json
cargo run -p cadre-cli -- view examples/robots/simple_arm.robot.json --once --json

# Serve (browser)
cargo run -p cadre-cli -- view \
  examples/fab/sample.gcode \
  examples/robots/simple_arm.robot.json
# open http://127.0.0.1:7411/
```

## G-code scrub

- `cadre_fab::extract_gcode_path` → points + Z layers (±0.05 mm)
- Blue segments = extrude; grey = travel
- Layer slider filters polyline

## URDF jog alpha

- `cadre_robot::jog_payload` from `RobotSpec`
- 2D stick FK (planar bias); not mesh/GLB
- Revolute/prismatic sliders with joint limits

## Honesty

- Not a full 3D robot viewer or G-code simulator (no time, no extrusion physics)
- JSON deep links via `/v/{i}/path.json` and `/v/{i}/robot.json`
- `--once` prepares artifacts without binding a port
