//! `cadre fab` + `cadre printer`

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use cadre_fab::{
    check_dfm, check_gcode, discover_slicers, face_to_dxf, hex_sha256, load_profile_json,
    plate_with_holes_dxf, sendcutsend_laser_v1, slice_command_preview, BambuAdapter, FacePick,
    FlatPart, Printer, PrinterVolume, StartRequest, CONFIRM_START,
};
use cadre_inspect::inspect_refs;
use serde_json::json;

use crate::cli::{
    Cli, FabArgs, FabCheckArgs, FabCmd, FabDxfArgs, FabDxfFaceArgs, FabGcodeCheckArgs,
    FabSliceArgs, PrinterArgs, PrinterCmd,
};
use crate::output::{emit, ExitCode};
use crate::topo_from_ir::topology_from_ir;

pub fn run_fab(cli: &Cli, args: &FabArgs) -> ExitCode {
    match &args.cmd {
        FabCmd::Dxf(a) => fab_dxf(cli, a),
        FabCmd::DxfFace(a) => fab_dxf_face(cli, a),
        FabCmd::Check(a) => fab_check(cli, a),
        FabCmd::Slicers => fab_slicers(cli),
        FabCmd::Slice(a) => fab_slice(cli, a),
        FabCmd::GcodeCheck(a) => fab_gcode_check(cli, a),
    }
}

pub fn run_printer(cli: &Cli, args: &PrinterArgs) -> ExitCode {
    match &args.cmd {
        PrinterCmd::Status(a) => printer_status(cli, a),
        PrinterCmd::DryRun(a) => printer_dry_run(cli, a),
        PrinterCmd::Start(a) => printer_start(cli, a),
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

fn fab_dxf_face(cli: &Cli, a: &FabDxfFaceArgs) -> ExitCode {
    use cadre_lang::{evaluate, EvalOptions};
    use std::fs;

    let source = match fs::read_to_string(&a.target) {
        Ok(s) => s,
        Err(e) => {
            emit(
                cli.json,
                &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-IO","message": e.to_string()}]}),
                false,
            );
            return ExitCode::Io;
        }
    };
    let name = a
        .target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("part.cad.star")
        .to_string();
    let mut opts = EvalOptions::new(name);
    // sets optional — reuse build_cmd parse if present
    if !a.set.is_empty() {
        match crate::build_cmd::parse_sets(&a.set) {
            Ok(m) => opts.overrides = m,
            Err(e) => {
                emit(
                    cli.json,
                    &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-USAGE","message": e}]}),
                    false,
                );
                return ExitCode::Usage;
            }
        }
    }
    let eval = evaluate(&source, &opts);
    if !eval.ok {
        emit(
            cli.json,
            &json!({"ok": false, "diagnostics": eval.diagnostics}),
            false,
        );
        return ExitCode::Eval;
    }
    let ir = eval.ir.expect("ir");

    // Prefer OCCT topology when requested; else IR analytic.
    let snap = match cli.kernel {
        crate::cli::KernelId::Occt => {
            #[cfg(feature = "occt")]
            {
                use crate::kernel_pick::open_kernel;
                use cadre_lang::execute_ir;
                match open_kernel(cli.kernel) {
                    Ok(mut kb) => match execute_ir(kb.as_mut(), &ir) {
                        Ok(sid) => match &kb {
                            crate::kernel_pick::KernelBox::Occt(k) => {
                                match k.topology_snapshot(sid) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        emit(
                                            cli.json,
                                            &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-TOPO","message": e.to_string()}]}),
                                            false,
                                        );
                                        return ExitCode::Kernel;
                                    }
                                }
                            }
                            _ => unreachable!(),
                        },
                        Err(e) => {
                            emit(
                                cli.json,
                                &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-KERNEL","message": e.to_string()}]}),
                                false,
                            );
                            return ExitCode::Kernel;
                        }
                    },
                    Err((c, v)) => {
                        emit(cli.json, &v, false);
                        return c;
                    }
                }
            }
            #[cfg(not(feature = "occt"))]
            {
                emit(
                    cli.json,
                    &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-KERNEL-UNAVAILABLE","message":"occt not in binary"}]}),
                    false,
                );
                return ExitCode::Usage;
            }
        }
        crate::cli::KernelId::Mock => match topology_from_ir(&ir) {
            Ok(s) => s,
            Err(e) => {
                emit(
                    cli.json,
                    &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-TOPO","message": e}]}),
                    false,
                );
                return ExitCode::Internal;
            }
        },
    };

    let report = inspect_refs(&snap, false);
    // Map selectors → (solid_idx, face_idx) by matching centroid/normal to snap faces
    let mut face_sels = Vec::new();
    for r in &report.refs {
        if r.kind != "face" {
            continue;
        }
        // find matching face in snap
        let mut found = None;
        for (si, solid) in snap.solids.iter().enumerate() {
            for (fi, face) in solid.faces.iter().enumerate() {
                let cdist = (face.centroid.x - r.centroid_mm.x).abs()
                    + (face.centroid.y - r.centroid_mm.y).abs()
                    + (face.centroid.z - r.centroid_mm.z).abs();
                if cdist > 1e-4 {
                    continue;
                }
                found = Some((si, fi));
                break;
            }
            if found.is_some() {
                break;
            }
        }
        if let Some((si, fi)) = found {
            face_sels.push((r.selector.clone(), si, fi));
        }
    }

    let pick = if let Some(f) = &a.face {
        FacePick::Selector(f.clone())
    } else if let Some(n) = &a.normal {
        match parse_vec3(n) {
            Some(v) => FacePick::Normal(v),
            None => {
                emit(
                    cli.json,
                    &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-USAGE","message":"bad --normal x,y,z"}]}),
                    false,
                );
                return ExitCode::Usage;
            }
        }
    } else {
        FacePick::Normal([0.0, 0.0, 1.0])
    };

    let result = match face_to_dxf(&snap, &face_sels, &pick, a.plane_tol, 0.15) {
        Ok(r) => r,
        Err(e) => {
            emit(
                cli.json,
                &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-DXF-FACE","message": e}]}),
                false,
            );
            return ExitCode::Validation;
        }
    };
    let out = a.out.clone().unwrap_or_else(|| PathBuf::from("face.dxf"));
    if let Err(e) = fs::write(&out, &result.dxf) {
        emit(
            cli.json,
            &json!({"ok": false, "diagnostics":[{"code":"CADRE-E-IO","message": e.to_string()}]}),
            false,
        );
        return ExitCode::Io;
    }
    emit(
        cli.json,
        &json!({
            "ok": true,
            "path": out,
            "bytes": result.dxf.len(),
            "face": result.face_selector,
            "normal": result.normal,
            "centroid_mm": result.centroid,
            "area_mm2": result.area_mm2,
            "edge_count": result.edge_count,
            "circle_count": result.circle_count,
            "kernel": match cli.kernel {
                crate::cli::KernelId::Mock => "mock",
                crate::cli::KernelId::Occt => "occt",
            },
        }),
        true,
    );
    ExitCode::Ok
}

fn parse_vec3(s: &str) -> Option<[f64; 3]> {
    let p: Vec<_> = s.split(',').collect();
    if p.len() != 3 {
        return None;
    }
    Some([p[0].parse().ok()?, p[1].parse().ok()?, p[2].parse().ok()?])
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

fn printer_status(cli: &Cli, a: &crate::cli::PrinterStatusArgs) -> ExitCode {
    let mut p = BambuAdapter::from_env(&a.id, &a.host, &a.model, a.serial.clone(), None);
    if let Some(s) = &a.serial {
        p = p.with_serial(s.clone());
    }
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

fn printer_dry_run(cli: &Cli, a: &crate::cli::PrinterDryRunArgs) -> ExitCode {
    let p = BambuAdapter::from_env(
        &a.id,
        &a.host,
        &a.model,
        a.serial.clone(),
        a.access_code.clone(),
    );
    match p.dry_run(&a.gcode, &PrinterVolume::default()) {
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

fn printer_start(cli: &Cli, a: &crate::cli::PrinterStartArgs) -> ExitCode {
    use cadre_fab::ExternalLiveTransport;
    use std::sync::Arc;

    let mut p = BambuAdapter::from_env(
        &a.id,
        &a.host,
        &a.model,
        a.serial.clone(),
        a.access_code.clone(),
    );
    if a.live {
        match ExternalLiveTransport::detect() {
            Ok(t) => p = p.with_transport(Arc::new(t)),
            Err(e) => {
                emit(
                    cli.json,
                    &json!({
                        "ok": false,
                        "error": e.to_string(),
                        "hint": "live start needs curl + mosquitto_pub on PATH (or CADRE_CURL / CADRE_MOSQUITTO_PUB)"
                    }),
                    false,
                );
                return ExitCode::Usage;
            }
        }
    }

    let mut allow = BTreeSet::new();
    if let Some(list) = &a.allowlist {
        for part in list.split(',') {
            let t = part.trim();
            if !t.is_empty() {
                allow.insert(t.to_string());
            }
        }
    }
    let req = StartRequest {
        printer_id: a.id.clone(),
        gcode_path: a.gcode.display().to_string(),
        gcode_sha256: a.sha256.clone(),
        confirm: a.confirm.clone().unwrap_or_default(),
        live: a.live,
        remote_name: a.remote_name.clone(),
    };
    match p.start(&req, &allow) {
        Ok(gate) => {
            emit(
                cli.json,
                &json!({
                    "ok": gate.ok,
                    "gate": gate,
                    "required_confirm": CONFIRM_START,
                    "live_requested": a.live,
                    "note": if a.live {
                        "live path: FTPS upload + MQTT gcode_file after gates"
                    } else {
                        "default is safe: gates only; pass --live to contact printer"
                    }
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
