//! End-to-end: Starlark → IR → OCCT → STEP + facts + topology.

use std::path::PathBuf;

use cadre_inspect::{inspect_refs, measure, MeasureKind, MeasureRequest};
use cadre_kernel::{BooleanOp, GeomKernel, Point3, StepWriteOpts};
use cadre_lang::{evaluate, execute_ir, EvalOptions};
use cadre_occt::OcctKernel;

/// Calibration-style block with center hole + light fillet (S3 fixture).
const CAL_BLOCK: &str = r#"
P = params(width=100.0, depth=60.0, height=20.0, hole_d=8.0, fillet_r=1.0)

def gen_step():
    blk = box(P.width, P.depth, P.height, at=CENTER)
    hole = cylinder(P.hole_d / 2.0, P.height + 2.0, at=(0.0, 0.0, -1.0))
    body = cut(blk, hole)
    body = fillet(body, radius=P.fillet_r)
    return solid(body, label="calibration_block")
"#;

#[test]
fn box_facts_and_step() {
    let mut k = OcctKernel::new();
    let id = k.box_at(10.0, 20.0, 30.0, Point3::ORIGIN).unwrap();
    let f = k.facts(id).unwrap();
    assert!(
        (f.volume_mm3 - 6000.0).abs() / 6000.0 < 0.05,
        "volume={}",
        f.volume_mm3
    );
    let e = f.bbox_mm.extents_mm();
    assert!((e[0] - 10.0).abs() < 0.5);
    assert!((e[1] - 20.0).abs() < 0.5);
    assert!((e[2] - 30.0).abs() < 0.5);

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("box.step");
    k.write_step(id, &path, &StepWriteOpts::default()).unwrap();
    assert!(path.metadata().unwrap().len() > 100);
}

#[test]
fn box_topology_has_six_faces_with_normals() {
    let mut k = OcctKernel::new();
    let id = k.box_at(10.0, 20.0, 30.0, Point3::ORIGIN).unwrap();
    let snap = k.topology_snapshot(id).unwrap();
    assert_eq!(snap.solids.len(), 1);
    let s = &snap.solids[0];
    assert_eq!(s.faces.len(), 6, "AABB box must have 6 faces");
    assert!(s.edges.len() >= 12, "edges={}", s.edges.len());
    let with_n = s.faces.iter().filter(|f| f.normal.is_some()).count();
    assert_eq!(with_n, 6, "all faces should carry normals");

    let report = inspect_refs(&snap, true);
    assert_eq!(report.faces, 6);
    let top = report
        .refs
        .iter()
        .find(|e| e.kind == "face" && e.normal.map(|n| (n.z - 1.0).abs() < 0.1).unwrap_or(false))
        .expect("top face");
    let bot = report
        .refs
        .iter()
        .find(|e| e.kind == "face" && e.normal.map(|n| (n.z + 1.0).abs() < 0.1).unwrap_or(false))
        .expect("bot face");
    let m = measure(
        &snap,
        &MeasureRequest {
            a: top.selector.clone(),
            b: Some(bot.selector.clone()),
            kind: MeasureKind::Thickness,
        },
    )
    .unwrap();
    assert!(
        (m.value - 30.0).abs() < 1.0,
        "thickness={} construction={}",
        m.value,
        m.construction
    );
}

#[test]
fn union_topology_live() {
    let mut k = OcctKernel::new();
    let a = k.box_at(10.0, 10.0, 10.0, Point3::ORIGIN).unwrap();
    let b = k
        .box_at(10.0, 10.0, 10.0, Point3::new(5.0, 0.0, 0.0))
        .unwrap();
    let u = k.boolean(BooleanOp::Union, a, b).unwrap();
    let snap = k.topology_snapshot(u).unwrap();
    assert!(snap.solids[0].faces.len() >= 6);
    let f = k.facts(u).unwrap();
    // two 1000 cubes overlapping 500 → ~1500
    assert!(
        (f.volume_mm3 - 1500.0).abs() / 1500.0 < 0.15,
        "union vol={}",
        f.volume_mm3
    );
}

/// Host OCCT 7.x + opencascade-rs 0.2 currently aborts inside BRepAlgoAPI_Cut
/// (C++ StdFail_NotDone) on this machine. Re-enable when cut is stable.
#[test]
#[ignore = "OCCT BRepAlgoAPI_Cut aborts StdFail_NotDone on host"]
fn calibration_block_star_to_step() {
    let r = evaluate(CAL_BLOCK, &EvalOptions::new("cal.cad.star"));
    assert!(r.ok, "{:?}", r.diagnostics);
    let ir = r.ir.expect("ir");
    let mut k = OcctKernel::new();
    let sid = execute_ir(&mut k, &ir).expect("execute");
    let f = k.facts(sid).unwrap();
    let expect = 120_000.0 - std::f64::consts::PI * 16.0 * 22.0;
    let err = (f.volume_mm3 - expect).abs() / expect;
    assert!(err < 0.08, "volume={} err={}", f.volume_mm3, err);
    let _ = PathBuf::from(".");
}

#[test]
#[ignore = "OCCT BRepAlgoAPI_Cut aborts StdFail_NotDone on host"]
fn parity_01_calibration_occt_volume() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity/parts/01_calibration_block");
    let star = std::fs::read_to_string(root.join("part.cad.star")).expect("star");
    let r = evaluate(&star, &EvalOptions::new("part.cad.star"));
    assert!(r.ok, "{:?}", r.diagnostics);
    let mut k = OcctKernel::new();
    let sid = execute_ir(&mut k, &r.ir.unwrap()).unwrap();
    let f = k.facts(sid).unwrap();
    assert!(f.volume_mm3 > 0.0);
}
