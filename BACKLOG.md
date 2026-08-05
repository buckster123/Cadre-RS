# Cadre-RS backlog — slice ledger

A row gets its ✅ when the slice is **merged, deployed, and verified live** — not when tests
pass (house doctrine #5). Notes carry the date and the evidence.

## v1 (agent CAD loop parity)

Milestones M0–M6 match `docs/CHARTER.md` / PRD §13. Slices below are the near-term ledger;
split further when a slice stops being reviewable in one PR.

- [x] **S0 — bootstrap**: Launchpad stamp, CLAUDE.md, CHARTER, design contract, PRD in-tree,
      workspace + `crates/cadre` facade, CI, dual license (2026-08-05)
- [x] **S1 — M0 kernel spike (trait + binding eval)**: `cadre-kernel` `GeomKernel` v0 +
      `MockKernel` tests; `docs/occt-binding.md`; charter D19 GO (2026-08-05). Live OCCT → S3
- [x] **S2 — M0 Starlark host PoC**: `cadre-lang` hermetic eval; `params`/`box`/`cylinder`/
      booleans/`solid` → feature IR v0; structured diagnostics JSON; overrides (2026-08-05)
- [x] **S3 — M0 e2e part 1**: `cadre-occt` GeomKernel; IR execute; fillet/chamfer in lang;
      calibration block → STEP + facts (OCCT local; CI excludes package) (2026-08-05)
- [x] **S4 — M1 build cache + selectors**: `cadre-model` (selectors + content-hash cache);
      `cadre-inspect` (refs/measure); stable `#o…` tokens (2026-08-05)
- [x] **S5 — M1 CLI face**: `cadre-cli` binary `cadre` — `build` / `inspect refs|measure` /
      `export step|stl|glb` with `--json`; mock default, optional `--features occt` (2026-08-05)
- [x] **S6 — M1 parity parts 1–4**: `parity/parts/01–04` + `cadre-bench` runner +
      `cadre bench run --suite parts1-4`; mock CI green (2026-08-05)
- [x] **S7 — M2 snapshot + viewer alpha**: `cadre-render` software PNG/GIF packets;
      `cadre snapshot` / `cadre view` deep links (2026-08-05)
- [x] **S8 — M2 MCP stdio + skill-pack alpha**: `cadre-mcp` 6 tools; `cadre mcp`;
      `cadre skills export`; bundled `skills/cadre` doctrine (2026-08-05)
- [x] **S9 — M3 assemblies + parts.lock + HTTP API**: `cadre-parts` lock/provider/assembly;
      `cadre-api` Axum `/v1` + jobs/SSE/OpenAPI; plate+bolt example; `cadre serve api` (2026-08-05)
- [ ] **S10 — M4 robots**: URDF+inertials validate; SRDF/SDF; ROS 2 parser load
- [ ] **S11 — M5 fab path**: DXF, DFM profile, slicer orch, gcode-check, Bambu dry-run + gated start
- [ ] **S12 — M6 1.0 hardening**: Windows, fuzz, skills export both agents, licensing review, metrics table

## Post-v1 parking

- Implicit SDF CAD (FR-9xx) — experimental, STEP-first preferred
- WASM component authoring against IR
- truck kernel promotion toward default (only with parity evidence)
- build123d → skeleton `.cad.star` migration assistant (clean-room from public docs only)
- Klipper/Moonraker/OctoPrint adapters; additional DFM vendor profiles
- STEP PMI/GD&T; drawing sheets
- Public multi-tenant hardening of the HTTP API
