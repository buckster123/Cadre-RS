//! `cadre mcp` + `cadre skills`

use std::fs;
use std::path::PathBuf;

use serde_json::json;

use crate::cli::{Cli, McpArgs, SkillsArgs, SkillsCmd};
use crate::output::{emit, ExitCode};

pub fn run_mcp(_cli: &Cli, _args: &McpArgs) -> ExitCode {
    // stdout is JSON-RPC only — do not emit human/json wrapper
    match cadre_mcp::run_stdio() {
        Ok(()) => ExitCode::Ok,
        Err(e) => {
            eprintln!("cadre mcp error: {e}");
            ExitCode::Internal
        }
    }
}

pub fn run_skills(cli: &Cli, args: &SkillsArgs) -> ExitCode {
    match &args.cmd {
        SkillsCmd::Export(a) => export_skills(cli, a.out.clone(), &a.agent),
    }
}

fn export_skills(cli: &Cli, out: Option<PathBuf>, agent: &str) -> ExitCode {
    let dest = out.unwrap_or_else(|| PathBuf::from("dist/skills/cadre"));
    let src = skill_source_dir();
    if !src.join("SKILL.md").is_file() {
        let v = json!({
            "ok": false,
            "diagnostics": [{
                "code": "CADRE-E-SKILLS",
                "message": format!("bundled skill missing at {}", src.display()),
            }]
        });
        emit(cli.json, &v, false);
        return ExitCode::Io;
    }
    if let Err(e) = copy_dir(&src, &dest) {
        let v = json!({"ok": false, "diagnostics": [{"code": "CADRE-E-IO", "message": e}]});
        emit(cli.json, &v, false);
        return ExitCode::Io;
    }
    // agent-specific note file
    let note = format!(
        "# Install notes ({agent})\n\nCopy this folder into your agent's skills directory.\n\n- Claude Code: project `.claude/skills/cadre/` or user skills path\n- Codex: plugin/skills path per current Codex docs\n- Hermes: `~/.hermes/skills/cadre/` or profile skills\n\nThen restart / reload skills. Prefer MCP `cadre mcp` for tool calls.\n"
    );
    let _ = fs::write(dest.join("INSTALL.md"), note);

    let body = json!({
        "ok": true,
        "agent": agent,
        "out": dest,
        "files": list_files(&dest),
    });
    emit(cli.json, &body, true);
    ExitCode::Ok
}

fn skill_source_dir() -> PathBuf {
    // repo layout: skills/cadre next to crates/
    let from_cli = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../skills/cadre");
    if from_cli.join("SKILL.md").is_file() {
        return from_cli.canonicalize().unwrap_or(from_cli);
    }
    PathBuf::from("skills/cadre")
}

fn copy_dir(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for ent in fs::read_dir(src).map_err(|e| e.to_string())? {
        let ent = ent.map_err(|e| e.to_string())?;
        let ty = ent.file_type().map_err(|e| e.to_string())?;
        let to = dst.join(ent.file_name());
        if ty.is_dir() {
            copy_dir(&ent.path(), &to)?;
        } else {
            fs::copy(ent.path(), to).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn list_files(dir: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(d: &std::path::Path, acc: &mut Vec<String>) {
        if let Ok(rd) = fs::read_dir(d) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, acc);
                } else {
                    acc.push(p.display().to_string());
                }
            }
        }
    }
    walk(dir, &mut out);
    out.sort();
    out
}
