//! Build [`TopologySnapshot`] from live OCCT shapes (faces/edges with COM + normals).

use cadre_inspect::{EdgeRec, FaceRec, SolidRec, TopologySnapshot};
use cadre_kernel::{GeomKernel, KernelResult, Point3, ShapeId, Vec3};
use glam::DVec3;
use opencascade::primitives::{Edge, Face, Shape};

use crate::OcctKernel;

impl OcctKernel {
    /// Topology snapshot from a live B-rep shape (for inspect/measure under `--kernel occt`).
    pub fn topology_snapshot(&self, shape: ShapeId) -> KernelResult<TopologySnapshot> {
        let s = self.get_pub(shape)?;
        let volume = self.facts(shape)?.volume_mm3;
        let solid = solid_from_shape(s, volume);
        Ok(TopologySnapshot::single_solid(solid))
    }
}

fn solid_from_shape(s: &Shape, volume_mm3: f64) -> SolidRec {
    let mut faces = Vec::new();
    for f in s.faces() {
        let c = f.center_of_mass();
        let n = f.normal_at_center();
        let nlen = (n.x * n.x + n.y * n.y + n.z * n.z).sqrt();
        let normal = if nlen > 1e-12 {
            Some(Vec3::new(n.x / nlen, n.y / nlen, n.z / nlen))
        } else {
            None
        };
        faces.push(FaceRec {
            area_mm2: face_area_approx(&f),
            centroid: Point3::new(c.x, c.y, c.z),
            normal,
        });
    }

    let mut edges = Vec::new();
    for e in s.edges() {
        let (len, mid) = edge_length_mid(&e);
        edges.push(EdgeRec {
            length_mm: len,
            midpoint: mid,
        });
    }

    let centroid = if faces.is_empty() {
        Point3::ORIGIN
    } else {
        let mut sx = 0.0;
        let mut sy = 0.0;
        let mut sz = 0.0;
        for f in &faces {
            sx += f.centroid.x;
            sy += f.centroid.y;
            sz += f.centroid.z;
        }
        let n = faces.len() as f64;
        Point3::new(sx / n, sy / n, sz / n)
    };

    SolidRec {
        volume_mm3,
        centroid,
        faces,
        edges,
        vertices: Vec::new(),
    }
}

fn face_area_approx(f: &Face) -> f64 {
    let mut pts: Vec<DVec3> = Vec::new();
    for e in f.edges() {
        let segs: Vec<DVec3> = e.approximation_segments().collect();
        let segs = if segs.is_empty() {
            vec![e.start_point(), e.end_point()]
        } else {
            segs
        };
        if pts.is_empty() {
            pts.extend(segs);
        } else if let Some(last) = pts.last().copied() {
            for p in segs {
                if (p - last).length() > 1e-6 {
                    pts.push(p);
                }
            }
        }
    }
    if pts.len() < 3 {
        return 0.0;
    }
    let n = f.normal_at_center();
    let nlen = n.length();
    if nlen < 1e-12 {
        return 0.0;
    }
    let nn = n / nlen;
    let mut t = DVec3::new(0.0, 0.0, 1.0).cross(nn);
    if t.length() < 1e-6 {
        t = DVec3::new(1.0, 0.0, 0.0).cross(nn);
    }
    t = t.normalize();
    let b = nn.cross(t);
    let origin = f.center_of_mass();
    let uv: Vec<(f64, f64)> = pts
        .iter()
        .map(|p| {
            let d = *p - origin;
            (d.dot(t), d.dot(b))
        })
        .collect();
    let mut a = 0.0;
    for i in 0..uv.len() {
        let (u1, v1) = uv[i];
        let (u2, v2) = uv[(i + 1) % uv.len()];
        a += u1 * v2 - u2 * v1;
    }
    (a * 0.5).abs()
}

fn edge_length_mid(e: &Edge) -> (f64, Point3) {
    let pts: Vec<DVec3> = e.approximation_segments().collect();
    if pts.len() < 2 {
        let a = e.start_point();
        let b = e.end_point();
        let mid = Point3::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5, (a.z + b.z) * 0.5);
        return ((b - a).length(), mid);
    }
    let mut len = 0.0;
    for w in pts.windows(2) {
        len += (w[1] - w[0]).length();
    }
    let mid_pt = pts[pts.len() / 2];
    (len, Point3::new(mid_pt.x, mid_pt.y, mid_pt.z))
}
