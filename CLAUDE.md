# Cadre-RS — Agent & Developer Guide

> Rust-native CAD runtime for AI agents: Starlark → B-rep → inspect → snapshot → fab.
> Single workspace; CLI + MCP + local HTTP as co-equal faces; skill pack teaches the loop.
> Standalone-first — agents and humans consume it directly; ApexOS assimilation is not assumed.

Bootstrapped 2026-08-05. House conventions come from `~/Projects/Launchpad-RS/`
— load a doc from there when you need the detail behind a rule below.

**Read `docs/CHARTER.md` before any non-trivial change — its decisions log (D1–Dn) is
binding.** Amend it with a dated entry when a decision changes, never silently. Where the
charter and this file disagree, the charter wins.

Siblings: none required at bootstrap (Imaginarium-RS for banners; garden MCP hosts later).
Reference/read-only: public docs of `earthtojake/text-to-cad` for *behavior* only — **do not
clone its source into this tree or translate it** (charter D1). Product PRD: `docs/cadre-prd.md`.

---

## What this is

Cadre gives coding agents a hermetic hardware-design loop without a Python CAD stack: author
parametric geometry in Starlark, build B-rep via an OCCT-backed kernel, verify with numeric
selectors/facts, review with snapshots/viewer, then export, source parts, emit robot
descriptions, or hand off to fabrication under consent gates. The skill pack is half the
product — tools without doctrine are not parity.

```
crates/
  cadre/            # facade (S0 placeholder → re-exports)
  cadre-kernel/     # GeomKernel trait
  cadre-occt/       # default kernel backend (separate engine)
  cadre-truck/      # experimental pure-Rust backend
  cadre-lang/       # Starlark + CAD stdlib → IR
  cadre-model/      # IR, selectors, cache, artifacts
  cadre-inspect/    # facts / measure / align / diff
  cadre-render/     # wgpu snapshots / orbit GIF
  cadre-export/     # STEP STL 3MF GLB DXF
  cadre-viewer/     # embedded local web viewer
  cadre-parts/      # catalog + parts.lock
  cadre-robot/      # URDF SRDF SDF
  cadre-fab/        # DFM, slicers, printers
  cadre-mcp/        # agent face
  cadre-api/        # local HTTP face
  cadre-cli/        # `cadre` binary
  cadre-skills/     # skill-pack export
docs/design.md      # THE contract — pinned before code
docs/cadre-prd.md   # full product requirements
BACKLOG.md          # slice ledger S0–Sn + post-v1 parking
```

---

## Locked decisions

The load-bearing summary; **`docs/CHARTER.md` D1–Dn is the binding long form.** House defaults
are pre-filled — delete what doesn't apply, add what's yours.
**Locked means locked — do not re-litigate these mid-session; amend deliberately, with a date.**

- **Language**: Rust — one Cargo workspace, every binary in it
- **Kernel exception**: OCCT via FFI is the default backend (D4); pure-Rust preferred elsewhere
- **Authoring**: Starlark `.cad.star` (etc.), hermetic, STEP-first (D2, D3)
- **License**: MIT OR Apache-2.0 dual for core; OCCT separate LGPL engine component (D6)
- **Faces**: CLI + MCP + local Axum HTTP; single schema source (D5, D13)
- **MCP**: stdio default; SDK vs hand-rolled confirmed at M2 (D17 / OQ-7). stdout sacred either way
- **HTTP**: `reqwest` (rustls) out, `axum` in; `clap` for CLI; `serde` everywhere
- **CI from commit 0**: fmt `--check` + clippy `-D warnings` + test + build
- **rustfmt-clean baseline from commit 0** — so `cargo fmt --all` is always safe here
- **Not nano-first**: CAD/OCCT targets developer laptops and workstations (NFR-1); graceful
  degrade when engine/slicer/GPU absent — never fake success
- **Safety**: printer/vendor effects dry-run + allow-list + explicit confirm (D10)
- **No telemetry** (D14)
- **Cerebro agent**: `CADRE` (D15)
- **Clean-room** vs text-to-cad (D1)

---

## The playbook (the house method — read once, then live it)

Full rationale: `~/Projects/Launchpad-RS/docs/house-doctrine.md`. The nine, condensed:

1. **Contract first.** Pin the wire/API/format in `docs/design.md` before code. Code follows
   docs; a PR updates both. **Docs travel with code.**
2. **Slices, not marathons.** One branch = one reviewable slice off freshly-fetched
   `origin/main`. Never open a PR whose base is another branch.
3. **Honesty invariants.** Never a fake success. Degrades are *stated* ("engine not installed"
   beats a timeout). Failures carry the real reason. Check the response body, not just the
   HTTP status. Never silently clamp what you can honestly reject.
4. **Pure-fn test discipline.** Pure functions (parsers, IR builders, selectors, validators,
   formatting) are the unit-test surface; handlers are thin I/O glue. Kernel/golden fixtures
   from real builds. Effectful e2e tests skip *loudly*.
5. **Field truth beats green CI.** A slice is done when it runs on a live node — real STEP,
   real snapshot, real agent loop — not when tests pass. The ledger row gets its ✅ only then.
6. **Secrets hygiene.** Never print a key or token (lengths and heads only). Never write one
   into a repo, a transcript, a doc, or a non-0600 file. **No credentials in CLAUDE.md** —
   these files get committed, and repos go public.
7. **Cerebro is the thread.** `session_recall` at start, `session_save` at milestones and end.
8. **Spend is gated.** Paid operations (API credits, GPU rental, image/music generation) never
   auto-fire from a default flow. Live-fire runs are explicit, counted, and André's call.
9. **Cost the failure, not the happy path.** A paid job that outlives its poll window is
   *pending*, not failed — leave it recoverable (resumable ids), never orphan spend.

---

## Git discipline

- **Never commit to `main`.** Feature branch off freshly-fetched `origin/main`: `feat/…`,
  `fix/…`, `chore/…`, `docs/…`. One branch = one slice.
- **Ship via PR** (`gh pr create`). **Do NOT merge it yourself** — André reviews and merges,
  or explicitly tells you to. (Pre-publication bootstrap commits are the sanctioned exception.)
- **Commit format:** imperative, lowercase. End with the `Co-Authored-By` trailer.
- **Never amend a pushed commit. Never force-push.**
- **Push after every commit.** Local git is the floor of resilience: if Cerebro is
  unavailable, the repo + its docs must be enough to reconstruct full project context.

---

## Cerebro session protocol (mandatory)

All Cerebro MCP calls use agent CADRE (`agent_id="CADRE"`) — memories stay isolated per project.
Full tool menu + grading discipline: `~/Projects/Launchpad-RS/docs/cerebro-protocol.md`.

**Session START** — before touching any code:
```
session_recall(query="Cadre-RS build status step progress", agent_id="CADRE")
```

**Session END** (and at milestones on long sessions):
```
session_save(session_summary="what was built, what broke, what was learned",
             key_discoveries=[...], unfinished_business=[...],
             agent_id="CADRE", priority="HIGH")
```
Then as needed: `store_procedure` · `record_procedure_outcome` (**grade every procedure you
exercised** — ungraded ones are invisible to the dream engine) · `store_intention` (parked
ideas, salience 0.8–0.95) · `episode_*` (multi-step sequences).

**The vaults:** CLAUDE.md = lean core + pointers · `docs/gotchas.md` = invariants ·
`docs/*.md` = per-topic detail · Cerebro = session memory, survives compaction · git = code truth.

---

## Dev commands

```bash
cargo test --workspace
cargo fmt --all && cargo clippy --workspace -- -D warnings   # clippy-zero policy
cargo build --release --workspace

# once faces land:
cargo run -p cadre-cli -- build cad/block.cad.star --json
cargo run -p cadre-cli -- schema mcp
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run -p cadre-mcp
```

Local-first tool: no Pi deploy required for core loop. Optional later: package `cadre` +
engine artifact per `~/Projects/Launchpad-RS/docs/deploy.md` patterns if a lab node hosts the
viewer/API.

---

## Gotchas

Project invariants live in **`docs/gotchas.md`** — grep it for your subsystem **before**
modifying it. Most entries were written after something broke on a live node; each ends with
an explicit "don't do X". **A new gotcha goes THERE, not here.** Cross-project version drift
(axum/comrak/gix/tantivy/slint/wgpu/…) is in `~/Projects/Launchpad-RS/docs/sharp-edges.md`.

Two that bite every project in this garden:

- **MCP stdout is sacred.** All `tracing`/log output goes to **stderr**. A stray `println!`
  corrupts the JSON-RPC stream.
- **Read the pinned crate's docs for the exact version** — not memory of an older API.
  Version drift gets recorded in a dated changelog line, never fought silently.

Cadre-specific seeds (also in `docs/gotchas.md`):

- **Clean-room:** never vendor or translate reference-project source.
- **OCCT stays behind `GeomKernel` + separate engine install.**
- **Do not git-diff binary STEP/STL** — use `inspect diff`.

---

## Docs

Load only the relevant doc when entering a subsystem — do not load all of them.

| File | Load when working on |
|------|----------------------|
| `docs/CHARTER.md` | **Binding decisions D1–Dn, phases, scope fence — read before non-trivial work** |
| `docs/design.md` | **The contract** — wire format, API, invariants |
| `docs/cadre-prd.md` | Full PRD, parity matrix, NFRs, milestone detail |
| `docs/gotchas.md` | **Any subsystem change — grep it first** |
| `BACKLOG.md` | Outstanding work — slice ledger + parked items |

---

## Meta — when to update this file

- A locked decision changes → **`docs/CHARTER.md` first** (dated amendment), then the summary here
- A gotcha is discovered → **`docs/gotchas.md`**, not here
- A slice completes → tick it in `BACKLOG.md`
- A doc file is created → add a row to `## Docs`
- **Keep this file under ~250 lines / ~20 KB.** Claude Code warns on oversized CLAUDE.md and
  it loads into every session's context. Fat goes to `docs/`; this file points.
- Before publishing the repo, inline anything it truly depends on from `Launchpad-RS/` so the
  repo stands alone for outside readers.

### What never goes in CLAUDE.md or docs/*.md

- Task progress, session logs, completed-work summaries → Cerebro (`session_save`)
- Git SHAs, version pins → stale in days, belong in git history
- Commentary on what you just did → belongs in commit messages
- **Credentials of any kind** → env files (0600, root-owned), never a tracked file
