//! Run one or many parity parts.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use cadre_inspect::{inspect_refs, measure, MeasureKind, MeasureRequest, TopologySnapshot};
use cadre_kernel::{GeomKernel, MockKernel};
use cadre_lang::{evaluate, execute_ir, EvalOptions, FeatureIr, IrNode};
use serde::{Deserialize, Serialize};

use crate::expect::{Expect, FindFace};
use crate::SUITE_PARTS_1_4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartResult {
    pub id: String,
    pub ok: bool,
    pub wall_ms: u64,
    pub checks: Vec<CheckResult>,
    pub facts: Option<serde_json::Value>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteReport {
    pub suite: String,
    pub ok: bool,
    pub parts: Vec<PartResult>,
    pub passed: u32,
    pub failed: u32,
    pub wall_ms: u64,
}

/// Discover and run suite under `parity_root` (repo `parity/` dir).
pub fn run_suite(parity_root: &Path, suite: &str) -> Result<SuiteReport, String> {
    let started = Instant::now();
    let part_dirs = match suite {
        SUITE_PARTS_1_4 | "parity4" | "m1" => vec![
            "01_calibration_block",
            "02_bolt_circle_flange",
            "03_l_bracket",
            "04_stepped_shaft",
        ],
        other => return Err(format!("unknown suite: {other}")),
    };

    let mut parts = Vec::new();
    for name in part_dirs {
        let dir = parity_root.join("parts").join(name);
        let r = run_part(&dir)?;
        parts.push(r);
    }
    let passed = parts.iter().filter(|p| p.ok).count() as u32;
    let failed = parts.len() as u32 - passed;
    Ok(SuiteReport {
        suite: suite.to_string(),
        ok: failed == 0,
        parts,
        passed,
        failed,
        wall_ms: started.elapsed().as_millis() as u64,
    })
}

/// Run a single part directory containing `part.cad.star` + `expect.json`.
pub fn run_part(dir: &Path) -> Result<PartResult, String> {
    let started = Instant::now();
    let star = dir.join("part.cad.star");
    let exp_path = dir.join("expect.json");
    if !star.is_file() {
        return Err(format!("missing {}", star.display()));
    }
    if !exp_path.is_file() {
        return Err(format!("missing {}", exp_path.display()));
    }
    let source = fs::read_to_string(&star).map_err(|e| e.to_string())?;
    let expect: Expect =
        serde_json::from_str(&fs::read_to_string(&exp_path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("expect.json: {e}"))?;

    let mut checks = Vec::new();

    // 1. Evaluate
    let eval = evaluate(
        &source,
        &EvalOptions::new(
            star.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("part.cad.star"),
        ),
    );
    if !eval.ok {
        checks.push(CheckResult {
            name: "eval".into(),
            ok: false,
            detail: format!("{:?}", eval.diagnostics),
        });
        return Ok(PartResult {
            id: expect.id,
            ok: false,
            wall_ms: started.elapsed().as_millis() as u64,
            checks,
            facts: None,
            label: None,
        });
    }
    checks.push(CheckResult {
        name: "eval".into(),
        ok: true,
        detail: "ok".into(),
    });
    let ir = eval.ir.unwrap();

    // 2. Label
    let label_ok = ir.label.as_deref() == Some(expect.label.as_str());
    checks.push(CheckResult {
        name: "label".into(),
        ok: label_ok,
        detail: format!("got {:?} want {}", ir.label, expect.label),
    });

    // 3. Params
    let mut params_ok = true;
    let mut param_detail = String::new();
    for (k, v) in &expect.params {
        match ir.params.get(k) {
            Some(got) if (got - v).abs() < 1e-9 => {}
            Some(got) => {
                params_ok = false;
                param_detail.push_str(&format!("{k}: got {got} want {v}; "));
            }
            None => {
                params_ok = false;
                param_detail.push_str(&format!("{k}: missing; "));
            }
        }
    }
    if param_detail.is_empty() {
        param_detail = "ok".into();
    }
    checks.push(CheckResult {
        name: "params".into(),
        ok: params_ok,
        detail: param_detail,
    });

    // 4. Required IR ops
    let ops = collect_ops(&ir);
    let mut ops_ok = true;
    let mut ops_detail = String::new();
    for req in &expect.required_ops {
        if !ops.iter().any(|o| o == req) {
            ops_ok = false;
            ops_detail.push_str(&format!("missing op {req}; "));
        }
    }
    if ops_detail.is_empty() {
        ops_detail = format!("ops={ops:?}");
    }
    checks.push(CheckResult {
        name: "ir_ops".into(),
        ok: ops_ok,
        detail: ops_detail,
    });

    // 5. Execute on mock + facts
    let mut kernel = MockKernel::new();
    let shape = match execute_ir(&mut kernel, &ir) {
        Ok(s) => s,
        Err(e) => {
            checks.push(CheckResult {
                name: "execute".into(),
                ok: false,
                detail: e.to_string(),
            });
            return Ok(finish(expect, checks, None, started));
        }
    };
    checks.push(CheckResult {
        name: "execute".into(),
        ok: true,
        detail: "mock".into(),
    });

    let facts = kernel.facts(shape).map_err(|e| e.to_string())?;
    let vol_err = (facts.volume_mm3 - expect.volume_mm3).abs() / expect.volume_mm3.max(1e-9);
    let vol_ok = vol_err <= expect.volume_tol_frac;
    checks.push(CheckResult {
        name: "volume".into(),
        ok: vol_ok,
        detail: format!(
            "got {:.4} want {:.4} (rel_err={:.4}, tol={})",
            facts.volume_mm3, expect.volume_mm3, vol_err, expect.volume_tol_frac
        ),
    });

    let bb = &facts.bbox_mm;
    let bbox_ok = approx3(bb.min.x, expect.bbox_mm.min[0], expect.bbox_tol_mm)
        && approx3(bb.min.y, expect.bbox_mm.min[1], expect.bbox_tol_mm)
        && approx3(bb.min.z, expect.bbox_mm.min[2], expect.bbox_tol_mm)
        && approx3(bb.max.x, expect.bbox_mm.max[0], expect.bbox_tol_mm)
        && approx3(bb.max.y, expect.bbox_mm.max[1], expect.bbox_tol_mm)
        && approx3(bb.max.z, expect.bbox_mm.max[2], expect.bbox_tol_mm);
    checks.push(CheckResult {
        name: "bbox".into(),
        ok: bbox_ok,
        detail: format!(
            "got min=({:.3},{:.3},{:.3}) max=({:.3},{:.3},{:.3})",
            bb.min.x, bb.min.y, bb.min.z, bb.max.x, bb.max.y, bb.max.z
        ),
    });

    // 6. Topology / refs from IR
    let snap = crate_topo(&ir)?;
    let report = inspect_refs(&snap, true);
    let faces_ok = report.faces >= expect.faces_min;
    checks.push(CheckResult {
        name: "faces_min".into(),
        ok: faces_ok,
        detail: format!("faces={} min={}", report.faces, expect.faces_min),
    });
    let edges_ok = report.edges >= expect.edges_min;
    checks.push(CheckResult {
        name: "edges_min".into(),
        ok: edges_ok,
        detail: format!("edges={} min={}", report.edges, expect.edges_min),
    });

    // Stable selectors: re-run twice
    let report2 = inspect_refs(&snap, false);
    let sel_ok = report.refs.iter().map(|r| &r.selector).collect::<Vec<_>>()
        == report2.refs.iter().map(|r| &r.selector).collect::<Vec<_>>();
    checks.push(CheckResult {
        name: "selectors_stable".into(),
        ok: sel_ok,
        detail: format!("{} refs", report.refs.len()),
    });

    // 7. Measures
    for (i, m) in expect.measures.iter().enumerate() {
        let a = match find_selector(&report, &m.find_a) {
            Ok(s) => s,
            Err(e) => {
                checks.push(CheckResult {
                    name: format!("measure_{i}"),
                    ok: false,
                    detail: e,
                });
                continue;
            }
        };
        let b = if let Some(fb) = &m.find_b {
            Some(match find_selector(&report, fb) {
                Ok(s) => s,
                Err(e) => {
                    checks.push(CheckResult {
                        name: format!("measure_{i}"),
                        ok: false,
                        detail: e,
                    });
                    continue;
                }
            })
        } else {
            None
        };
        let kind = match m.kind.as_str() {
            "distance" => MeasureKind::Distance,
            "angle" => MeasureKind::Angle,
            "diameter" => MeasureKind::Diameter,
            "thickness" => MeasureKind::Thickness,
            other => {
                checks.push(CheckResult {
                    name: format!("measure_{i}"),
                    ok: false,
                    detail: format!("unknown kind {other}"),
                });
                continue;
            }
        };
        match measure(
            &snap,
            &MeasureRequest {
                a: a.clone(),
                b: b.clone(),
                kind,
            },
        ) {
            Ok(r) => {
                let ok = if let Some(vmin) = m.value_min {
                    r.value + m.tol >= vmin
                } else if let Some(v) = m.value {
                    (r.value - v).abs() <= m.tol
                } else {
                    true
                };
                checks.push(CheckResult {
                    name: format!("measure_{i}_{}", m.kind),
                    ok,
                    detail: format!("got {} {} ({})", r.value, r.unit, r.construction),
                });
            }
            Err(e) => checks.push(CheckResult {
                name: format!("measure_{i}"),
                ok: false,
                detail: e.to_string(),
            }),
        }
    }

    let facts_json = serde_json::to_value(&facts).ok();
    Ok(finish(expect, checks, facts_json, started))
}

fn finish(
    expect: Expect,
    checks: Vec<CheckResult>,
    facts: Option<serde_json::Value>,
    started: Instant,
) -> PartResult {
    let ok = checks.iter().all(|c| c.ok);
    PartResult {
        id: expect.id,
        ok,
        wall_ms: started.elapsed().as_millis() as u64,
        checks,
        facts,
        label: Some(expect.label),
    }
}

fn collect_ops(ir: &FeatureIr) -> Vec<String> {
    ir.nodes
        .iter()
        .map(|n| {
            match n {
                IrNode::Box { .. } => "box",
                IrNode::Cylinder { .. } => "cylinder",
                IrNode::Boolean { .. } => "boolean",
                IrNode::Fillet { .. } => "fillet",
                IrNode::Chamfer { .. } => "chamfer",
                IrNode::Label { .. } => "label",
            }
            .to_string()
        })
        .collect()
}

fn crate_topo(ir: &FeatureIr) -> Result<TopologySnapshot, String> {
    // Reuse CLI logic inline — duplicate small walker to avoid depending on cadre-cli.
    use cadre_inspect::{box_topology, cylinder_topology, SolidRec};
    use cadre_kernel::Point3;
    use cadre_lang::BooleanKind;

    let mut solids: Vec<Option<SolidRec>> = vec![None; ir.nodes.len()];
    for (idx, node) in ir.nodes.iter().enumerate() {
        let rec = match node {
            IrNode::Box { dx, dy, dz, at } => {
                box_topology(*dx, *dy, *dz, Point3::new(at[0], at[1], at[2]))
            }
            IrNode::Cylinder { radius, height, at } => {
                cylinder_topology(*radius, *height, Point3::new(at[0], at[1], at[2]))
            }
            IrNode::Boolean { kind, a, b } => {
                let sa = solids
                    .get(a.0 as usize)
                    .and_then(|s| s.as_ref())
                    .ok_or_else(|| format!("missing node {}", a.0))?;
                let sb = solids
                    .get(b.0 as usize)
                    .and_then(|s| s.as_ref())
                    .ok_or_else(|| format!("missing node {}", b.0))?;
                let volume = match kind {
                    BooleanKind::Union => sa.volume_mm3 + sb.volume_mm3,
                    BooleanKind::Cut => (sa.volume_mm3 - sb.volume_mm3).max(0.0),
                    BooleanKind::Intersect => sa.volume_mm3.min(sb.volume_mm3),
                };
                SolidRec {
                    volume_mm3: volume,
                    centroid: sa.centroid,
                    faces: sa.faces.clone(),
                    edges: sa.edges.clone(),
                    vertices: sa.vertices.clone(),
                }
            }
            IrNode::Fillet { of, .. } | IrNode::Chamfer { of, .. } | IrNode::Label { of, .. } => {
                solids
                    .get(of.0 as usize)
                    .and_then(|s| s.as_ref())
                    .ok_or_else(|| format!("missing node {}", of.0))?
                    .clone()
            }
        };
        solids[idx] = Some(rec);
    }
    let root = solids
        .get(ir.root.0 as usize)
        .and_then(|s| s.as_ref())
        .ok_or("missing root")?
        .clone();
    Ok(TopologySnapshot::single_solid(root))
}

fn find_selector(report: &cadre_inspect::RefsReport, find: &FindFace) -> Result<String, String> {
    if find.kind != "face" {
        return Err(format!("find kind {} not supported yet", find.kind));
    }
    let n = find.normal.ok_or("find face needs normal")?;
    let hit = report.refs.iter().find(|r| {
        r.kind == "face"
            && r.normal
                .map(|nr| {
                    (nr.x - n[0]).abs() < 1e-6
                        && (nr.y - n[1]).abs() < 1e-6
                        && (nr.z - n[2]).abs() < 1e-6
                })
                .unwrap_or(false)
    });
    hit.map(|r| r.selector.clone())
        .ok_or_else(|| format!("no face with normal {n:?}"))
}

fn approx3(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

/// Locate repo `parity/` from CARGO_MANIFEST_DIR or cwd.
pub fn default_parity_root() -> PathBuf {
    // cadre-bench crate is crates/cadre-bench → ../../parity
    let from_crate = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity");
    if from_crate.is_dir() {
        return from_crate.canonicalize().unwrap_or(from_crate);
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cand = cwd.join("parity");
    if cand.is_dir() {
        return cand;
    }
    cwd.join("parity")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_1_4_pass() {
        let root = default_parity_root();
        assert!(
            root.join("parts/01_calibration_block/part.cad.star")
                .is_file(),
            "parity root missing: {}",
            root.display()
        );
        let report = run_suite(&root, SUITE_PARTS_1_4).expect("suite");
        if !report.ok {
            for p in &report.parts {
                if !p.ok {
                    eprintln!("FAIL {}:", p.id);
                    for c in &p.checks {
                        if !c.ok {
                            eprintln!("  {} — {}", c.name, c.detail);
                        }
                    }
                }
            }
        }
        assert!(
            report.ok,
            "suite failed: {}/{} passed",
            report.passed,
            report.passed + report.failed
        );
        assert_eq!(report.parts.len(), 4);
    }
}
