# H9 — Klipper / Moonraker adapter

Second printer family behind the **same consent gates** as Bambu.

## Gates (identical story)

| Gate | Rule |
|------|------|
| allowlist | `--allowlist klipper:ender` |
| sha256 | must match `printer dry-run` |
| confirm | exactly `START` |
| gcode-check | static validation |
| `--live` | second consent — no sockets without it |

## CLI

```sh
# status / dry-run (no network)
cargo run -p cadre-cli -- printer status \
  --backend klipper --id klipper:ender --host 192.168.1.60 --json

cargo run -p cadre-cli -- printer dry-run examples/fab/sample.gcode \
  --backend klipper --id klipper:ender --host 192.168.1.60 --json

# gates only (still no network)
cargo run -p cadre-cli -- printer start examples/fab/sample.gcode \
  --backend klipper --id klipper:ender --host 192.168.1.60 \
  --sha256 <from-dry-run> --confirm START --allowlist klipper:ender --json

# LIVE Moonraker (YOU opt in)
export CADRE_MOONRAKER_API_KEY=optional
cargo run -p cadre-cli -- printer start examples/fab/sample.gcode \
  --backend klipper --id klipper:ender --host 192.168.1.60 \
  --sha256 <from-dry-run> --confirm START --allowlist klipper:ender \
  --live --json
```

Id prefix `klipper:` / `moonraker:` auto-selects backend even if `--backend bambu` left default.

## Live path

1. `POST {url}/server/files/upload` (multipart, root=gcodes) via curl  
2. `POST {url}/printer/print/start?filename=…`

Default URL: `http://HOST:7125` or `CADRE_MOONRAKER_URL` / `--moonraker-url`.

## Env

| Var | Role |
|-----|------|
| `CADRE_MOONRAKER_URL` | base URL |
| `CADRE_MOONRAKER_API_KEY` | optional `X-Api-Key` |
| `CADRE_CURL` | curl binary |

## Honesty

- Not a full Moonraker client (no webcam, no history, no pause/resume tools)
- API surface can drift with Moonraker versions — label accordingly
- Bambu path unchanged
