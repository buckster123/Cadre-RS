//! Local slicer CLI discovery (no reimplementation).

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlicerKind {
    PrusaSlicer,
    OrcaSlicer,
    BambuStudio,
    SuperSlicer,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlicerInfo {
    pub kind: SlicerKind,
    pub name: String,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

const CANDIDATES: &[(&str, SlicerKind)] = &[
    ("prusa-slicer", SlicerKind::PrusaSlicer),
    ("prusa-slicer-console", SlicerKind::PrusaSlicer),
    ("PrusaSlicer", SlicerKind::PrusaSlicer),
    ("orca-slicer", SlicerKind::OrcaSlicer),
    ("OrcaSlicer", SlicerKind::OrcaSlicer),
    ("bambu-studio", SlicerKind::BambuStudio),
    ("BambuStudio", SlicerKind::BambuStudio),
    ("superslicer", SlicerKind::SuperSlicer),
];

/// Discover slicer binaries on PATH (and a few common install dirs).
pub fn discover_slicers() -> Vec<SlicerInfo> {
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    for (bin, kind) in CANDIDATES {
        if let Some(path) = which(bin) {
            let key = path.display().to_string();
            if seen.insert(key) {
                let version = probe_version(&path);
                out.push(SlicerInfo {
                    kind: *kind,
                    name: bin.to_string(),
                    path,
                    version,
                });
            }
        }
    }

    // common Linux appimage / local bins
    let extras = [
        "/usr/bin/prusa-slicer",
        "/usr/local/bin/prusa-slicer",
        "/usr/bin/orca-slicer",
        "/opt/bambu-studio/bambu-studio",
    ];
    for p in extras {
        let path = PathBuf::from(p);
        if path.is_file() {
            let key = path.display().to_string();
            if seen.insert(key) {
                let kind = kind_from_name(p);
                let version = probe_version(&path);
                out.push(SlicerInfo {
                    kind,
                    name: path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("slicer")
                        .into(),
                    path,
                    version,
                });
            }
        }
    }
    out
}

fn kind_from_name(p: &str) -> SlicerKind {
    let l = p.to_ascii_lowercase();
    if l.contains("prusa") {
        SlicerKind::PrusaSlicer
    } else if l.contains("orca") {
        SlicerKind::OrcaSlicer
    } else if l.contains("bambu") {
        SlicerKind::BambuStudio
    } else if l.contains("super") {
        SlicerKind::SuperSlicer
    } else {
        SlicerKind::Unknown
    }
}

fn which(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn probe_version(path: &Path) -> Option<String> {
    let output = Command::new(path).arg("--version").output().ok()?;
    let s = String::from_utf8_lossy(&output.stdout);
    let line = s.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        let e = String::from_utf8_lossy(&output.stderr);
        let line = e.lines().next().unwrap_or("").trim();
        if line.is_empty() {
            None
        } else {
            Some(line.to_string())
        }
    } else {
        Some(line.to_string())
    }
}

/// Build a dry-run command line for documentation (does not execute).
pub fn slice_command_preview(
    slicer: &SlicerInfo,
    mesh: &Path,
    out_gcode: &Path,
    printer_profile: Option<&str>,
) -> String {
    match slicer.kind {
        SlicerKind::PrusaSlicer | SlicerKind::SuperSlicer | SlicerKind::OrcaSlicer => {
            let mut cmd = format!(
                "{} --export-gcode --output {} {}",
                slicer.path.display(),
                out_gcode.display(),
                mesh.display()
            );
            if let Some(p) = printer_profile {
                cmd.push_str(&format!(" --load {p}"));
            }
            cmd
        }
        SlicerKind::BambuStudio => format!(
            "{} --slice {}  # profile wiring is host-specific; Cadre does not reimplement slicing",
            slicer.path.display(),
            mesh.display()
        ),
        SlicerKind::Unknown => format!("{} {}", slicer.path.display(), mesh.display()),
    }
}
