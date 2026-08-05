//! Snapshot / view CLI tests.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

const BOX: &str = r#"
def gen_step():
    return solid(box(30.0, 20.0, 10.0, at=CENTER), label="snapbox")
"#;

#[test]
fn snapshot_writes_packet() {
    let dir = tempdir().unwrap();
    let star = dir.path().join("box.cad.star");
    fs::write(&star, BOX).unwrap();

    cargo_bin_cmd!("cadre")
        .current_dir(dir.path())
        .args([
            "--json",
            "snapshot",
            "box.cad.star",
            "--size",
            "64",
            "--gif-frames",
            "6",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("orbit.gif"));

    let snap = dir.path().join("box.snap");
    assert!(snap.join("iso.png").is_file());
    assert!(snap.join("front.png").is_file());
    assert!(snap.join("orbit.gif").is_file());
    assert!(snap.join("manifest.json").is_file());
}

#[test]
fn view_once_prepares_snap() {
    let dir = tempdir().unwrap();
    let star = dir.path().join("box.cad.star");
    fs::write(&star, BOX).unwrap();

    cargo_bin_cmd!("cadre")
        .current_dir(dir.path())
        .args(["--json", "view", "box.cad.star", "--once"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"once\": true"));

    assert!(dir.path().join("box.snap/iso.png").is_file());
}
