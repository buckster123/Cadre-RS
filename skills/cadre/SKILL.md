---
name: cadre
description: >
  Cadre CAD runtime for agents — hermetic Starlark → IR → inspect → snapshot.
  Use when the user wants parametric CAD parts, STEP-oriented design, or geometry
  verification. Prefer cadre CLI/MCP tools over freeform guessing.
---

# Cadre skill (alpha)

## Defaults
- Units: **millimeters**, XY base, **+Z up**
- Source files: `*.cad.star` with `def gen_step():`
- Always **numeric inspect before visual**; then **snapshot** after visible changes
- Never directory-wide builds; pass explicit file paths
- Do not git-diff binary STEP/PNG; use `inspect` / `snapshot`

## Loop
1. Clarify one question max if critical dims missing
2. Write `part.cad.star` (`write_source` / editor)
3. `build` → check facts (volume, bbox)
4. `inspect_refs` / `measure` to verify holes/thickness
5. `snapshot` and **review images** (mandatory if geometry changed)
6. Hand off paths (`.ir.json`, `.snap/`, later STEP via OCCT)

## Starlark flavor
```python
P = params(width=100.0, depth=60.0, height=20.0, hole_d=8.0)

def gen_step():
    blk = box(P.width, P.depth, P.height, at=CENTER)
    hole = cylinder(P.hole_d / 2.0, P.height + 2.0, at=(0.0, 0.0, -1.0))
    return solid(cut(blk, hole), label="part")
```

Ops: `params`, `box`, `cylinder`, `cut`/`union`/`intersect`/`union_all`, `fillet`, `chamfer`, `solid`, `CENTER`.

## CLI
```sh
cargo run -p cadre-cli -- build path.cad.star --json
cargo run -p cadre-cli -- inspect refs path.cad.star --facts --json
cargo run -p cadre-cli -- snapshot path.cad.star --json
cargo run -p cadre-cli -- mcp          # stdio MCP
```

## MCP tools
`build`, `write_source`, `read_source`, `inspect_refs`, `measure`, `snapshot`

## Honesty
- Default kernel is **mock** (IR + analytic facts). Real STEP needs OCCT feature.
- Snapshot preview mesh is approximate (cuts keep solid A). Trust measures for truth.
- Selectors are `#o1.1.f3` (1-based, stable sort).

## Sanctioned snapshot skip
Only if **no visible geometry change** or **no valid artifact** — report the reason.
