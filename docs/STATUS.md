# Cadre-RS — live status (post S12 / v1 surface)

> AI-oriented status block. Prefer this + `docs/METRICS.md` + `BACKLOG.md` over README narrative.

**As of:** 2026-08-05 · **tip:** `main` @ S12 merge · **version:** 0.1.0  
**agent_id:** `CADRE` · **repo:** https://github.com/buckster123/Cadre-RS

## Ship state
- **v1 surface (M0–M6 / S0–S12): COMPLETE** on default (mock) path
- CI: ubuntu + windows, OCCT-free default workspace
- Binary: `cadre` (`cadre-cli`), mock kernel default; OCCT optional feature

## Crate map (as-built)
| Crate | Role |
|-------|------|
| cadre | facade |
| cadre-kernel | GeomKernel + MockKernel |
| cadre-occt | OCCT backend (LGPL, non-default CI) |
| cadre-lang | hermetic Starlark → IR + execute_ir |
| cadre-model | selectors + BuildCache |
| cadre-inspect | refs / measure |
| cadre-render | software z-buffer PNG + orbit GIF |
| cadre-bench | parity suite parts 1–4 |
| cadre-mcp | stdio MCP (6 tools) |
| cadre-api | Axum `/v1/*` + jobs/SSE/OpenAPI |
| cadre-parts | parts.lock + LocalFsProvider + AssemblySpec |
| cadre-robot | URDF/SRDF/SDF + urdf-rs |
| cadre-fab | DXF, DFM, slicer discover, gcode-check, Bambu dry-run |
| cadre-cli | clap binary |

**Not built (parked names from early design):** cadre-truck, cadre-export (export lives in cli/occt/render), cadre-viewer (view is CLI loopback HTML), cadre-skills (export in cli).

## CLI surface (high signal)
```
build | inspect refs|measure | export step|stl|glb
snapshot | view | bench run
mcp | skills export [--all]
serve api
robot gen|validate
fab dxf|check|slicers|slice|gcode-check
printer status|dry-run|start
version --json
```

## Examples
- `parity/parts/01..04` — geometry fixtures
- `examples/assembly/` — plate + bolt lock
- `examples/robots/simple_arm.robot.json`
- `examples/fab/` — DFM plate + sample gcode

## Green bar (METRICS 1–16)
See `docs/METRICS.md`. Amber/red remaining: live OCCT e2e, live Bambu start, face→DXF B-rep, harness score.

## Post-v1 candidates (not committed)
1. Deeper OCCT topology/parity goldens
2. Live Bambu FTPS/MQTT behind same gates
3. Face projection DXF from selectors
4. Streamable-HTTP MCP
5. Agent harness eval ≥6/10
6. Parts 5–10 parity

## Honesty defaults
- Mock ≠ OCCT; STEP needs `--features occt` + `--kernel occt`
- Snapshot cut preview keeps operand A
- Printer start: allowlist + sha256 + `confirm=START`; S11/S12 still no live start
- DFM = profile-version truth, not vendor API
