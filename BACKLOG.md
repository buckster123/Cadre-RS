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
- [ ] **S3 — M0 e2e part 1**: boolean + fillet + STEP write; calibration block from `.cad.star`;
      facts (bbox/volume) golden within tol
- [ ] **S4 — M1 build cache + selectors**: content-hash cache; `#o…` tokens stable; `inspect refs|measure`
- [ ] **S5 — M1 CLI face**: `cadre-cli` `build` / `inspect` / `export step|stl|glb` with `--json`
- [ ] **S6 — M1 parity parts 1–4**: deterministic suite green on Linux CI
- [ ] **S7 — M2 snapshot + viewer alpha**: PNG packet + orbit GIF; `cadre view` deep links
- [ ] **S8 — M2 MCP stdio + skill-pack alpha**: tools budget ≤ 4k tokens; agent part 1 with snapshot review
- [ ] **S9 — M3 assemblies + parts.lock + HTTP API**: S3 scenario; jobs/SSE/OpenAPI; harness ≥ 6/10
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
