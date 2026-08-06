//! DFM engine + versioned vendor profiles (data-driven).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DfmSeverity {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfmFinding {
    pub rule: String,
    pub severity: DfmSeverity,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfmReport {
    pub ok: bool,
    pub profile_id: String,
    pub profile_version: String,
    pub findings: Vec<DfmFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfmProfile {
    pub id: String,
    pub version: String,
    pub vendor: String,
    /// mm
    pub materials: Vec<MaterialOption>,
    pub rules: DfmRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialOption {
    pub name: String,
    /// available thicknesses mm
    pub thicknesses_mm: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfmRules {
    /// min hole diameter as multiple of thickness (e.g. 1.0 => d >= t)
    pub min_hole_dia_vs_thickness: f64,
    /// absolute min hole diameter mm
    pub min_hole_dia_mm: f64,
    /// min web/bridge between holes or to edge mm
    pub min_web_mm: f64,
    /// min feature overall size mm
    pub min_part_size_mm: f64,
}

/// Abstract flat part for checks (from DXF/projection facts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatPart {
    pub width_mm: f64,
    pub height_mm: f64,
    pub thickness_mm: f64,
    pub material: String,
    /// hole diameters mm
    pub holes_dia_mm: Vec<f64>,
    /// optional min edge distance for holes mm
    #[serde(default)]
    pub min_hole_edge_mm: Option<f64>,
    /// optional min hole-hole spacing mm
    #[serde(default)]
    pub min_hole_spacing_mm: Option<f64>,
}

/// Built-in SendCutSend-style laser profile (profile-version truth, not live vendor API).
pub fn sendcutsend_laser_v1() -> DfmProfile {
    DfmProfile {
        id: "sendcutsend.laser".into(),
        version: "1.0.0".into(),
        vendor: "SendCutSend-style (bundled profile)".into(),
        materials: vec![
            MaterialOption {
                name: "Aluminum 5052".into(),
                thicknesses_mm: vec![1.0, 1.5, 2.0, 3.0, 4.0, 6.0],
            },
            MaterialOption {
                name: "Stainless 304".into(),
                thicknesses_mm: vec![1.0, 1.5, 2.0, 3.0],
            },
            MaterialOption {
                name: "Mild Steel".into(),
                thicknesses_mm: vec![1.5, 2.0, 3.0, 6.0],
            },
        ],
        rules: DfmRules {
            min_hole_dia_vs_thickness: 1.0,
            min_hole_dia_mm: 1.0,
            min_web_mm: 1.0,
            min_part_size_mm: 6.0,
        },
    }
}

/// Bundled PCB outline profile (generic fab house, not a live API).
/// Stricter holes / smaller webs typical of FR4 routing.
pub fn pcb_outline_v1() -> DfmProfile {
    DfmProfile {
        id: "pcb.outline".into(),
        version: "1.0.0".into(),
        vendor: "Generic PCB outline (bundled profile)".into(),
        materials: vec![
            MaterialOption {
                name: "FR4".into(),
                thicknesses_mm: vec![0.8, 1.0, 1.2, 1.6, 2.0],
            },
            MaterialOption {
                name: "Aluminum PCB".into(),
                thicknesses_mm: vec![1.0, 1.5, 2.0],
            },
        ],
        rules: DfmRules {
            min_hole_dia_vs_thickness: 0.25,
            min_hole_dia_mm: 0.3,
            min_web_mm: 0.25,
            min_part_size_mm: 5.0,
        },
    }
}

/// Bundled waterjet / abrasive-cut style profile (generic, not a live vendor API).
pub fn waterjet_v1() -> DfmProfile {
    DfmProfile {
        id: "waterjet.generic".into(),
        version: "1.0.0".into(),
        vendor: "Generic waterjet (bundled profile)".into(),
        materials: vec![
            MaterialOption {
                name: "Aluminum 6061".into(),
                thicknesses_mm: vec![1.5, 3.0, 6.0, 12.0, 25.0],
            },
            MaterialOption {
                name: "Stainless 304".into(),
                thicknesses_mm: vec![1.5, 3.0, 6.0, 12.0],
            },
            MaterialOption {
                name: "Mild Steel".into(),
                thicknesses_mm: vec![3.0, 6.0, 10.0, 20.0],
            },
            MaterialOption {
                name: "HDPE".into(),
                thicknesses_mm: vec![3.0, 6.0, 12.0, 25.0],
            },
        ],
        rules: DfmRules {
            // Waterjet tolerates smaller holes vs thickness than laser in some shops
            min_hole_dia_vs_thickness: 0.5,
            min_hole_dia_mm: 1.5,
            min_web_mm: 1.5,
            min_part_size_mm: 10.0,
        },
    }
}

/// All bundled profiles (id + version).
pub fn bundled_profiles() -> Vec<DfmProfile> {
    vec![sendcutsend_laser_v1(), pcb_outline_v1(), waterjet_v1()]
}

pub fn resolve_bundled_profile(id: &str) -> Option<DfmProfile> {
    match id {
        "sendcutsend.laser" | "sendcutsend.laser@1" | "scs" => Some(sendcutsend_laser_v1()),
        "pcb.outline" | "pcb.outline@1" | "pcb" | "jlcpcb.outline" => Some(pcb_outline_v1()),
        "waterjet.generic" | "waterjet.generic@1" | "waterjet" | "wj" => Some(waterjet_v1()),
        _ => None,
    }
}

pub fn check_dfm(profile: &DfmProfile, part: &FlatPart) -> DfmReport {
    let mut findings = Vec::new();

    // material + thickness availability
    let mat = profile
        .materials
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case(&part.material));
    match mat {
        None => findings.push(DfmFinding {
            rule: "material_available".into(),
            severity: DfmSeverity::Fail,
            message: format!(
                "material '{}' not in profile {}@{}",
                part.material, profile.id, profile.version
            ),
            measured: None,
            limit: None,
        }),
        Some(m) => {
            let ok_t = m
                .thicknesses_mm
                .iter()
                .any(|t| (*t - part.thickness_mm).abs() < 1e-6);
            if ok_t {
                findings.push(DfmFinding {
                    rule: "material_available".into(),
                    severity: DfmSeverity::Pass,
                    message: format!(
                        "{} @ {} mm available in profile",
                        part.material, part.thickness_mm
                    ),
                    measured: Some(part.thickness_mm),
                    limit: None,
                });
            } else {
                findings.push(DfmFinding {
                    rule: "thickness_available".into(),
                    severity: DfmSeverity::Fail,
                    message: format!(
                        "thickness {} mm not listed for {} (have {:?})",
                        part.thickness_mm, part.material, m.thicknesses_mm
                    ),
                    measured: Some(part.thickness_mm),
                    limit: None,
                });
            }
        }
    }

    // part size
    let min_dim = part.width_mm.min(part.height_mm);
    if min_dim + 1e-9 < profile.rules.min_part_size_mm {
        findings.push(DfmFinding {
            rule: "min_part_size".into(),
            severity: DfmSeverity::Fail,
            message: format!(
                "min dimension {min_dim} mm < {}",
                profile.rules.min_part_size_mm
            ),
            measured: Some(min_dim),
            limit: Some(profile.rules.min_part_size_mm),
        });
    } else {
        findings.push(DfmFinding {
            rule: "min_part_size".into(),
            severity: DfmSeverity::Pass,
            message: format!("min dimension {min_dim} mm ok"),
            measured: Some(min_dim),
            limit: Some(profile.rules.min_part_size_mm),
        });
    }

    // holes
    for (i, d) in part.holes_dia_mm.iter().enumerate() {
        let need = profile
            .rules
            .min_hole_dia_mm
            .max(profile.rules.min_web_mm) // keep simple
            .max(part.thickness_mm * profile.rules.min_hole_dia_vs_thickness);
        if *d + 1e-9 < need {
            findings.push(DfmFinding {
                rule: format!("hole_dia[{i}]"),
                severity: DfmSeverity::Fail,
                message: format!("hole dia {d} mm < required {need} mm (t-based)"),
                measured: Some(*d),
                limit: Some(need),
            });
        } else {
            findings.push(DfmFinding {
                rule: format!("hole_dia[{i}]"),
                severity: DfmSeverity::Pass,
                message: format!("hole dia {d} mm ok (>= {need})"),
                measured: Some(*d),
                limit: Some(need),
            });
        }
    }

    if let Some(edge) = part.min_hole_edge_mm {
        if edge + 1e-9 < profile.rules.min_web_mm {
            findings.push(DfmFinding {
                rule: "hole_edge_web".into(),
                severity: DfmSeverity::Fail,
                message: format!(
                    "hole-edge distance {edge} mm < min web {}",
                    profile.rules.min_web_mm
                ),
                measured: Some(edge),
                limit: Some(profile.rules.min_web_mm),
            });
        } else {
            findings.push(DfmFinding {
                rule: "hole_edge_web".into(),
                severity: DfmSeverity::Pass,
                message: format!("hole-edge {edge} mm ok"),
                measured: Some(edge),
                limit: Some(profile.rules.min_web_mm),
            });
        }
    }

    if let Some(sp) = part.min_hole_spacing_mm {
        if sp + 1e-9 < profile.rules.min_web_mm {
            findings.push(DfmFinding {
                rule: "hole_spacing_web".into(),
                severity: DfmSeverity::Warn,
                message: format!(
                    "hole spacing {sp} mm < min web {}",
                    profile.rules.min_web_mm
                ),
                measured: Some(sp),
                limit: Some(profile.rules.min_web_mm),
            });
        } else {
            findings.push(DfmFinding {
                rule: "hole_spacing_web".into(),
                severity: DfmSeverity::Pass,
                message: format!("hole spacing {sp} mm ok"),
                measured: Some(sp),
                limit: Some(profile.rules.min_web_mm),
            });
        }
    }

    let ok = !findings.iter().any(|f| f.severity == DfmSeverity::Fail);
    DfmReport {
        ok,
        profile_id: profile.id.clone(),
        profile_version: profile.version.clone(),
        findings,
    }
}

pub fn load_profile_json(text: &str) -> Result<DfmProfile, String> {
    serde_json::from_str(text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_plate_passes() {
        let p = sendcutsend_laser_v1();
        let part = FlatPart {
            width_mm: 100.0,
            height_mm: 50.0,
            thickness_mm: 3.0,
            material: "Aluminum 5052".into(),
            holes_dia_mm: vec![6.0, 6.0],
            min_hole_edge_mm: Some(5.0),
            min_hole_spacing_mm: Some(10.0),
        };
        let r = check_dfm(&p, &part);
        assert!(r.ok, "{r:?}");
    }

    #[test]
    fn tiny_hole_fails() {
        let p = sendcutsend_laser_v1();
        let part = FlatPart {
            width_mm: 40.0,
            height_mm: 40.0,
            thickness_mm: 3.0,
            material: "Aluminum 5052".into(),
            holes_dia_mm: vec![1.5],
            min_hole_edge_mm: Some(5.0),
            min_hole_spacing_mm: None,
        };
        let r = check_dfm(&p, &part);
        assert!(!r.ok);
        assert!(r
            .findings
            .iter()
            .any(|f| f.rule.starts_with("hole_dia") && f.severity == DfmSeverity::Fail));
    }

    #[test]
    fn pcb_profile_allows_small_via() {
        let p = pcb_outline_v1();
        assert_eq!(p.id, "pcb.outline");
        let part = FlatPart {
            width_mm: 50.0,
            height_mm: 40.0,
            thickness_mm: 1.6,
            material: "FR4".into(),
            holes_dia_mm: vec![0.4],
            min_hole_edge_mm: Some(1.0),
            min_hole_spacing_mm: Some(0.5),
        };
        let r = check_dfm(&p, &part);
        assert!(r.ok, "{r:?}");
    }

    #[test]
    fn resolve_bundled_ids() {
        assert!(resolve_bundled_profile("scs").is_some());
        assert!(resolve_bundled_profile("pcb").is_some());
        assert!(resolve_bundled_profile("waterjet").is_some());
        assert!(resolve_bundled_profile("nope").is_none());
        assert_eq!(bundled_profiles().len(), 3);
    }

    #[test]
    fn waterjet_plate_passes() {
        let p = waterjet_v1();
        assert_eq!(p.id, "waterjet.generic");
        let part = FlatPart {
            width_mm: 120.0,
            height_mm: 80.0,
            thickness_mm: 6.0,
            material: "Aluminum 6061".into(),
            holes_dia_mm: vec![4.0],
            min_hole_edge_mm: Some(5.0),
            min_hole_spacing_mm: Some(8.0),
        };
        let r = check_dfm(&p, &part);
        assert!(r.ok, "{r:?}");
    }
}
