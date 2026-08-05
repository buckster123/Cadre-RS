# cadre-render

Software (CPU) mesh rasterizer for Cadre snapshot packets.

- Multi-view PNG (iso/front/top/right/…)
- Orbit turntable GIF
- IR → preview mesh (box/cylinder analytic; cut keeps A)

No GPU required — default CI path.

```rust
use cadre_render::{mesh_from_ir, write_snapshot_packet, SnapshotOptions};
```

See `docs/design.md` § Snapshots & viewer.
