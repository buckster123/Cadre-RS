# Cadre-RS — live status

> AI-oriented status block. Prefer this + `docs/METRICS.md` + `BACKLOG.md` + `docs/HORIZON.md`.

**As of:** 2026-08-05 · **tip:** `main` @ post-v1 #14–#22 · **version:** 0.1.0  
**agent_id:** `CADRE` · **repo:** https://github.com/buckster123/Cadre-RS  
**kernels:** mock (default CI) · occt (`parts1-4-occt`, `parts5-10-occt`, `--features occt`)

## Ship state
- **v1 surface (M0–M6 / S0–S12): COMPLETE**
- **Post-v1 cook #14–#22: COMPLETE** (OCCT depth → transforms; face DXF; live Bambu; HTTP MCP; harness; inspect polish; parity 5–10)
- CI: ubuntu + windows, OCCT-free default workspace
- Binary: `cadre` (`cadre-cli`), mock default; OCCT optional

## Next board
See **[`docs/HORIZON.md`](HORIZON.md)** Top-N **H1–H10**. Default next: **H5 viewer G-code + URDF jog**.
H1 live harness: `harness run --suite agent10 --cmd '…'` (oracle: `@oracle` or `harness/drivers/oracle_agent.py`).
H2 stdlib: `sphere` / `cone` / `mirror` / `linear_pattern` / `polar_pattern` · IR v2 · example `examples/stdlib/pattern_hub.cad.star`.
H3 OCCT transforms: direct BRep (`third_party/opencascade` patch) — no STEP thrash on translate/rotate/mirror/clone/sphere.
H4 fillet/chamfer: OCCT `CADRE-E-FILLET-FAILED` + suite `fillet-occt` · mock stays Unsupported · `docs/FILLET_CHAMFER.md`.

## Crate map (as-built)
| Crate | Role |
|-------|------|
| cadre | facade |
| cadre-kernel | GeomKernel + MockKernel |
| cadre-occt | OCCT backend (LGPL, non-default CI) |
| cadre-lang | hermetic Starlark → IR + execute_ir |
| cadre-model | selectors + BuildCache |
| cadre-inspect | refs / measure / align / frame / diff |
| cadre-render | software z-buffer PNG + orbit GIF |
| cadre-bench | parity parts1-10 mock + OCCT lanes |
| cadre-mcp | stdio + streamable HTTP MCP |
| cadre-api | Axum `/v1/*` + jobs/SSE/OpenAPI |
| cadre-parts | parts.lock + LocalFsProvider + AssemblySpec |
| cadre-robot | URDF/SRDF/SDF + urdf-rs |
| cadre-fab | DXF, DFM, slicer, gcode-check, Bambu gated live |
| cadre-harness | scripted agent10 scorecard |
| cadre-cli | clap binary |

**Parked names:** cadre-truck, standalone cadre-export/viewer/skills (logic lives in cli/occt/render).

## CLI surface (high signal)
```
build | inspect refs|measure|align|frame|diff | export
snapshot | view | bench run | harness run
mcp | serve api|mcp | skills export [--all]
robot gen|validate
fab dxf|dxf-face|check|slicers|slice|gcode-check
printer status|dry-run|start [--live]
version --json
```

## Examples
- `parity/parts/01..10` — geometry fixtures (+ expect.json / expect.occt.json)
- `harness/tasks/` — agent10 scripted loops
- `examples/assembly/` · `examples/robots/` · `examples/fab/`

## Honesty defaults
- Mock ≠ OCCT; STEP needs `--features occt` + `--kernel occt`
- Snapshot cut preview keeps operand A
- Printer: allowlist + sha256 + `confirm=START` + optional `--live`
- DFM = profile-version truth, not vendor API
- Harness agent10 scripted ≠ live LLM score (H1)
