//! Assembly specification (FR-106 data model) + simple align check.

use cadre_kernel::{Point3, Vec3};
use serde::{Deserialize, Serialize};

/// Explicit placement: origin + axis-aligned for v0 (rotation as ZYX degrees later).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacementSpec {
    pub origin_mm: [f64; 3],
    /// Euler XYZ degrees (applied X then Y then Z) — identity default.
    #[serde(default)]
    pub rpy_deg: [f64; 3],
}

impl Default for PlacementSpec {
    fn default() -> Self {
        Self {
            origin_mm: [0.0, 0.0, 0.0],
            rpy_deg: [0.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentSpec {
    pub name: String,
    /// Path to `.cad.star` or lock key for catalog part.
    pub source: String,
    #[serde(default)]
    pub from_lock: bool,
    #[serde(default)]
    pub placement: PlacementSpec,
    #[serde(default)]
    pub datums: std::collections::BTreeMap<String, [f64; 3]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JointSpec {
    pub name: String,
    pub a: String,
    pub b: String,
    /// Kind: fixed | revolute | prismatic (v0 records only).
    #[serde(default = "fixed_kind")]
    pub kind: String,
}

fn fixed_kind() -> String {
    "fixed".into()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssemblySpec {
    pub name: String,
    pub version: u32,
    pub components: Vec<ComponentSpec>,
    #[serde(default)]
    pub joints: Vec<JointSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignExpect {
    Coplanar,
    Coaxial,
    Distance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignReport {
    pub ok: bool,
    pub expect: String,
    pub translation_err_mm: f64,
    pub angular_err_deg: f64,
    pub distance_mm: Option<f64>,
    pub tol_mm: f64,
    pub tol_deg: f64,
    pub detail: String,
}

/// Simple align between two world-space points + optional normals.
#[allow(clippy::too_many_arguments)]
pub fn align_check(
    a_origin: Point3,
    a_normal: Option<Vec3>,
    b_origin: Point3,
    b_normal: Option<Vec3>,
    expect: AlignExpect,
    expect_distance: Option<f64>,
    tol_mm: f64,
    tol_deg: f64,
) -> AlignReport {
    let dx = b_origin.x - a_origin.x;
    let dy = b_origin.y - a_origin.y;
    let dz = b_origin.z - a_origin.z;
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let ang = match (a_normal, b_normal) {
        (Some(na), Some(nb)) => {
            let dot = (na.x * nb.x + na.y * nb.y + na.z * nb.z).clamp(-1.0, 1.0);
            dot.abs().acos().to_degrees() // 0 = parallel (same or opposite)
        }
        _ => 0.0,
    };

    let (ok, detail) = match expect {
        AlignExpect::Coplanar => {
            // same plane if distance along normal ~ 0 when normals parallel
            let ok = ang <= tol_deg && dist <= tol_mm;
            (ok, format!("coplanar check ang={ang:.4} dist={dist:.4}"))
        }
        AlignExpect::Coaxial => {
            let ok = ang <= tol_deg;
            (ok, format!("coaxial/parallel normals ang={ang:.4}"))
        }
        AlignExpect::Distance => {
            let want = expect_distance.unwrap_or(0.0);
            let err = (dist - want).abs();
            (
                err <= tol_mm,
                format!("distance got={dist:.4} want={want:.4}"),
            )
        }
    };

    AlignReport {
        ok,
        expect: format!("{expect:?}").to_ascii_lowercase(),
        translation_err_mm: dist,
        angular_err_deg: ang,
        distance_mm: Some(dist),
        tol_mm,
        tol_deg,
        detail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_align() {
        let r = align_check(
            Point3::ORIGIN,
            None,
            Point3::new(10.0, 0.0, 0.0),
            None,
            AlignExpect::Distance,
            Some(10.0),
            0.1,
            1.0,
        );
        assert!(r.ok);
    }

    #[test]
    fn assembly_json_roundtrip() {
        let a = AssemblySpec {
            name: "bracket_assy".into(),
            version: 1,
            components: vec![
                ComponentSpec {
                    name: "plate".into(),
                    source: "plate.cad.star".into(),
                    from_lock: false,
                    placement: PlacementSpec::default(),
                    datums: Default::default(),
                },
                ComponentSpec {
                    name: "bolt".into(),
                    source: "m6_bolt".into(),
                    from_lock: true,
                    placement: PlacementSpec {
                        origin_mm: [0.0, 0.0, 5.0],
                        rpy_deg: [0.0, 0.0, 0.0],
                    },
                    datums: Default::default(),
                },
            ],
            joints: vec![JointSpec {
                name: "bolt_to_plate".into(),
                a: "plate".into(),
                b: "bolt".into(),
                kind: "fixed".into(),
            }],
        };
        let j = serde_json::to_string(&a).unwrap();
        let b: AssemblySpec = serde_json::from_str(&j).unwrap();
        assert_eq!(a, b);
    }
}
