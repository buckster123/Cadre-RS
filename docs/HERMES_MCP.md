# Hermes MCP wiring (Cadre)

## Install binary

```sh
cargo build -p cadre-cli --release
cp -f target/release/cadre ~/.local/bin/cadre
```

## Config (`~/.hermes/config.yaml`)

```yaml
mcp_servers:
  cadre:
    command: /home/andre/.local/bin/cadre
    args:
      - mcp
    env:
      CADRE_PROJECT: /home/andre/Projects/Cadre-RS
      CADRE_MCP_WRITE_SOURCE: "1"   # allow write_source on stdio for agent loops
      RUST_LOG: warn
    timeout: 120
    connect_timeout: 30
    enabled: true
```

## Framing note

Hermes uses the official Python `mcp` SDK, which speaks **NDJSON** (one JSON-RPC object per line).
Cadre auto-detects NDJSON vs Content-Length. Override with `CADRE_MCP_FRAMING=ndjson|content-length`.

## Verify

```sh
hermes mcp test cadre
# → Connected, 9 tools: build write_source read_source inspect_refs measure snapshot
#   inspect_dims assembly_validate sdf_sample
```

## Live session

Config changes need **`/reload-mcp`** (or a new Hermes session). Disk-green `hermes mcp test` is not the same as the already-spawned child in a long-lived CLI session.

## Tools (prefix)

After reload, tools appear as `mcp_cadre_*` / deferred catalog names depending on Hermes version:

- `build` · `write_source` · `read_source` · `inspect_refs` · `measure` · `snapshot`
- **H3-3:** `inspect_dims` · `assembly_validate` · `sdf_sample` (secondary)
- resources: `resources/list` · `resources/read` (`cadre://doc/**`)

## Drive example

```text
Use cadre MCP build on parity/parts/01_*/…cad.star and report volume_mm3.
```
