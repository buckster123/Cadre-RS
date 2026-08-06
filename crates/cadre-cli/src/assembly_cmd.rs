//! `cadre assembly`

use std::fs;
use std::path::Path;

use cadre_parts::{validate_assembly, AssemblySpec};
use serde_json::json;

use crate::cli::{AssemblyArgs, AssemblyCmd, Cli};
use crate::output::{emit, ExitCode};

pub fn run(cli: &Cli, args: &AssemblyArgs) -> ExitCode {
    match &args.cmd {
        AssemblyCmd::Validate(a) => validate(cli, &a.target),
    }
}

fn validate(cli: &Cli, path: &Path) -> ExitCode {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            emit(
                cli.json,
                &json!({"ok": false, "error": format!("read {}: {e}", path.display())}),
                false,
            );
            return ExitCode::Io;
        }
    };
    let spec: AssemblySpec = match serde_json::from_str(&text) {
        Ok(s) => s,
        Err(e) => {
            emit(
                cli.json,
                &json!({"ok": false, "error": format!("parse assembly: {e}")}),
                false,
            );
            return ExitCode::Validation;
        }
    };
    let report = validate_assembly(&spec);
    emit(
        cli.json,
        &json!({
            "ok": report.ok,
            "path": path,
            "report": report,
        }),
        report.ok,
    );
    if report.ok {
        ExitCode::Ok
    } else {
        ExitCode::Validation
    }
}
