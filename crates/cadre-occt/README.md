# cadre-occt

Open CASCADE Technology backend implementing `cadre_kernel::GeomKernel`.

**License:** this crate is **LGPL-2.1** (links OCCT via `opencascade` 0.2). Core
Cadre crates remain MIT OR Apache-2.0.

## Build

Ubuntu 25.10+ / CMake 4.x needs:

```sh
export CMAKE_POLICY_VERSION_MINIMUM=3.5
cargo test -p cadre-occt
```

First build compiles OCCT from source via `occt-sys` (several minutes).

Default workspace CI **excludes** this package (`default-members` + CI workflow).

## Usage

```rust
use cadre_kernel::GeomKernel;
use cadre_lang::{evaluate, execute_ir, EvalOptions};
use cadre_occt::OcctKernel;

let r = evaluate(src, &EvalOptions::new("part.cad.star"));
let mut k = OcctKernel::new();
let sid = execute_ir(&mut k, r.ir.as_ref().unwrap()).unwrap();
k.write_step(sid, "part.step", &Default::default()).unwrap();
```
