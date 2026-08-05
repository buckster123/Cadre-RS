//! Open CASCADE Technology backend for [`cadre_kernel::GeomKernel`].
//!
//! Links LGPL OCCT via the `opencascade` crate. Not part of default CI
//! (`cargo test --workspace --exclude cadre-occt`). See `docs/occt-binding.md`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use cadre_kernel::{
    BBox, BooleanOp, EdgeRef, GeomKernel, KernelError, KernelResult, Mesh, Placement, Point3,
    ShapeFacts, ShapeId, ShapeLabel, StepReadOpts, StepWriteOpts, TessTol, ValidityReport,
};
use glam::dvec3;
use opencascade::adhoc::AdHocShape;
use opencascade::primitives::{IntoShape, Shape};

static CLONE_SEQ: AtomicU64 = AtomicU64::new(0);

/// OCCT-backed geometry kernel.
///
/// # Thread safety
///
/// `opencascade::Shape` is `!Send` because cxx unique pointers are not auto-Send.
/// Each `OcctKernel` is still safe to move between threads if **one thread at a time**
/// owns and mutates it (no shared interior mutability). HTTP/MCP job workers should
/// hold a kernel per job, not share one across tasks.
pub struct OcctKernel {
    next_id: u64,
    shapes: HashMap<u64, Shape>,
    labels: HashMap<u64, String>,
}

// SAFETY: Shapes are uniquely owned behind the HashMap; we never share &mut across threads.
unsafe impl Send for OcctKernel {}

impl Default for OcctKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl OcctKernel {
    /// Create an empty kernel.
    pub fn new() -> Self {
        Self {
            next_id: 0,
            shapes: HashMap::new(),
            labels: HashMap::new(),
        }
    }

    fn alloc(&mut self, shape: Shape) -> ShapeId {
        self.next_id += 1;
        let id = self.next_id;
        self.shapes.insert(id, shape);
        ShapeId(id)
    }

    fn get(&self, id: ShapeId) -> KernelResult<&Shape> {
        self.shapes
            .get(&id.0)
            .ok_or_else(|| KernelError::unknown_shape(id))
    }

    fn map_occt_err(op: &str, err: opencascade::Error) -> KernelError {
        KernelError::diagnostic(
            "CADRE-E-KERNEL",
            format!("{op}: {err}"),
            Some("check geometry validity / feature parameters".into()),
        )
    }

    /// Clone a shape via STEP round-trip (public API has no Shape::clone).
    fn clone_shape(shape: &Shape) -> KernelResult<Shape> {
        let n = CLONE_SEQ.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("cadre-occt-clone-{n}.step"));
        shape
            .write_step(&path)
            .map_err(|e| Self::map_occt_err("clone/write_step", e))?;
        let out = Shape::read_step(&path).map_err(|e| Self::map_occt_err("clone/read_step", e))?;
        let _ = std::fs::remove_file(&path);
        Ok(out)
    }

    fn bbox_from_mesh(shape: &Shape) -> BBox {
        let mesh = shape.mesh();
        if mesh.vertices.is_empty() {
            return BBox::from_min_max(Point3::ORIGIN, Point3::ORIGIN);
        }
        let mut min = Point3::new(mesh.vertices[0].x, mesh.vertices[0].y, mesh.vertices[0].z);
        let mut max = min;
        for v in mesh.vertices.iter().skip(1) {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            min.z = min.z.min(v.z);
            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
            max.z = max.z.max(v.z);
        }
        BBox::from_min_max(min, max)
    }

    /// Volume estimate from tessellation (signed tetrahedra from origin).
    /// Good enough for golden tests within a few percent; exact GProp needs sys ffi.
    fn volume_from_mesh(shape: &Shape) -> f64 {
        let mesh = shape.mesh();
        let v = &mesh.vertices;
        let mut vol = 0.0;
        for tri in mesh.indices.chunks_exact(3) {
            let a = v[tri[0]];
            let b = v[tri[1]];
            let c = v[tri[2]];
            // scalar triple product / 6
            vol += a.dot(b.cross(c)) / 6.0;
        }
        vol.abs()
    }

    fn area_from_mesh(shape: &Shape) -> f64 {
        let mesh = shape.mesh();
        let v = &mesh.vertices;
        let mut area = 0.0;
        for tri in mesh.indices.chunks_exact(3) {
            let a = v[tri[0]];
            let b = v[tri[1]];
            let c = v[tri[2]];
            area += (b - a).cross(c - a).length() * 0.5;
        }
        area
    }

    fn centroid_from_mesh(shape: &Shape) -> Point3 {
        let mesh = shape.mesh();
        if mesh.vertices.is_empty() {
            return Point3::ORIGIN;
        }
        let mut sx = 0.0;
        let mut sy = 0.0;
        let mut sz = 0.0;
        for p in &mesh.vertices {
            sx += p.x;
            sy += p.y;
            sz += p.z;
        }
        let n = mesh.vertices.len() as f64;
        Point3::new(sx / n, sy / n, sz / n)
    }
}

impl GeomKernel for OcctKernel {
    fn backend_id(&self) -> &'static str {
        "occt"
    }

    fn backend_version(&self) -> &str {
        "opencascade-0.2"
    }

    fn parity_eligible(&self) -> bool {
        true
    }

    fn box_solid(
        &mut self,
        dx: f64,
        dy: f64,
        dz: f64,
        placement: Placement,
    ) -> KernelResult<ShapeId> {
        if dx <= 0.0 || dy <= 0.0 || dz <= 0.0 {
            return Err(KernelError::invalid_arg(format!(
                "box dims must be > 0, got {dx},{dy},{dz}"
            )));
        }
        // Centered at placement.origin; OCCT make_box is corner-based.
        let o = placement.origin;
        let p1 = dvec3(o.x - dx * 0.5, o.y - dy * 0.5, o.z - dz * 0.5);
        let p2 = dvec3(o.x + dx * 0.5, o.y + dy * 0.5, o.z + dz * 0.5);
        let shape = AdHocShape::make_box_point_point(p1, p2).into_shape();
        Ok(self.alloc(shape))
    }

    fn cylinder(
        &mut self,
        radius: f64,
        height: f64,
        placement: Placement,
    ) -> KernelResult<ShapeId> {
        if radius <= 0.0 || height <= 0.0 {
            return Err(KernelError::invalid_arg(format!(
                "cylinder radius/height must be > 0, got r={radius} h={height}"
            )));
        }
        let o = placement.origin;
        let shape = AdHocShape::make_cylinder(dvec3(o.x, o.y, o.z), radius, height).into_shape();
        Ok(self.alloc(shape))
    }

    fn boolean(&mut self, op: BooleanOp, a: ShapeId, b: ShapeId) -> KernelResult<ShapeId> {
        let sa = self.get(a)?;
        let sb = self.get(b)?;
        let result = match op {
            BooleanOp::Union => sa.union(sb).shape,
            BooleanOp::Cut => sa.subtract(sb).shape,
            BooleanOp::Intersect => {
                let mut left = AdHocShape(Self::clone_shape(sa)?);
                left.intersect(sb);
                left.into_shape()
            }
        };
        Ok(self.alloc(result))
    }

    fn fillet(&mut self, shape: ShapeId, edges: &[EdgeRef], radius: f64) -> KernelResult<ShapeId> {
        if radius <= 0.0 {
            return Err(KernelError::invalid_arg(format!(
                "fillet radius must be > 0, got {radius}"
            )));
        }
        let mut work = Self::clone_shape(self.get(shape)?)?;
        if edges.is_empty() {
            work.fillet(radius);
        } else {
            let wanted: std::collections::HashSet<u32> = edges.iter().map(|e| e.0).collect();
            let mut selected = Vec::new();
            let mut i = 0u32;
            for edge in work.edges() {
                if wanted.contains(&i) {
                    selected.push(edge);
                }
                i += 1;
            }
            if selected.len() != wanted.len() {
                return Err(KernelError::diagnostic(
                    "CADRE-E-UNKNOWN-EDGE",
                    format!(
                        "requested {} edges, found {} on shape {shape}",
                        wanted.len(),
                        selected.len()
                    ),
                    Some("run inspect edges / use smaller indices".into()),
                ));
            }
            work.fillet_edges(radius, &selected);
        }
        Ok(self.alloc(work))
    }

    fn chamfer(
        &mut self,
        shape: ShapeId,
        edges: &[EdgeRef],
        distance: f64,
    ) -> KernelResult<ShapeId> {
        if distance <= 0.0 {
            return Err(KernelError::invalid_arg(format!(
                "chamfer distance must be > 0, got {distance}"
            )));
        }
        let mut work = Self::clone_shape(self.get(shape)?)?;
        if edges.is_empty() {
            work.chamfer(distance);
        } else {
            let wanted: std::collections::HashSet<u32> = edges.iter().map(|e| e.0).collect();
            let mut selected = Vec::new();
            let mut i = 0u32;
            for edge in work.edges() {
                if wanted.contains(&i) {
                    selected.push(edge);
                }
                i += 1;
            }
            work.chamfer_edges(distance, &selected);
        }
        Ok(self.alloc(work))
    }

    fn set_label(&mut self, shape: ShapeId, label: ShapeLabel) -> KernelResult<ShapeId> {
        let _ = self.get(shape)?;
        self.labels.insert(shape.0, label.0);
        Ok(shape)
    }

    fn facts(&self, shape: ShapeId) -> KernelResult<ShapeFacts> {
        let s = self.get(shape)?;
        let volume = Self::volume_from_mesh(s);
        let area = Self::area_from_mesh(s);
        let bbox = Self::bbox_from_mesh(s);
        let centroid = Self::centroid_from_mesh(s);
        let faces = s.faces().count() as u32;
        let edges = s.edges().count() as u32;
        Ok(ShapeFacts {
            bbox_mm: bbox,
            volume_mm3: volume,
            area_mm2: Some(area),
            centroid_mm: Some(centroid),
            solids: 1,
            faces,
            edges,
            vertices: None,
            mass_g: None,
        })
    }

    fn validity(&self, shape: ShapeId) -> KernelResult<ValidityReport> {
        let f = self.facts(shape)?;
        let mut notes = vec!["volume/area from tessellation (approx)".into()];
        if let Some(l) = self.labels.get(&shape.0) {
            notes.push(format!("label={l}"));
        }
        Ok(ValidityReport {
            closed: f.volume_mm3 > 0.0,
            positive_volume: f.volume_mm3 > 0.0,
            shells: 1,
            notes,
        })
    }

    fn edges(&self, shape: ShapeId) -> KernelResult<Vec<EdgeRef>> {
        let s = self.get(shape)?;
        let n = s.edges().count() as u32;
        Ok((0..n).map(EdgeRef).collect())
    }

    fn write_step(&self, shape: ShapeId, path: &Path, _opts: &StepWriteOpts) -> KernelResult<()> {
        let s = self.get(shape)?;
        s.write_step(path)
            .map_err(|e| Self::map_occt_err("write_step", e))
    }

    fn read_step(&mut self, path: &Path, _opts: &StepReadOpts) -> KernelResult<ShapeId> {
        let shape = Shape::read_step(path).map_err(|e| Self::map_occt_err("read_step", e))?;
        Ok(self.alloc(shape))
    }

    fn tessellate(&self, shape: ShapeId, _tol: TessTol) -> KernelResult<Mesh> {
        let s = self.get(shape)?;
        let m = s.mesh();
        let mut positions = Vec::with_capacity(m.vertices.len() * 3);
        for v in &m.vertices {
            positions.push(v.x as f32);
            positions.push(v.y as f32);
            positions.push(v.z as f32);
        }
        let mut normals = Vec::with_capacity(m.normals.len() * 3);
        for n in &m.normals {
            normals.push(n.x as f32);
            normals.push(n.y as f32);
            normals.push(n.z as f32);
        }
        let indices: Vec<u32> = m.indices.iter().map(|&i| i as u32).collect();
        Ok(Mesh {
            positions,
            normals: if normals.is_empty() {
                None
            } else {
                Some(normals)
            },
            indices,
        })
    }
}

/// Convenience: write STEP next to a logical name under `dir`.
pub fn step_path(dir: impl Into<PathBuf>, basename: &str) -> PathBuf {
    dir.into().join(format!("{basename}.step"))
}
