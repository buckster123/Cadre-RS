//! Minimal cut smoke — ignored while host OCCT cut aborts.

use cadre_kernel::{BooleanOp, GeomKernel, Point3};
use cadre_occt::OcctKernel;

#[test]
#[ignore = "OCCT BRepAlgoAPI_Cut aborts StdFail_NotDone on host"]
fn cut_cylinder_from_box() {
    let mut k = OcctKernel::new();
    let a = k.box_at(20.0, 20.0, 20.0, Point3::ORIGIN).unwrap();
    let b = k
        .cylinder_at(5.0, 30.0, Point3::new(0.0, 0.0, -5.0))
        .unwrap();
    let c = k.boolean(BooleanOp::Cut, a, b).expect("cut");
    let f = k.facts(c).unwrap();
    assert!(f.volume_mm3 > 1000.0);
}
