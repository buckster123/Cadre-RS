<div align="center">

<img src="assets/banner.jpg" alt="Cadre-RS" width="100%">

<h1>Cadre-RS</h1>

<p><strong>CAD runtime for AI agents — hermetic Starlark in, verified STEP out.</strong><br>
Rust-native toolkit: build, inspect, snapshot, export, source parts, describe robots, and
hand off to fabrication through CLI, MCP, and local HTTP. Clean-room peer to text-to-cad skills.</p>

<p>
<img alt="license" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue">
<img alt="rust" src="https://img.shields.io/badge/rust-2021-orange?logo=rust&logoColor=white">
<img alt="ci" src="https://img.shields.io/github/actions/workflow/status/buckster123/Cadre-RS/ci.yml?label=ci">
<img alt="status" src="https://img.shields.io/badge/status-v0.1%20%C2%B7%20bootstrap-brightgreen">
</p>

</div>

---

> [!NOTE]
> Model code has zero ambient authority (no clock, net, or filesystem). Hardware and vendor
> effects are dry-run-first and consent-gated on every surface — nothing prints by default.

## What it is

Cadre is a single workspace that turns agent-written parametric CAD (Starlark) into B-rep
geometry via an OCCT-backed kernel, then gives the agent numeric facts, stable selectors,
mandatory visual review packets, and paths to parts catalogs, robot descriptions, and fab
tools. Prompt-ware (exported skill packs) is half the product: doctrine for the loop, not
just binaries.

## Install

```sh
git clone https://github.com/buckster123/Cadre-RS
cd Cadre-RS
cargo build --release --workspace
# later: cadre engine install   # OCCT backend component (checksummed)
```

## Use

```sh
# after M1 surfaces land — illustrative:
cadre build cad/block.cad.star --json
cadre inspect refs cad/block.step --facts --json
cadre snapshot cad/block.step --views iso,front --gif
cadre view cad/block.step
```

Today (bootstrap): library crate + docs/CI only — see [`BACKLOG.md`](BACKLOG.md).

## How it works

```
agent/human ──CLI/MCP/HTTP──▶ cadre-lang (Starlark) → IR → GeomKernel (OCCT)
                              inspect · snapshot · export · parts · robot · fab
```

Contract: [`docs/design.md`](docs/design.md). Binding decisions: [`docs/CHARTER.md`](docs/CHARTER.md).
Full PRD: [`docs/cadre-prd.md`](docs/cadre-prd.md).

## Docs

| File | What's in it |
|------|--------------|
| [`docs/design.md`](docs/design.md) | The contract — wire format, API, invariants |
| [`docs/CHARTER.md`](docs/CHARTER.md) | Binding decisions, phases, scope fence |
| [`docs/cadre-prd.md`](docs/cadre-prd.md) | Product requirements, parity matrix, NFRs |
| [`BACKLOG.md`](BACKLOG.md) | Slice ledger — what's shipped, what's next |

## License

MIT OR Apache-2.0 — see [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
The optional OCCT engine component is LGPL-2.1 with the OCCT exception and is distributed
separately (see charter D4/D6).

<sub>Banner: pending Imaginarium-RS job (not generated at bootstrap — spend is gated).</sub>
