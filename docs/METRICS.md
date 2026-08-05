# Cadre-RS v1 exit metrics (PRD §12 / M6)

Living scorecard. A row is **green** only with evidence (command + date), not intent.

| # | Metric | Target | Status | Evidence |
|---|--------|--------|--------|----------|
| 1 | Hermetic Starlark → IR | refuse `load()`, stable IR | **green** | `cargo test -p cadre-lang` |
| 2 | Mock kernel CI | default workspace tests OCCT-free | **green** | `.github/workflows/ci.yml` |
| 3 | Selectors + inspect | stable `#o…` + measure | **green** | S4/S5 CLI tests |
| 4 | Parity parts 1–4 | deterministic suite | **green** | `cargo test -p cadre-bench` |
| 5 | Snapshot packet | multi-view PNG + orbit GIF | **green** | `cli_snapshot` tests |
| 6 | MCP stdio | Content-Length tools | **green** | `cargo test -p cadre-mcp` |
| 7 | HTTP API | `/v1/*` + OpenAPI + jobs | **green** | `http_api` 5 tests |
| 8 | parts.lock fail-closed | checksum verify | **green** | `cadre-parts` tests |
| 9 | URDF validate | urdf-rs parse | **green** | `cadre-robot` + `robot validate` |
| 10 | DFM preflight | profile findings cite rules | **green** | `fab check` plate.flat.json |
| 11 | G-code check | bbox/temp/flavor | **green** | `fab gcode-check` sample.gcode |
| 12 | Printer start gates | allowlist+hash+confirm; no silent start | **green** | `printer` unit tests + dry-run |
| 13 | Skills export | claude-code **and** codex packs | **green** | `skills export --all` (S12) |
| 14 | Licensing review | dual MIT/Apache core; OCCT LGPL isolated | **green** | `docs/LICENSING.md` (S12) |
| 15 | Windows CI | `cargo test` on windows-latest | **green** | CI `windows` job (S12) |
| 16 | Fuzz / property | parsers don't panic on junk | **green** | property tests (S12) |
| 17 | Live OCCT STEP e2e | optional local | **green** | cal-block STEP + cut via AdHocShape |
| 18 | Live Bambu MQTT start | gated + network | **red / deferred** | dry-run only by design |
| 19 | Face→DXF projection | from B-rep face ref | **amber** | plate DXF helper only |
| 20 | Agent harness ≥6/10 | external eval | **amber** | manual / future |
| 21 | OCCT live topology inspect | faces/normals from B-rep | **green** | `topology_snapshot` + box thickness |

**v1 ship bar (this table):** rows 1–16 green; 17–20 may remain amber/red with honesty notes.

Last updated: 2026-08-05 (S12 merged on `main`; docs sync same day).
As-built companion: [`STATUS.md`](STATUS.md).
