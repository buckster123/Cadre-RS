# H10 — truck experimental lane (non-parity)

## What

`cadre-truck` implements a **subset** of `GeomKernel`:

- box / cylinder
- boolean (analytic volume approx)
- facts / validity / edges
- coarse bbox tessellate

## Honesty (binding)

| Rule | |
|------|--|
| Default kernel | **never** truck (`mock` default; `occt` optional) |
| Parity-10 | **`parity_eligible() == false`** always |
| STEP | unsupported |
| Fillet/chamfer | unsupported |
| Upstream `truck` crate | **not** wired yet — pure-Rust CSG seed |

## CLI

```sh
cargo run -p cadre-cli -- build part.cad.star --kernel truck --json
cargo run -p cadre-cli -- version --json   # shows truck_parity_eligible: false
cargo test -p cadre-truck
```

## Promotion bar

Do **not** promote toward default until:

1. H1–H4-style agent loops still prefer OCCT for real B-rep
2. A real pure-Rust B-rep (or truck binding) replaces analytic CSG
3. Explicit parity suite + CHARTER amendment

This crate is an honesty valve + future seed — not a sneaky default flip.
