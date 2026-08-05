//! The `GeomKernel` contract — backends implement this; faces never call OCCT directly.

use std::path::Path;

use crate::error::KernelResult;
use crate::facts::{ShapeFacts, ValidityReport};
use crate::handles::{EdgeRef, ShapeId, ShapeLabel};
use crate::mesh::Mesh;
use crate::step::{StepReadOpts, StepWriteOpts};
use crate::types::{BooleanOp, Density, Placement, Point3, TessTol};

/// CAD geometry kernel.
///
/// # Invariants
///
/// - Handles are opaque and backend-local.
/// - Failures are structured ([`crate::KernelError`]); never empty success.
/// - Units: millimeters.
/// - Implementations must be `Send` so HTTP/MCP job workers can own them.
///
/// # M0 surface
///
/// box, cylinder, boolean, fillet, chamfer, facts, validity, STEP write/read, tessellate.
/// Later milestones extend via new methods or companion traits — prefer additive methods
/// with default `Unsupported` rather than breaking the trait in a minor.
pub trait GeomKernel: Send {
    /// Backend id (`occt`, `truck`, `mock`, …).
    fn backend_id(&self) -> &'static str;

    /// Human/version string for `engine_info` / build meta.
    fn backend_version(&self) -> &str;

    /// Whether this backend may carry Parity-10 claims (OCCT yes; mock/truck no).
    fn parity_eligible(&self) -> bool {
        false
    }

    // --- primitives --------------------------------------------------------

    /// Axis-aligned box centered at `placement.origin` by default convention:
    /// extends `±dx/2, ±dy/2, ±dz/2` from origin (matches PRD calibration-block flavor).
    fn box_solid(
        &mut self,
        dx: f64,
        dy: f64,
        dz: f64,
        placement: Placement,
    ) -> KernelResult<ShapeId>;

    /// Cylinder along +Z, base centered at placement origin, height `h`.
    fn cylinder(&mut self, radius: f64, height: f64, placement: Placement)
        -> KernelResult<ShapeId>;

    // --- boolean / features ------------------------------------------------

    fn boolean(&mut self, op: BooleanOp, a: ShapeId, b: ShapeId) -> KernelResult<ShapeId>;

    /// Fillet edges. Empty `edges` means backend-defined default (often all).
    fn fillet(&mut self, shape: ShapeId, edges: &[EdgeRef], radius: f64) -> KernelResult<ShapeId>;

    fn chamfer(
        &mut self,
        shape: ShapeId,
        edges: &[EdgeRef],
        distance: f64,
    ) -> KernelResult<ShapeId>;

    // --- labels ------------------------------------------------------------

    /// Attach a label; may return the same or a new handle depending on backend.
    fn set_label(&mut self, shape: ShapeId, label: ShapeLabel) -> KernelResult<ShapeId>;

    // --- queries -----------------------------------------------------------

    fn facts(&self, shape: ShapeId) -> KernelResult<ShapeFacts>;

    fn facts_with_density(&self, shape: ShapeId, density: Density) -> KernelResult<ShapeFacts> {
        let mut f = self.facts(shape)?;
        // volume mm³ → cm³ = /1000; mass_g = density * cm³
        f.mass_g = Some(density.g_per_cm3 * (f.volume_mm3 / 1000.0));
        Ok(f)
    }

    fn validity(&self, shape: ShapeId) -> KernelResult<ValidityReport>;

    /// List edge refs in stable kernel order (for fillet/chamfer selection).
    fn edges(&self, shape: ShapeId) -> KernelResult<Vec<EdgeRef>>;

    // --- I/O + mesh --------------------------------------------------------

    fn write_step(&self, shape: ShapeId, path: &Path, opts: &StepWriteOpts) -> KernelResult<()>;

    fn read_step(&mut self, path: &Path, opts: &StepReadOpts) -> KernelResult<ShapeId>;

    fn tessellate(&self, shape: ShapeId, tol: TessTol) -> KernelResult<Mesh>;

    // --- convenience -------------------------------------------------------

    /// Box convenience with origin placement.
    fn box_at(&mut self, dx: f64, dy: f64, dz: f64, at: Point3) -> KernelResult<ShapeId> {
        self.box_solid(dx, dy, dz, Placement::at(at))
    }

    fn cylinder_at(&mut self, radius: f64, height: f64, at: Point3) -> KernelResult<ShapeId> {
        self.cylinder(radius, height, Placement::at(at))
    }
}
