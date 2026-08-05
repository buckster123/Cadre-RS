//! Printer adapters — Bambu LAN dry-run first; start is hard-gated.

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::gcode::{check_gcode, GcodeReport, PrinterVolume};

#[derive(Debug, Error)]
pub enum PrinterError {
    #[error("{0}")]
    Msg(String),
    #[error("start gate failed: {0}")]
    Gate(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterInfo {
    pub id: String,
    pub model: String,
    pub host: String,
    pub allowlisted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunReport {
    pub ok: bool,
    pub printer_id: String,
    pub gcode_sha256: String,
    pub gcode_check: GcodeReport,
    pub would_upload_to: String,
    pub notes: Vec<String>,
}

/// Explicit start confirmation token (must be exactly "START").
pub const CONFIRM_START: &str = "START";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRequest {
    pub printer_id: String,
    pub gcode_path: String,
    pub gcode_sha256: String,
    /// Must equal CONFIRM_START.
    pub confirm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartGate {
    pub ok: bool,
    pub errors: Vec<String>,
}

pub trait Printer: Send + Sync {
    fn info(&self) -> &PrinterInfo;
    fn status(&self) -> Result<serde_json::Value, PrinterError>;
    fn dry_run(
        &self,
        gcode_path: &Path,
        volume: &PrinterVolume,
    ) -> Result<DryRunReport, PrinterError>;
    fn start(
        &self,
        req: &StartRequest,
        allowlist: &BTreeSet<String>,
    ) -> Result<StartGate, PrinterError>;
}

/// Bambu Lab LAN adapter (alpha): **no network I/O** in dry-run/start.
/// Validates gates and reports what *would* happen. Live FTPS/MQTT is a later slice.
#[derive(Debug, Clone)]
pub struct BambuAdapter {
    info: PrinterInfo,
}

impl BambuAdapter {
    pub fn new(id: impl Into<String>, host: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            info: PrinterInfo {
                id: id.into(),
                model: model.into(),
                host: host.into(),
                allowlisted: false,
            },
        }
    }

    pub fn with_allowlisted(mut self, yes: bool) -> Self {
        self.info.allowlisted = yes;
        self
    }
}

impl Printer for BambuAdapter {
    fn info(&self) -> &PrinterInfo {
        &self.info
    }

    fn status(&self) -> Result<serde_json::Value, PrinterError> {
        Ok(serde_json::json!({
            "ok": true,
            "printer": self.info,
            "mode": "alpha-no-network",
            "state": "idle_simulated",
            "note": "Live MQTT/FTPS not enabled in S11; status is local stub."
        }))
    }

    fn dry_run(
        &self,
        gcode_path: &Path,
        volume: &PrinterVolume,
    ) -> Result<DryRunReport, PrinterError> {
        let bytes =
            std::fs::read(gcode_path).map_err(|e| PrinterError::Msg(format!("read gcode: {e}")))?;
        let sha = hex_sha256(&bytes);
        let text = String::from_utf8_lossy(&bytes);
        let gcode_check = check_gcode(&text, volume);
        let mut notes = vec![
            "dry-run only: no FTPS upload performed".into(),
            format!("target ftps://{} (not contacted)", self.info.host),
        ];
        if !gcode_check.ok {
            notes.push("gcode-check failed — upload would be refused".into());
        }
        Ok(DryRunReport {
            ok: gcode_check.ok,
            printer_id: self.info.id.clone(),
            gcode_sha256: sha,
            gcode_check,
            would_upload_to: format!("ftps://{}/", self.info.host),
            notes,
        })
    }

    fn start(
        &self,
        req: &StartRequest,
        allowlist: &BTreeSet<String>,
    ) -> Result<StartGate, PrinterError> {
        let mut errors = Vec::new();
        if req.confirm != CONFIRM_START {
            errors.push(format!(
                "confirm must be exactly \"{CONFIRM_START}\" (got {:?})",
                req.confirm
            ));
        }
        if req.printer_id != self.info.id {
            errors.push(format!(
                "printer_id mismatch: req={} adapter={}",
                req.printer_id, self.info.id
            ));
        }
        if !allowlist.contains(&req.printer_id) && !self.info.allowlisted {
            errors.push(format!(
                "printer '{}' not on allow-list (set allowlist or adapter.allowlisted)",
                req.printer_id
            ));
        }
        let path = Path::new(&req.gcode_path);
        let bytes =
            std::fs::read(path).map_err(|e| PrinterError::Msg(format!("read gcode: {e}")))?;
        let sha = hex_sha256(&bytes);
        if !sha.eq_ignore_ascii_case(&req.gcode_sha256) {
            errors.push(format!(
                "gcode hash mismatch: file={sha} req={}",
                req.gcode_sha256
            ));
        }
        let report = check_gcode(&String::from_utf8_lossy(&bytes), &PrinterVolume::default());
        if !report.ok {
            errors.push(format!("gcode-check failed: {:?}", report.errors));
        }
        // Never actually start in S11 alpha.
        if errors.is_empty() {
            errors.push(
                "start refused by design in S11 alpha (gates passed; live MQTT start not enabled)"
                    .into(),
            );
        }
        Ok(StartGate { ok: false, errors })
    }
}

pub fn hex_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// Evaluate start gates without a live adapter (unit-test helper).
pub fn evaluate_start_gates(
    req: &StartRequest,
    allowlist: &BTreeSet<String>,
    file_sha: &str,
    gcode_ok: bool,
) -> StartGate {
    let mut errors = Vec::new();
    if req.confirm != CONFIRM_START {
        errors.push("confirm must be START".into());
    }
    if !allowlist.contains(&req.printer_id) {
        errors.push("printer not allowlisted".into());
    }
    if !file_sha.eq_ignore_ascii_case(&req.gcode_sha256) {
        errors.push("hash mismatch".into());
    }
    if !gcode_ok {
        errors.push("gcode-check failed".into());
    }
    StartGate {
        ok: errors.is_empty(),
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn dry_run_hashes() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "G28\nG1 X1 Y1 Z0.2\n").unwrap();
        let p = BambuAdapter::new("bambu:x1c-01", "192.168.1.50", "X1C");
        let r = p.dry_run(f.path(), &PrinterVolume::default()).unwrap();
        assert_eq!(r.gcode_sha256.len(), 64);
        assert!(r.ok);
    }

    #[test]
    fn start_requires_confirm() {
        let mut allow = BTreeSet::new();
        allow.insert("bambu:x1c-01".into());
        let gate = evaluate_start_gates(
            &StartRequest {
                printer_id: "bambu:x1c-01".into(),
                gcode_path: "x.gcode".into(),
                gcode_sha256: "abc".into(),
                confirm: "yes".into(),
            },
            &allow,
            "abc",
            true,
        );
        assert!(!gate.ok);
        assert!(gate.errors.iter().any(|e| e.contains("confirm")));
    }

    #[test]
    fn start_gates_pass_when_complete() {
        let mut allow = BTreeSet::new();
        allow.insert("bambu:x1c-01".into());
        let gate = evaluate_start_gates(
            &StartRequest {
                printer_id: "bambu:x1c-01".into(),
                gcode_path: "x.gcode".into(),
                gcode_sha256: "abc".into(),
                confirm: CONFIRM_START.into(),
            },
            &allow,
            "abc",
            true,
        );
        assert!(gate.ok);
    }
}
