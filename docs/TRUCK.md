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

Do **not** promote toward default until the **go criteria** in
[`docs/TRUCK_PARITY_BID.md`](TRUCK_PARITY_BID.md) (H2-10) are all met and CHARTER is
amended. Summary:

1. Real BREP (or truck binding) replaces analytic CSG  
2. STEP + tessellate + inspect path honest  
3. Explicit parity suite green; `parity_eligible` still gated  
4. Agent loops still prefer OCCT until default decision  

**H2-10 decision: NO-GO** for default/parity — bid prepared only.

This crate is an honesty valve + future seed — not a sneaky default flip.
