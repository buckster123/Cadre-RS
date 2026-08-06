# H2-5 — Assembly joint depth

## What changed

### Assembly `JointSpec` (v2 fields)
| Field | Meaning |
|-------|---------|
| `kind` | `fixed` \| `revolute` \| `prismatic` |
| `axis` | parent-frame axis (default +Z) |
| `origin_mm` | joint origin in parent frame |
| `lower` / `upper` | **required** for revolute (rad) and prismatic (mm) |
| `effort` / `velocity` | optional; must be ≥ 0 if set |

### Fail-closed validation
`cadre_parts::validate_assembly` + CLI:

```sh
cargo run -p cadre-cli -- assembly validate examples/assembly/lid_hinge.assy.json --json
cargo run -p cadre-cli -- assembly validate examples/assembly/bad_limits.assy.json --json
# bad_limits → ok:false (lower > upper)
```

Also: unknown components, duplicate names, zero axis, unknown kind.

### Robot path (same doctrine)
`validate_robot` now **errors** (not warns) when revolute/prismatic lack limits, or `lower > upper`, or negative effort/velocity.

### Examples
| File | Expect |
|------|--------|
| `plate_bolt.assy.json` | fixed joint — still valid |
| `lid_hinge.assy.json` | good revolute with limits |
| `bad_limits.assy.json` | inverted prismatic limits — **must fail** |

## OQ-4 honesty

Still **not** AP242 kinematic STEP joints. This is labels + axis + limit envelope for agent fail-closed checks. Full PMI/AP242 remains Horizon parking / later.

## CHARTER
OQ-4 remains open for STEP depth; H2-5 closes the **assembly/robot limit envelope** bite.
