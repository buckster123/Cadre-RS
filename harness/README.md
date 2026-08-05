# Agent harness (`agent10`)

Two modes:

| Mode | How | What the agent sees |
|------|-----|---------------------|
| **scripted** (default) | Built-in step loops | N/A (no external process) |
| **live** | `--cmd '…'` | **Prompt only** + empty workdir |

## Scripted (CI default)

```sh
cargo test -p cadre-harness
cargo run -p cadre-cli -- harness run --suite agent10 --json
```

Target bar (PRD M3): **≥ 6/10**.

## Live driver protocol

```sh
# Plumbing check (oracle — not an LLM)
cargo run -p cadre-cli -- harness run --suite agent10 \
  --cmd 'python3 harness/drivers/oracle_agent.py' --json

# Real agent: your process must write $CADRE_HARNESS_PART from the prompt alone.
# Do not read CADRE_HARNESS_TASK_FILE for solutions (oracle only).
export CADRE_BIN="$(pwd)/target/debug/cadre"
cargo run -p cadre-cli -- harness run --suite agent10 \
  --cmd 'my-agent-runner' --timeout 600 --json
```

### Env vars (each loop)

| Variable | Meaning |
|----------|---------|
| `CADRE_HARNESS_TASK_ID` | e.g. `01_block` |
| `CADRE_HARNESS_PROMPT` | Natural language only |
| `CADRE_HARNESS_WORKDIR` | Temp workspace (cwd of `--cmd`) |
| `CADRE_HARNESS_PART` | Absolute path to write (`…/part.cad.star`) |
| `CADRE_HARNESS_LOOP` | 1-based attempt |
| `CADRE_HARNESS_MAX_LOOPS` | Cap (default 3) |
| `CADRE_HARNESS_TASK_FILE` | Task JSON (oracle/debug; **not** for fair LLM runs) |

### Contract

1. Exit **0** when you produced a candidate part.  
2. Leave valid Starlark at `CADRE_HARNESS_PART` with `def gen_step(): …`.  
3. Harness **builds + asserts** (volume/label/faces/snapshot). Fail → next loop.  
4. Scorecard `mode: "live"`; same ≥6/10 target.

### Example agent sketch

```sh
# pseudocode
# read $CADRE_HARNESS_PROMPT
# write Starlark to $CADRE_HARNESS_PART
# optional: $CADRE_BIN build "$CADRE_HARNESS_PART" --json for self-check
```

## Honesty

- Scripted 10/10 ≠ live LLM score.  
- Oracle driver cheats via task file — only for plumbing.  
- Live verify uses **mock** kernel (same as scripted CI path).  
- Snapshot packets are real software renders when not `--no-snapshot`.

# In-process oracle (CI)
cargo run -p cadre-cli -- harness run --suite agent10 --cmd '@oracle' --json
