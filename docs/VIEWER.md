# Viewer — snaps · mesh 3D · G-code · robot jog (H5 + H2-6)

Loopback `cadre view` stays a tiny HTTP server (no new crate).

## Targets

| Path | Kind | Artifact |
|------|------|----------|
| `.snap/` / `.cad.star` | star/snap | multi-view PNG + GIF + **`mesh.json`** (H2-6) |
| `.gcode` / `.gco` / `.nc` | gcode | `{stem}.view/path.json` + XY scrub **+ 3D path orbit** |
| `.robot.json` | robot | `{stem}.view/robot.json` + joint sliders (**3D stick FK**) |
| `.urdf` | — | validates only; jog needs `.robot.json` |

## Commands

```sh
# CI-friendly prepare
cargo run -p cadre-cli -- view examples/studio/stellar_crown.cad.star --once --json
cargo run -p cadre-cli -- view examples/fab/sample.gcode --once --json
cargo run -p cadre-cli -- view examples/robots/simple_arm.robot.json --once --json

# Serve (browser)
cargo run -p cadre-cli -- view \
  examples/studio/stellar_crown.cad.star \
  examples/fab/sample.gcode \
  examples/robots/simple_arm.robot.json
# open http://127.0.0.1:7411/
```

## H2-6 — coarse 3D

### Mesh (`.cad.star`)
- Preview mesh from `mesh_from_ir` written as `mesh.json` (positions + indices + bbox)
- Canvas painter’s algorithm, drag to orbit, backface cull
- Still shows static PNG/GIF snapshot grid below

### G-code
- Keeps layer XY scrub
- Adds second canvas: 3D path with orbit; layer slider filters cumulative points

### Robot
- Full 3D stick FK (4×4 matrices, axis-angle revolute / prismatic)
- Drag canvas to orbit; joint sliders with limits

## Honesty

- **Not Blender / not GLB / not STEP tessellation parity**
- Mock cut/polar preview mesh remains approximate
- Not a physics G-code simulator (no time, no extrusion dynamics)
- Robot is stick figure, not link meshes
- `--once` prepares artifacts without binding a port
