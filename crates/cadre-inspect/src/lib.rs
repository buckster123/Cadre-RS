//! Inspect engines: topology snapshot, refs inventory, measurements.

#![deny(unsafe_code)]

mod measure;
mod refs;
mod topology;

pub use measure::{measure, MeasureError, MeasureKind, MeasureRequest, MeasureResult};
pub use refs::{inspect_refs, RefEntry, RefsReport};
pub use topology::{
    box_topology, cylinder_topology, EdgeRec, FaceRec, SolidRec, TopologySnapshot, VertexRec,
};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
