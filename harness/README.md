# Agent harness (`agent10`)

Deterministic **scripted** agent-loop scorecard — not a live LLM judge.

Each task is natural-language `prompt` + ordered `loops` of steps
(`write` → `build` → `inspect_refs` → `snapshot` → `assert`). The runner
counts **loops-to-success** (≤ `max_loops`, default 3).

## Run

```sh
cargo test -p cadre-harness
cargo run -p cadre-cli -- harness run --suite agent10 --json
```

Target bar (PRD M3): **≥ 6/10**.

## Honesty

- Uses mock kernel + IR topology (same CI path as parity mock).
- Scripted steps stand in for an agent; plug a real model later via external driver.
- Snapshot packets are real software renders (small size for speed).
