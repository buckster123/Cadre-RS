//! `cadre inspect`

use std::fs;

use cadre_inspect::{inspect_refs, measure, MeasureKind, MeasureRequest};
use cadre_lang::{evaluate, EvalOptions};
use serde_json::json;

use crate::build_cmd::parse_sets;
use crate::cli::Cli;
use crate::cli::{InspectArgs, InspectCmd, MeasureKindArg};
use crate::output::{emit, ExitCode};
use crate::topo_from_ir::topology_from_ir;

pub fn run(cli: &Cli, args: &InspectArgs) -> ExitCode {
    match &args.cmd {
        InspectCmd::Refs(a) => refs(cli, a.target.clone(), a.facts, &a.set),
        InspectCmd::Measure(a) => measure_cmd(
            cli,
            a.target.clone(),
            a.a.clone(),
            a.b.clone(),
            a.kind,
            &a.set,
        ),
    }
}

fn load_ir(
    target: &std::path::Path,
    sets: &[String],
) -> Result<cadre_lang::FeatureIr, (ExitCode, serde_json::Value)> {
    if target
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e == "json")
    {
        let text = fs::read_to_string(target).map_err(|e| {
            (
                ExitCode::Io,
                json!({"ok": false, "diagnostics": [{"code": "CADRE-E-IO", "message": e.to_string()}]}),
            )
        })?;
        let ir: cadre_lang::FeatureIr = serde_json::from_str(&text).map_err(|e| {
            (
                ExitCode::Eval,
                json!({"ok": false, "diagnostics": [{"code": "CADRE-E-EVAL", "message": format!("bad IR json: {e}")}]}),
            )
        })?;
        return Ok(ir);
    }

    let source = fs::read_to_string(target).map_err(|e| {
        (
            ExitCode::Io,
            json!({"ok": false, "diagnostics": [{"code": "CADRE-E-IO", "message": e.to_string()}]}),
        )
    })?;
    let overrides = parse_sets(sets).map_err(|m| {
        (
            ExitCode::Usage,
            json!({"ok": false, "diagnostics": [{"code": "CADRE-E-USAGE", "message": m}]}),
        )
    })?;
    let name = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("model.cad.star")
        .to_string();
    let mut opts = EvalOptions::new(name);
    opts.overrides = overrides;
    let eval = evaluate(&source, &opts);
    if !eval.ok {
        return Err((
            ExitCode::Eval,
            json!({"ok": false, "diagnostics": eval.diagnostics}),
        ));
    }
    Ok(eval.ir.expect("ir"))
}

fn refs(cli: &Cli, target: std::path::PathBuf, facts: bool, sets: &[String]) -> ExitCode {
    let ir = match load_ir(&target, sets) {
        Ok(ir) => ir,
        Err((c, v)) => {
            emit(cli.json, &v, false);
            return c;
        }
    };
    let snap = match topology_from_ir(&ir) {
        Ok(s) => s,
        Err(m) => {
            let v = json!({"ok": false, "diagnostics": [{"code": "CADRE-E-TOPO", "message": m}]});
            emit(cli.json, &v, false);
            return ExitCode::Internal;
        }
    };
    let report = inspect_refs(&snap, facts);
    let body = json!({
        "ok": true,
        "object": report.object,
        "solids": report.solids,
        "faces": report.faces,
        "edges": report.edges,
        "refs": report.refs,
        "facts": report.facts,
        "meta": {"source": target, "note": "topology from IR analytic approx (booleans/fillets coarse)"},
    });
    emit(cli.json, &body, true);
    ExitCode::Ok
}

fn measure_cmd(
    cli: &Cli,
    target: std::path::PathBuf,
    a: String,
    b: Option<String>,
    kind: MeasureKindArg,
    sets: &[String],
) -> ExitCode {
    let ir = match load_ir(&target, sets) {
        Ok(ir) => ir,
        Err((c, v)) => {
            emit(cli.json, &v, false);
            return c;
        }
    };
    let snap = match topology_from_ir(&ir) {
        Ok(s) => s,
        Err(m) => {
            let v = json!({"ok": false, "diagnostics": [{"code": "CADRE-E-TOPO", "message": m}]});
            emit(cli.json, &v, false);
            return ExitCode::Internal;
        }
    };
    let kind = match kind {
        MeasureKindArg::Distance => MeasureKind::Distance,
        MeasureKindArg::Angle => MeasureKind::Angle,
        MeasureKindArg::Diameter => MeasureKind::Diameter,
        MeasureKindArg::Thickness => MeasureKind::Thickness,
    };
    match measure(&snap, &MeasureRequest { a, b, kind }) {
        Ok(r) => {
            let body = json!({
                "ok": true,
                "kind": r.kind,
                "value": r.value,
                "unit": r.unit,
                "construction": r.construction,
                "a": r.a,
                "b": r.b,
            });
            emit(cli.json, &body, true);
            ExitCode::Ok
        }
        Err(e) => {
            let v = json!({
                "ok": false,
                "diagnostics": [{
                    "code": "CADRE-E-MEASURE",
                    "severity": "error",
                    "message": e.to_string(),
                }]
            });
            emit(cli.json, &v, false);
            ExitCode::Validation
        }
    }
}
