# Cadre-RS backlog — slice ledger

A row gets its ✅ when the slice is **merged, deployed, and verified live** — not when tests
pass (house doctrine #5). Notes carry the date and the evidence.

## v1 (agent CAD loop parity) — COMPLETE

Milestones M0–M6 / slices S0–S12 shipped on `main` (2026-08-05). Scorecard: `docs/METRICS.md`.
As-built map: `docs/STATUS.md`.

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
- [x] **S10 — M4 robots**: `cadre-robot` URDF/SRDF/SDF gen+validate+inertials; urdf-rs parse;
      `cadre robot gen|validate`; simple_arm example (2026-08-05)
- [x] **S11 — M5 fab path**: `cadre-fab` DXF + DFM (SendCutSend-style) + slicer discover +
      gcode-check + Bambu dry-run/gated start (no live print); `cadre fab` / `cadre printer`
      (2026-08-05)
- [x] **S12 — M6 1.0 hardening**: metrics table, licensing review, Windows CI job, dual-agent
      skills export (`--all`), property/fuzz-style parser tests, release checklist (2026-08-05)

## Post-v1 candidates (unordered; pick deliberately)

Priority suggestions when resuming:

1. **OCCT depth** — live topology + AdHoc booleans + mesh topology ✅ (PR #14)
2. **OCCT bench lane** — `parts1-4-occt` + expect.occt.json ✅ (PR #15)
3. **Face→DXF** — project planar face selector to DXF ✅ (PR #16)
4. **Live Bambu** — FTPS/MQTT behind gates + `--live` ✅ (PR #17)
5. **Streamable-HTTP MCP** — POST /mcp + SSE ✅ (PR #18)
6. **Agent harness score** — scripted agent10 ≥6/10 ✅ (PR #19)
8. **Diff/align/frame** CLI polish beyond assembly align_check ✅ (PR #20)
9. **Parity 5–10** — full parts1-10 mock suite + translate/rotate ✅ (PR #21)
10. **OCCT translate/rotate + expect.occt 5–10** ✅ (PR #22)

## Horizon-1 board (ordered)

**Source of truth:** [`docs/HORIZON.md`](docs/HORIZON.md) — cook order, exit criteria, anti-goals.

| # | Slice | Status |
|---|-------|--------|
| H1 | Live agent harness driver (`--cmd` / MCP score) | next |
| H2 | Stdlib depth (mirror, patterns, cone/sphere, …) | pending |
| H3 | OCCT transform quality (drop STEP rotate round-trip) | pending |
| H4 | Fillet/chamfer in OCCT parity + diagnostics | pending |
| H5 | Viewer: G-code scrub + URDF jog alpha | pending |
| H6 | Slicer execute (gated) + 2nd DFM profile | pending |
| H7 | MCP resources + write_source policy | pending |
| H8 | build123d → skeleton migrator (clean-room) | pending |
| H9 | Klipper/Moonraker gated adapter | pending |
| H10 | truck experimental non-parity lane | pending |

Default when resuming with no pref: **H1**.

## Post-v1 parking (Horizon-2+)

- Implicit SDF CAD (FR-9xx) — experimental, STEP-first preferred; not Horizon-1
- WASM component authoring against IR
- truck **promotion toward default** (only after H10 seed + parity evidence)
- STEP PMI/GD&T; drawing sheets
- Public multi-tenant hardening of the HTTP API
- Klipper/Moonraker/OctoPrint **beyond H9**; additional DFM vendor profiles beyond H6
