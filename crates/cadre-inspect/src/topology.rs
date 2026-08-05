//! Kernel-independent topology snapshot used for selector assignment.

use cadre_kernel::{Point3, Vec3};
use serde::{Deserialize, Serialize};

/// Vertex record (optional detail).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VertexRec {
    pub position: Point3,
}

/// Edge record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeRec {
    pub length_mm: f64,
    pub midpoint: Point3,
}

/// Face record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaceRec {
    pub area_mm2: f64,
    pub centroid: Point3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normal: Option<Vec3>,
}

/// Solid record with unordered faces/edges (ordering applied at inspect time).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolidRec {
    pub volume_mm3: f64,
    pub centroid: Point3,
    pub faces: Vec<FaceRec>,
    pub edges: Vec<EdgeRec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vertices: Vec<VertexRec>,
}

/// One inspected object (usually one root shape).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopologySnapshot {
    /// Object index (1-based), default 1 for single-body builds.
    pub object: u32,
    pub solids: Vec<SolidRec>,
}

impl TopologySnapshot {
    pub fn single_solid(solid: SolidRec) -> Self {
        Self {
            object: 1,
            solids: vec![solid],
        }
    }
}

/// Analytic topology for an axis-aligned box centered at `at`.
pub fn box_topology(dx: f64, dy: f64, dz: f64, at: Point3) -> SolidRec {
    let hx = dx / 2.0;
    let hy = dy / 2.0;
    let hz = dz / 2.0;
    let faces = vec![
        // +Z top, -Z bot, ±X, ±Y
        FaceRec {
            area_mm2: dx * dy,
            centroid: Point3::new(at.x, at.y, at.z + hz),
            normal: Some(Vec3::Z),
        },
        FaceRec {
            area_mm2: dx * dy,
            centroid: Point3::new(at.x, at.y, at.z - hz),
            normal: Some(Vec3::new(0.0, 0.0, -1.0)),
        },
        FaceRec {
            area_mm2: dy * dz,
            centroid: Point3::new(at.x + hx, at.y, at.z),
            normal: Some(Vec3::X),
        },
        FaceRec {
            area_mm2: dy * dz,
            centroid: Point3::new(at.x - hx, at.y, at.z),
            normal: Some(Vec3::new(-1.0, 0.0, 0.0)),
        },
        FaceRec {
            area_mm2: dx * dz,
            centroid: Point3::new(at.x, at.y + hy, at.z),
            normal: Some(Vec3::Y),
        },
        FaceRec {
            area_mm2: dx * dz,
            centroid: Point3::new(at.x, at.y - hy, at.z),
            normal: Some(Vec3::new(0.0, -1.0, 0.0)),
        },
    ];
    // 12 edges
    let mut edges = Vec::new();
    for &(len, mid) in &[
        (dx, Point3::new(at.x, at.y + hy, at.z + hz)),
        (dx, Point3::new(at.x, at.y - hy, at.z + hz)),
        (dx, Point3::new(at.x, at.y + hy, at.z - hz)),
        (dx, Point3::new(at.x, at.y - hy, at.z - hz)),
        (dy, Point3::new(at.x + hx, at.y, at.z + hz)),
        (dy, Point3::new(at.x - hx, at.y, at.z + hz)),
        (dy, Point3::new(at.x + hx, at.y, at.z - hz)),
        (dy, Point3::new(at.x - hx, at.y, at.z - hz)),
        (dz, Point3::new(at.x + hx, at.y + hy, at.z)),
        (dz, Point3::new(at.x - hx, at.y + hy, at.z)),
        (dz, Point3::new(at.x + hx, at.y - hy, at.z)),
        (dz, Point3::new(at.x - hx, at.y - hy, at.z)),
    ] {
        edges.push(EdgeRec {
            length_mm: len,
            midpoint: mid,
        });
    }
    SolidRec {
        volume_mm3: dx * dy * dz,
        centroid: at,
        faces,
        edges,
        vertices: Vec::new(),
    }
}

/// Analytic open cylinder (lateral + 2 caps) along +Z from base `at`.
pub fn cylinder_topology(radius: f64, height: f64, at: Point3) -> SolidRec {
    use std::f64::consts::PI;
    let r = radius;
    let h = height;
    let cx = at.x;
    let cy = at.y;
    let faces = vec![
        FaceRec {
            area_mm2: PI * r * r,
            centroid: Point3::new(cx, cy, at.z + h),
            normal: Some(Vec3::Z),
        },
        FaceRec {
            area_mm2: PI * r * r,
            centroid: Point3::new(cx, cy, at.z),
            normal: Some(Vec3::new(0.0, 0.0, -1.0)),
        },
        FaceRec {
            area_mm2: 2.0 * PI * r * h,
            centroid: Point3::new(cx, cy, at.z + h * 0.5),
            normal: None, // cylindrical
        },
    ];
    let edges = vec![
        EdgeRec {
            length_mm: 2.0 * PI * r,
            midpoint: Point3::new(cx + r, cy, at.z + h),
        },
        EdgeRec {
            length_mm: 2.0 * PI * r,
            midpoint: Point3::new(cx + r, cy, at.z),
        },
    ];
    SolidRec {
        volume_mm3: PI * r * r * h,
        centroid: Point3::new(cx, cy, at.z + h * 0.5),
        faces,
        edges,
        vertices: Vec::new(),
    }
}
