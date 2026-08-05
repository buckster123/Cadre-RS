//! OCCT translate / rotate smoke.
//!
//! OCCT global state is not thread-safe across concurrent STEP I/O — keep this
//! file as a **single** test so cargo doesn't parallelize siblings.

use cadre_kernel::{GeomKernel, Placement, Point3};
use cadre_lang::{evaluate, execute_ir, EvalOptions};
use cadre_occt::OcctKernel;

#[test]
fn translate_rotate_and_finned_serial() {
    // translate
    {
        let mut k = OcctKernel::new();
        let s = k
            .box_solid(20.0, 10.0, 5.0, Placement::at(Point3::ORIGIN))
            .unwrap();
        let t = k.translate(s, 30.0, 0.0, 0.0).expect("translate");
        let f = k.facts(t).unwrap();
        assert!(
            (f.volume_mm3 - 1000.0).abs() / 1000.0 < 0.1,
            "translate vol {}",
            f.volume_mm3
        );
        assert!(f.bbox_mm.center().x > 20.0);
    }

    // rotate Z
    {
        let mut k = OcctKernel::new();
        let s = k
            .box_solid(20.0, 10.0, 5.0, Placement::at(Point3::ORIGIN))
            .unwrap();
        let t = k.rotate_about_axis(s, "z", 45.0).expect("rotate");
        let f = k.facts(t).unwrap();
        assert!(
            (f.volume_mm3 - 1000.0).abs() / 1000.0 < 0.1,
            "rotate vol {}",
            f.volume_mm3
        );
    }

    // star: translate + rotate_z
    {
        let src = r#"
def gen_step():
    b = box(20.0, 10.0, 5.0, at=CENTER)
    b = translate(b, 30.0, 0.0, 0.0)
    b = rotate_z(b, 45.0)
    return solid(b, label="xf")
"#;
        let r = evaluate(src, &EvalOptions::new("xf.cad.star"));
        assert!(r.ok, "{:?}", r.diagnostics);
        let ir = r.ir.unwrap();
        let mut k = OcctKernel::new();
        let sid = execute_ir(&mut k, &ir).expect("execute");
        let f = k.facts(sid).expect("facts");
        assert!(
            (f.volume_mm3 - 1000.0).abs() / 1000.0 < 0.1,
            "star vol {}",
            f.volume_mm3
        );
    }

    // parity part 07 (uses rotate/translate heavily)
    {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../parity/parts/07_finned_cylinder/part.cad.star"
        ))
        .expect("part");
        let r = evaluate(&src, &EvalOptions::new("fin.cad.star"));
        assert!(r.ok, "{:?}", r.diagnostics);
        let ir = r.ir.unwrap();
        let mut k = OcctKernel::new();
        let sid = execute_ir(&mut k, &ir).expect("execute fins");
        let f = k.facts(sid).expect("facts");
        assert!(f.volume_mm3 > 5000.0, "fin volume {}", f.volume_mm3);
        let dir = tempfile::tempdir().unwrap();
        let step = dir.path().join("fin.step");
        k.write_step(sid, &step, &Default::default()).expect("step");
        assert!(step.is_file());
    }
}
