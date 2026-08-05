//! `cadre fab` + `cadre printer`

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use cadre_fab::{
    check_dfm, check_gcode, discover_slicers, hex_sha256, load_profile_json, plate_with_holes_dxf,
    sendcutsend_laser_v1, slice_command_preview, BambuAdapter, FlatPart, Printer, PrinterVolume,
    StartRequest, CONFIRM_START,
};
use serde_json::json;

use crate::cli::{
    Cli, FabArgs, FabCheckArgs, FabCmd, FabDxfArgs, FabGcodeCheckArgs, FabSliceArgs, PrinterArgs,
    PrinterCmd,
};
use crate::output::{emit, ExitCode};

pub fn run_fab(cli: &Cli, args: &FabArgs) -> ExitCode {
    match &args.cmd {
        FabCmd::Dxf(a) => fab_dxf(cli, a),
        FabCmd::Check(a) => fab_check(cli, a),
        FabCmd::Slicers => fab_slicers(cli),
        FabCmd::Slice(a) => fab_slice(cli, a),
        FabCmd::GcodeCheck(a) => fab_gcode_check(cli, a),
    }
}

pub fn run_printer(cli: &Cli, args: &PrinterArgs) -> ExitCode {
    match &args.cmd {
        PrinterCmd::Status(a) => printer_status(cli, &a.id, &a.host, &a.model),
        PrinterCmd::DryRun(a) => printer_dry_run(cli, &a.id, &a.host, &a.model, &a.gcode),
        PrinterCmd::Start(a) => printer_start(
            cli,
            &a.id,
            &a.host,
            &a.gcode,
            &a.sha256,
            a.confirm.as_deref(),
            a.allowlist.as_deref(),
        ),
    }
}

fn fab_dxf(cli: &Cli, a: &FabDxfArgs) -> ExitCode {
    let holes: Vec<(f64, f64, f64)> = a.hole.iter().filter_map(|s| parse_hole(s)).collect();
    let dxf = plate_with_holes_dxf(a.width, a.height, &holes);
    let out = a.out.clone().unwrap_or_else(|| PathBuf::from("part.dxf"));
    if let Err(e) = fs::write(&out, &dxf) {
        emit(
            cli.json,
            &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-IO","message": e.to_string()}]}),
            false,
        );
        return ExitCode::Io;
    }
    emit(
        cli.json,
        &json!({"ok": true, "path": out, "bytes": dxf.len(), "holes": holes.len()}),
        true,
    );
    ExitCode::Ok
}

fn parse_hole(s: &str) -> Option<(f64, f64, f64)> {
    let parts: Vec<_> = s.split(',').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

fn fab_check(cli: &Cli, a: &FabCheckArgs) -> ExitCode {
    let profile = if let Some(p) = &a.profile_file {
        match fs::read_to_string(p).and_then(|t| {
            load_profile_json(&t)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }) {
            Ok(p) => p,
            Err(e) => {
                emit(
                    cli.json,
                    &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-IO","message": e.to_string()}]}),
                    false,
                );
                return ExitCode::Io;
            }
        }
    } else {
        match a.profile.as_str() {
            "sendcutsend.laser" | "sendcutsend.laser@1" | "scs" => sendcutsend_laser_v1(),
            other => {
                emit(
                    cli.json,
                    &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-USAGE","message": format!("unknown profile '{other}' (try sendcutsend.laser or --profile-file)")}]}),
                    false,
                );
                return ExitCode::Usage;
            }
        }
    };

    let part = if let Some(path) = &a.part_json {
        match fs::read_to_string(path).and_then(|t| {
            serde_json::from_str::<FlatPart>(&t)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }) {
            Ok(p) => p,
            Err(e) => {
                emit(
                    cli.json,
                    &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-IO","message": e.to_string()}]}),
                    false,
                );
                return ExitCode::Io;
            }
        }
    } else {
        FlatPart {
            width_mm: a.width.unwrap_or(100.0),
            height_mm: a.height.unwrap_or(50.0),
            thickness_mm: a.thickness.unwrap_or(3.0),
            material: a.material.clone().unwrap_or_else(|| "Aluminum 5052".into()),
            holes_dia_mm: a.hole_dia.clone(),
            min_hole_edge_mm: a.min_edge,
            min_hole_spacing_mm: a.min_spacing,
        }
    };

    let report = check_dfm(&profile, &part);
    let ok = report.ok;
    emit(
        cli.json,
        &json!({"ok": ok, "report": report, "part": part}),
        ok,
    );
    if ok {
        ExitCode::Ok
    } else {
        ExitCode::Validation
    }
}

fn fab_slicers(cli: &Cli) -> ExitCode {
    let found = discover_slicers();
    emit(
        cli.json,
        &json!({"ok": true, "slicers": found, "count": found.len()}),
        true,
    );
    ExitCode::Ok
}

fn fab_slice(cli: &Cli, a: &FabSliceArgs) -> ExitCode {
    let slicers = discover_slicers();
    let slicer = if let Some(name) = &a.slicer {
        slicers.iter().find(|s| {
            s.name.eq_ignore_ascii_case(name)
                || s.path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .is_some_and(|f| f.eq_ignore_ascii_case(name))
        })
    } else {
        slicers.first()
    };

    let Some(slicer) = slicer else {
        emit(
            cli.json,
            &json!({
                "ok": false,
                "diagnostics":[{"code":"CADRE-E-FAB","message":"no slicer found on PATH; install PrusaSlicer/OrcaSlicer or pass after discovery"}],
                "slicers": slicers,
            }),
            false,
        );
        return ExitCode::Io;
    };

    let out = a
        .out
        .clone()
        .unwrap_or_else(|| a.mesh.with_extension("gcode"));
    let cmd = slice_command_preview(slicer, &a.mesh, &out, a.profile.as_deref());

    if a.execute {
        // Real execution is optional and host-dependent; still refuse silently failing.
        emit(
            cli.json,
            &json!({
                "ok": false,
                "diagnostics":[{"code":"CADRE-E-FAB","message":"execute mode not enabled in S11 alpha (preview only); run the command manually"}],
                "command": cmd,
                "slicer": slicer,
            }),
            false,
        );
        return ExitCode::Usage;
    }

    emit(
        cli.json,
        &json!({
            "ok": true,
            "preview": true,
            "command": cmd,
            "slicer": slicer,
            "mesh": a.mesh,
            "out": out,
            "note": "Cadre orchestrates real slicers; S11 prints the command only unless --execute is later enabled."
        }),
        true,
    );
    ExitCode::Ok
}

fn fab_gcode_check(cli: &Cli, a: &FabGcodeCheckArgs) -> ExitCode {
    let text = match fs::read_to_string(&a.gcode) {
        Ok(t) => t,
        Err(e) => {
            emit(
                cli.json,
                &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-IO","message": e.to_string()}]}),
                false,
            );
            return ExitCode::Io;
        }
    };
    let vol = PrinterVolume {
        x_mm: a.bed_x.unwrap_or(256.0),
        y_mm: a.bed_y.unwrap_or(256.0),
        z_mm: a.bed_z.unwrap_or(256.0),
        max_hotend_c: a.max_hotend.unwrap_or(300.0),
        max_bed_c: a.max_bed.unwrap_or(110.0),
    };
    let report = check_gcode(&text, &vol);
    let sha = hex_sha256(text.as_bytes());
    let ok = report.ok;
    emit(
        cli.json,
        &json!({"ok": ok, "report": report, "sha256": sha, "path": a.gcode}),
        ok,
    );
    if ok {
        ExitCode::Ok
    } else {
        ExitCode::Validation
    }
}

fn printer_status(cli: &Cli, id: &str, host: &str, model: &str) -> ExitCode {
    let p = BambuAdapter::new(id, host, model);
    match p.status() {
        Ok(v) => {
            emit(cli.json, &v, true);
            ExitCode::Ok
        }
        Err(e) => {
            emit(
                cli.json,
                &json!({"ok": false, "error": e.to_string()}),
                false,
            );
            ExitCode::Network
        }
    }
}

fn printer_dry_run(cli: &Cli, id: &str, host: &str, model: &str, gcode: &Path) -> ExitCode {
    let p = BambuAdapter::new(id, host, model);
    match p.dry_run(gcode, &PrinterVolume::default()) {
        Ok(r) => {
            let ok = r.ok;
            emit(cli.json, &json!({"ok": ok, "dry_run": r}), ok);
            if ok {
                ExitCode::Ok
            } else {
                ExitCode::Validation
            }
        }
        Err(e) => {
            emit(
                cli.json,
                &json!({"ok": false, "error": e.to_string()}),
                false,
            );
            ExitCode::Io
        }
    }
}

fn printer_start(
    cli: &Cli,
    id: &str,
    host: &str,
    gcode: &Path,
    sha256: &str,
    confirm: Option<&str>,
    allowlist: Option<&str>,
) -> ExitCode {
    let p = BambuAdapter::new(id, host, "X1C").with_allowlisted(false);
    let mut allow = BTreeSet::new();
    if let Some(list) = allowlist {
        for part in list.split(',') {
            let t = part.trim();
            if !t.is_empty() {
                allow.insert(t.to_string());
            }
        }
    }
    let req = StartRequest {
        printer_id: id.into(),
        gcode_path: gcode.display().to_string(),
        gcode_sha256: sha256.into(),
        confirm: confirm.unwrap_or("").into(),
    };
    match p.start(&req, &allow) {
        Ok(gate) => {
            emit(
                cli.json,
                &json!({
                    "ok": gate.ok,
                    "gate": gate,
                    "required_confirm": CONFIRM_START,
                    "note": "S11 never starts a real print; gates must still pass."
                }),
                gate.ok,
            );
            if gate.ok {
                ExitCode::Ok
            } else {
                ExitCode::Safety
            }
        }
        Err(e) => {
            emit(
                cli.json,
                &json!({"ok": false, "error": e.to_string()}),
                false,
            );
            ExitCode::Safety
        }
    }
}
