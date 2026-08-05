//! `cadre serve`

use std::path::PathBuf;

use serde_json::json;

use crate::cli::{Cli, ServeArgs, ServeCmd};
use crate::output::{emit, ExitCode};

pub fn run(cli: &Cli, args: &ServeArgs) -> ExitCode {
    match &args.cmd {
        ServeCmd::Api(a) => serve_api(
            cli,
            a.port,
            a.host.clone(),
            a.token.clone(),
            a.project.clone(),
        ),
    }
}

fn serve_api(
    cli: &Cli,
    port: u16,
    host: String,
    token: Option<String>,
    project: Option<PathBuf>,
) -> ExitCode {
    let bind = format!("{host}:{port}");
    let project_root = project
        .or_else(|| cli.project.clone())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let cfg = cadre_api::AppConfig {
        bind: bind.clone(),
        token: token.clone(),
        project_root: project_root.clone(),
    };

    if cli.json {
        // emit once then block serving
        let body = json!({
            "ok": true,
            "bind": bind,
            "project_root": project_root,
            "auth": token.is_some(),
            "openapi": format!("http://{bind}/v1/openapi.json"),
        });
        emit(true, &body, true);
    } else if !cli.quiet {
        eprintln!("cadre serve api on http://{bind}");
        eprintln!("  openapi: http://{bind}/v1/openapi.json");
        if token.is_some() {
            eprintln!("  auth: bearer token required");
        }
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("tokio runtime: {e}");
            return ExitCode::Internal;
        }
    };
    match rt.block_on(cadre_api::serve(cfg)) {
        Ok(()) => ExitCode::Ok,
        Err(e) => {
            eprintln!("serve error: {e}");
            ExitCode::Io
        }
    }
}
