//! `cadre view` — tiny loopback HTTP viewer for snapshot packets.

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;

use crate::cli::{Cli, ViewArgs};
use crate::output::{emit, ExitCode};

pub fn run(cli: &Cli, args: &ViewArgs) -> ExitCode {
    if args.paths.is_empty() {
        let v = json!({"ok": false, "diagnostics": [{"code": "CADRE-E-USAGE", "message": "pass at least one path"}]});
        emit(cli.json, &v, false);
        return ExitCode::Usage;
    }

    let mut entries = Vec::new();
    for p in &args.paths {
        match prepare_entry(p) {
            Ok(e) => entries.push(e),
            Err(msg) => {
                let v =
                    json!({"ok": false, "diagnostics": [{"code": "CADRE-E-VIEW", "message": msg}]});
                emit(cli.json, &v, false);
                return ExitCode::Io;
            }
        }
    }

    if args.once {
        // Prepare artifacts only — no long-lived server (CI-friendly).
        let links: Vec<_> = entries
            .iter()
            .map(|e| {
                json!({
                    "path": e.path,
                    "root": e.root,
                    "kind": e.kind,
                    "url": null,
                    "note": "prepared; run without --once to serve",
                })
            })
            .collect();
        let body = json!({
            "ok": true,
            "once": true,
            "links": links,
        });
        emit(cli.json, &body, true);
        return ExitCode::Ok;
    }

    let listener = match bind_viewer(&args.host, args.port) {
        Ok(l) => l,
        Err(e) => {
            let v = json!({"ok": false, "diagnostics": [{"code": "CADRE-E-IO", "message": e}]});
            emit(cli.json, &v, false);
            return ExitCode::Io;
        }
    };
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{}:{}", addr.ip(), addr.port());

    let links: Vec<_> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            json!({
                "path": e.path,
                "url": format!("{base}/v/{i}/"),
                "kind": e.kind,
            })
        })
        .collect();

    let body = json!({
        "ok": true,
        "base": base,
        "links": links,
        "note": "viewer alpha — open links in a browser; Ctrl-C to stop",
    });
    emit(cli.json, &body, true);
    if !cli.json && !cli.quiet {
        eprintln!("cadre view listening on {base}");
        for l in &links {
            if let Some(u) = l.get("url").and_then(|u| u.as_str()) {
                eprintln!("  {u}");
            }
        }
    }

    let entries = std::sync::Arc::new(entries);
    let base_c = base.clone();
    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let ents = entries.clone();
                let base = base_c.clone();
                thread::spawn(move || {
                    let _ = handle_client(&mut stream, &ents, &base);
                });
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

fn bind_viewer(host: &str, port: u16) -> Result<TcpListener, String> {
    let bind = format!("{host}:{port}");
    match TcpListener::bind(&bind) {
        Ok(l) => Ok(l),
        Err(e) => {
            for d in 1..30 {
                let b = format!("{host}:{}", port + d);
                if let Ok(l) = TcpListener::bind(&b) {
                    return Ok(l);
                }
            }
            Err(format!("bind {bind}: {e}"))
        }
    }
}

struct Entry {
    path: PathBuf,
    kind: &'static str,
    root: PathBuf,
}

fn prepare_entry(p: &Path) -> Result<Entry, String> {
    let p = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    if p.is_dir() {
        return Ok(Entry {
            path: p.clone(),
            kind: "snap",
            root: p,
        });
    }
    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if name.ends_with(".cad.star") || name.ends_with(".star") {
        let stem = name
            .strip_suffix(".cad.star")
            .or_else(|| name.strip_suffix(".star"))
            .unwrap_or(name);
        let snap = p.with_file_name(format!("{stem}.snap"));
        let src = fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let eval = cadre_lang::evaluate(&src, &cadre_lang::EvalOptions::new(name));
        if !eval.ok {
            return Err(format!("eval failed: {:?}", eval.diagnostics));
        }
        let (mesh, notes) = cadre_render::mesh_from_ir(eval.ir.as_ref().unwrap())?;
        let opts = cadre_render::SnapshotOptions {
            notes,
            width: 384,
            height: 384,
            gif_frames: 16,
            ..Default::default()
        };
        cadre_render::write_snapshot_packet(&mesh, &snap, &opts)?;
        return Ok(Entry {
            path: p,
            kind: "star",
            root: snap,
        });
    }
    if name.ends_with(".png") || name.ends_with(".gif") {
        let root = p.parent().unwrap_or(Path::new(".")).to_path_buf();
        return Ok(Entry {
            path: p,
            kind: "image",
            root,
        });
    }
    Err(format!("unsupported view target: {}", p.display()))
}

fn handle_client(stream: &mut TcpStream, entries: &[Entry], base: &str) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf)?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let line = req.lines().next().unwrap_or("");
    let path = line.split_whitespace().nth(1).unwrap_or("/");

    if path == "/" || path == "/index.html" {
        let html = index_html(entries, base);
        return respond(stream, "text/html; charset=utf-8", html.as_bytes());
    }

    if let Some(rest) = path.strip_prefix("/v/") {
        let mut parts = rest.splitn(2, '/');
        let idx: usize = parts.next().unwrap_or("0").parse().unwrap_or(9999);
        let file = parts.next().unwrap_or("");
        if idx >= entries.len() {
            return respond(stream, "text/plain", b"not found");
        }
        let ent = &entries[idx];
        if file.is_empty() {
            let html = packet_html(ent, idx, base);
            return respond(stream, "text/html; charset=utf-8", html.as_bytes());
        }
        let file = file.trim_start_matches('/');
        if file.contains("..") || file.contains('/') || file.contains('\\') {
            return respond(stream, "text/plain", b"bad path");
        }
        let fp = ent.root.join(file);
        if fp.is_file() {
            let data = fs::read(&fp)?;
            return respond(stream, content_type(&fp), &data);
        }
    }

    respond(stream, "text/plain", b"not found")
}

fn content_type(p: &Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

fn respond(stream: &mut TcpStream, ct: &str, body: &[u8]) -> std::io::Result<()> {
    let hdr = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(hdr.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}

fn index_html(entries: &[Entry], base: &str) -> String {
    let mut items = String::new();
    for (i, e) in entries.iter().enumerate() {
        items.push_str(&format!(
            "<li><a href=\"{base}/v/{i}/\">{path}</a> <small>{kind}</small></li>",
            base = base,
            i = i,
            path = e.path.display(),
            kind = e.kind
        ));
    }
    format!(
        r#"<!doctype html>
<html><head><meta charset="utf-8"><title>Cadre View</title>
<style>
body{{font-family:system-ui,sans-serif;background:#12141a;color:#e8eaed;margin:2rem}}
a{{color:#8ab4f8}} li{{margin:.5rem 0}}
</style></head><body>
<h1>Cadre Viewer <small style="opacity:.6">alpha</small></h1>
<ul>{items}</ul>
</body></html>"#
    )
}

fn packet_html(ent: &Entry, idx: usize, base: &str) -> String {
    let mut imgs = String::new();
    if let Ok(rd) = fs::read_dir(&ent.root) {
        let mut files: Vec<_> = rd.filter_map(|e| e.ok()).collect();
        files.sort_by_key(|e| e.file_name());
        for f in files {
            let name = f.file_name().to_string_lossy().into_owned();
            if name.ends_with(".png") || name.ends_with(".gif") {
                imgs.push_str(&format!(
                    "<figure><img src=\"{base}/v/{idx}/{name}\" alt=\"{name}\"><figcaption>{name}</figcaption></figure>",
                    base = base,
                    idx = idx,
                    name = name
                ));
            }
        }
    }
    format!(
        r#"<!doctype html>
<html><head><meta charset="utf-8"><title>{title}</title>
<style>
body{{font-family:system-ui,sans-serif;background:#12141a;color:#e8eaed;margin:1.5rem}}
a{{color:#8ab4f8}}
.grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:1rem}}
img{{max-width:100%;background:#1a1d24;border-radius:8px}}
figcaption{{opacity:.7;font-size:.85rem;margin-top:.35rem}}
</style></head><body>
<p><a href="{base}/">← all</a></p>
<h1>{title}</h1>
<p style="opacity:.7">{kind} · {root}</p>
<div class="grid">{imgs}</div>
</body></html>"#,
        title = ent.path.display(),
        base = base,
        kind = ent.kind,
        root = ent.root.display(),
        imgs = imgs
    )
}

// silence unused Instant warning if any
#[allow(dead_code)]
fn _now() -> Instant {
    Instant::now()
}
