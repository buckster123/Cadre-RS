# Cadre workflow (progressive reference)

## Authoring
- One part per `.cad.star`; entry `gen_step()` returns a labeled `solid(...)`.
- Parameters via `P = params(...)`; override at build with `--set k=v` / MCP `set`.
- Prefer through-features (holes taller than plate) for robust cuts.

## Verify
1. `inspect_refs --facts` — solid/face/edge inventory + volume
2. `measure` thickness between opposite face normals
3. `snapshot` iso+front minimum; read PNG content / open viewer

## Repair
- Fillet fail → reduce radius (mock may UNSUPPORTED fillet entirely)
- Wrong volume → check hole count / params
- Bad selector after edit → re-run `inspect_refs` (tokens remapped)

## Export
- IR always: `*.ir.json`
- STEP/STL: `cadre --kernel occt` (binary with `--features occt`)
- Snap packet: `*.snap/`
