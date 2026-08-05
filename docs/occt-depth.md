# OCCT depth notes (post-v1)

## Shipped (this slice)
- `OcctKernel::topology_snapshot(shape)` — live B-rep faces (COM + normals + planar area
  approx) and edges (polyline length + midpoint)
- `cadre inspect … --kernel occt` uses live topology when binary built `--features occt`
- Tests: box topology + thickness measure; union topology + volume

## Known host issue
`BRepAlgoAPI_Cut` (boolean cut) aborts with C++ `StdFail_NotDone` on the current Ubuntu OCCT
+ `opencascade` 0.2 stack. **Union works.** Cut/fillet e2e tests are `#[ignore]` until the
binding/runtime is fixed.

Repro:
```sh
CMAKE_POLICY_VERSION_MINIMUM=3.5 cargo test -p cadre-occt --test cut_smoke -- --ignored
```

## Local verify (works today)
```sh
CMAKE_POLICY_VERSION_MINIMUM=3.5 cargo test -p cadre-occt
CMAKE_POLICY_VERSION_MINIMUM=3.5 cargo run -p cadre-cli --features occt -- \
  --kernel occt inspect refs <box-only.cad.star> --facts --json
```

## Next
1. Fix/cut-wrap boolean cut (catch NotDone → KernelError, or upgrade opencascade-rs)
2. Re-enable calibration + parity-01 OCCT goldens
3. Wire tessellation tolerance for tighter volume goldens
