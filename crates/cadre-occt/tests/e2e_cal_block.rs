//! End-to-end: Starlark → IR → OCCT → STEP + facts.

use std::path::PathBuf;

use cadre_kernel::{GeomKernel, StepWriteOpts};
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
    let id = k
        .box_at(10.0, 20.0, 30.0, cadre_kernel::Point3::ORIGIN)
        .unwrap();
    let f = k.facts(id).unwrap();
    // Tessellated volume within 5% of 6000
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
fn calibration_block_star_to_step() {
    let r = evaluate(CAL_BLOCK, &EvalOptions::new("cal.cad.star"));
    assert!(r.ok, "{:?}", r.diagnostics);
    let ir = r.ir.expect("ir");

    // IR must include cut + fillet
    assert!(ir.nodes.iter().any(|n| matches!(
        n,
        cadre_lang::IrNode::Boolean {
            kind: cadre_lang::BooleanKind::Cut,
            ..
        }
    )));
    assert!(ir
        .nodes
        .iter()
        .any(|n| matches!(n, cadre_lang::IrNode::Fillet { .. })));

    let mut k = OcctKernel::new();
    let sid = execute_ir(&mut k, &ir).expect("execute");
    let f = k.facts(sid).unwrap();

    // Solid box 100*60*20 = 120000; hole π*4²*22 ≈ 1105; fillet removes a bit more.
    let expect = 120_000.0 - std::f64::consts::PI * 16.0 * 22.0;
    let err = (f.volume_mm3 - expect).abs() / expect;
    assert!(
        err < 0.08,
        "volume={} expect≈{} rel_err={}",
        f.volume_mm3,
        expect,
        err
    );

    let e = f.bbox_mm.extents_mm();
    assert!((e[0] - 100.0).abs() < 1.0, "dx={}", e[0]);
    assert!((e[1] - 60.0).abs() < 1.0, "dy={}", e[1]);
    assert!((e[2] - 20.0).abs() < 1.0, "dz={}", e[2]);

    let dir = tempfile::tempdir().unwrap();
    let path: PathBuf = dir.path().join("calibration_block.step");
    k.write_step(sid, &path, &StepWriteOpts::default()).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.len() > 500, "STEP too small: {}", bytes.len());
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(64)]);
    assert!(
        head.contains("ISO-10303") || head.contains("STEP") || bytes.starts_with(b"ISO"),
        "unexpected STEP header: {head:?}"
    );

    // Round-trip read
    let sid2 = k.read_step(&path, &Default::default()).unwrap();
    let f2 = k.facts(sid2).unwrap();
    assert!((f2.volume_mm3 - f.volume_mm3).abs() / f.volume_mm3 < 0.05);
}
