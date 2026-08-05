//! `cadre view` — loopback HTTP viewer: snaps + G-code scrub + URDF jog (alpha).

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

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
        let links: Vec<_> = entries
            .iter()
            .map(|e| {
                json!({
                    "path": e.path,
                    "root": e.root,
                    "kind": e.kind,
                    "meta": e.meta_name,
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
                "meta": e.meta_name,
            })
        })
        .collect();

    let body = json!({
        "ok": true,
        "base": base,
        "links": links,
        "note": "viewer alpha — snaps / gcode scrub / urdf jog; Ctrl-C to stop",
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
    /// Directory served under /v/i/…
    root: PathBuf,
    /// Optional primary meta file name (path.json / robot.json).
    meta_name: Option<String>,
}

fn prepare_entry(p: &Path) -> Result<Entry, String> {
    let p = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    if p.is_dir() {
        return Ok(Entry {
            path: p.clone(),
            kind: "snap",
            root: p,
            meta_name: None,
        });
    }
    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let lower = name.to_ascii_lowercase();

    if lower.ends_with(".cad.star") || lower.ends_with(".star") {
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
            meta_name: None,
        });
    }

    if lower.ends_with(".gcode") || lower.ends_with(".gco") || lower.ends_with(".nc") {
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("gcode");
        let root = p.with_file_name(format!("{stem}.view"));
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let text = fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let path = cadre_fab::extract_gcode_path(&text);
        let meta = root.join("path.json");
        fs::write(
            &meta,
            serde_json::to_string_pretty(&path).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        // Keep a tiny copy of source head for display
        let head: String = text.lines().take(40).collect::<Vec<_>>().join("\n");
        fs::write(root.join("preview.txt"), head).ok();
        return Ok(Entry {
            path: p,
            kind: "gcode",
            root,
            meta_name: Some("path.json".into()),
        });
    }

    if lower.ends_with(".robot.json") || lower.ends_with(".urdf") || lower.ends_with(".json") {
        let robot = load_robot(&p)?;
        let payload = cadre_robot::jog_payload(&robot);
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("robot");
        let root = p.with_file_name(format!("{stem}.view"));
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let meta = root.join("robot.json");
        fs::write(
            &meta,
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        return Ok(Entry {
            path: p,
            kind: "robot",
            root,
            meta_name: Some("robot.json".into()),
        });
    }

    if lower.ends_with(".png") || lower.ends_with(".gif") {
        let root = p.parent().unwrap_or(Path::new(".")).to_path_buf();
        return Ok(Entry {
            path: p,
            kind: "image",
            root,
            meta_name: None,
        });
    }

    Err(format!(
        "unsupported view target: {} (want .snap / .cad.star / .gcode / .robot.json / .urdf)",
        p.display()
    ))
}

fn load_robot(p: &Path) -> Result<cadre_robot::RobotSpec, String> {
    let text = fs::read_to_string(p).map_err(|e| e.to_string())?;
    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if name.ends_with(".urdf") {
        // Alpha: jog payload needs RobotSpec JSON. Validate URDF then refuse with hint.
        cadre_robot::parse_urdf_xml(&text).map_err(|e| e.to_string())?;
        return Err(
            "URDF jog alpha expects Cadre .robot.json (export via `cadre robot emit`). URDF validated OK."
                .into(),
        );
    }
    serde_json::from_str(&text).map_err(|e| format!("robot json: {e}"))
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
            let html = match ent.kind {
                "gcode" => gcode_html(ent, idx, base),
                "robot" => robot_html(ent, idx, base),
                _ => packet_html(ent, idx, base),
            };
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
        Some("txt") => "text/plain; charset=utf-8",
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
<p style="opacity:.7">snaps · gcode scrub · urdf jog</p>
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

fn gcode_html(ent: &Entry, idx: usize, base: &str) -> String {
    let meta = ent.meta_name.as_deref().unwrap_or("path.json");
    format!(
        r#"<!doctype html>
<html><head><meta charset="utf-8"><title>G-code scrub · {title}</title>
<style>
body{{font-family:system-ui,sans-serif;background:#12141a;color:#e8eaed;margin:1.5rem}}
a{{color:#8ab4f8}}
#c{{background:#1a1d24;border-radius:8px;width:min(720px,100%);height:480px;display:block}}
.row{{display:flex;gap:1rem;flex-wrap:wrap;align-items:center;margin:1rem 0}}
input[type=range]{{width:min(420px,70vw)}}
.meta{{opacity:.75;font-size:.9rem}}
.travel{{stroke:#5f6368}} .extrude{{stroke:#8ab4f8}}
</style></head><body>
<p><a href="{base}/">← all</a></p>
<h1>G-code layer scrub</h1>
<p class="meta">{title} · <a href="{base}/v/{idx}/{meta}">{meta}</a></p>
<canvas id="c" width="720" height="480"></canvas>
<div class="row">
  <label>layer <span id="li">0</span> / <span id="ln">0</span> · Z=<span id="lz">0</span></label>
  <input id="slider" type="range" min="0" max="0" value="0">
</div>
<p class="meta">Blue = extrude · grey = travel · scrub layers by Z (±0.05 mm bucket)</p>
<script>
const metaUrl = "{base}/v/{idx}/{meta}";
const canvas = document.getElementById('c');
const ctx = canvas.getContext('2d');
let data = null;
async function main() {{
  data = await (await fetch(metaUrl)).json();
  const n = Math.max(0, (data.layers||[]).length - 1);
  document.getElementById('ln').textContent = n;
  const sl = document.getElementById('slider');
  sl.max = n;
  sl.oninput = () => draw(+sl.value);
  draw(0);
}}
function draw(li) {{
  const L = data.layers[li];
  if (!L) return;
  document.getElementById('li').textContent = li;
  document.getElementById('lz').textContent = L.z.toFixed(3);
  const pts = data.points.slice(L.start, L.end);
  ctx.clearRect(0,0,canvas.width,canvas.height);
  if (pts.length < 2) return;
  let xmin=Infinity,xmax=-Infinity,ymin=Infinity,ymax=-Infinity;
  for (const p of pts) {{ xmin=Math.min(xmin,p.x); xmax=Math.max(xmax,p.x); ymin=Math.min(ymin,p.y); ymax=Math.max(ymax,p.y); }}
  const pad=24;
  const sx = (canvas.width-2*pad)/Math.max(1e-6, xmax-xmin);
  const sy = (canvas.height-2*pad)/Math.max(1e-6, ymax-ymin);
  const s = Math.min(sx,sy);
  const ox = pad - xmin*s;
  const oy = canvas.height - pad + ymin*s;
  const map = p => [ox + p.x*s, oy - p.y*s];
  ctx.lineWidth = 1.5;
  for (let i=1;i<pts.length;i++) {{
    const a=pts[i-1], b=pts[i];
    ctx.beginPath();
    const [x0,y0]=map(a), [x1,y1]=map(b);
    ctx.moveTo(x0,y0); ctx.lineTo(x1,y1);
    ctx.strokeStyle = b.extrude ? '#8ab4f8' : '#5f6368';
    ctx.stroke();
  }}
}}
main().catch(e => {{ document.body.insertAdjacentHTML('beforeend','<pre>'+e+'</pre>'); }});
</script>
</body></html>"#,
        title = ent.path.display(),
        base = base,
        idx = idx,
        meta = meta,
    )
}

fn robot_html(ent: &Entry, idx: usize, base: &str) -> String {
    let meta = ent.meta_name.as_deref().unwrap_or("robot.json");
    format!(
        r#"<!doctype html>
<html><head><meta charset="utf-8"><title>URDF jog · {title}</title>
<style>
body{{font-family:system-ui,sans-serif;background:#12141a;color:#e8eaed;margin:1.5rem}}
a{{color:#8ab4f8}}
#c{{background:#1a1d24;border-radius:8px;width:min(720px,100%);height:480px;display:block}}
.sliders{{display:grid;gap:.75rem;max-width:720px;margin-top:1rem}}
label{{display:flex;flex-direction:column;gap:.25rem;font-size:.9rem}}
input[type=range]{{width:100%}}
.meta{{opacity:.75;font-size:.9rem}}
</style></head><body>
<p><a href="{base}/">← all</a></p>
<h1>URDF joint jog <small style="opacity:.6">alpha · 2D FK</small></h1>
<p class="meta">{title} · <a href="{base}/v/{idx}/{meta}">{meta}</a></p>
<canvas id="c" width="720" height="480"></canvas>
<div class="sliders" id="sliders"></div>
<p class="meta">Revolute about Z shown in XY; prismatic along axis projection. Alpha stick figure — not full 3D meshes.</p>
<script>
const metaUrl = "{base}/v/{idx}/{meta}";
const canvas = document.getElementById('c');
const ctx = canvas.getContext('2d');
let robot = null;
const q = {{}};
async function main() {{
  robot = await (await fetch(metaUrl)).json();
  const box = document.getElementById('sliders');
  for (const j of robot.joints) {{
    if (!j.movable) continue;
    q[j.name] = 0;
    const lab = document.createElement('label');
    lab.innerHTML = j.name+' ('+j.joint_type+') <span id="v_'+j.name+'">0</span>';
    const inp = document.createElement('input');
    inp.type = 'range';
    inp.min = j.lower; inp.max = j.upper; inp.step = (j.upper-j.lower)/200 || 0.01;
    inp.value = 0;
    inp.oninput = () => {{ q[j.name]=+inp.value; document.getElementById('v_'+j.name).textContent=inp.value; draw(); }};
    lab.appendChild(inp); box.appendChild(lab);
  }}
  draw();
}}
function mul(A,B) {{
  // 3x3 affine 2D [a b tx; c d ty; 0 0 1]
  return [
    A[0]*B[0]+A[1]*B[3], A[0]*B[1]+A[1]*B[4], A[0]*B[2]+A[1]*B[5]+A[2],
    A[3]*B[0]+A[4]*B[3], A[3]*B[1]+A[4]*B[4], A[3]*B[2]+A[4]*B[5]+A[5]
  ];
}}
function T(x,y,th) {{
  const c=Math.cos(th), s=Math.sin(th);
  return [c,-s,x, s,c,y];
}}
function apply(M,x,y) {{ return [M[0]*x+M[1]*y+M[2], M[3]*x+M[4]*y+M[5]]; }}
function draw() {{
  ctx.clearRect(0,0,canvas.width,canvas.height);
  // Build parent map
  const parentJ = {{}};
  for (const j of robot.joints) parentJ[j.child] = j;
  const poses = {{}};
  poses[robot.root] = T(0,0,0);
  // topological-ish: iterate joints multiple times
  for (let k=0;k<robot.joints.length+2;k++) {{
    for (const j of robot.joints) {{
      if (!poses[j.parent] || poses[j.child]) continue;
      const base = poses[j.parent];
      const ox=j.origin_xyz[0], oy=j.origin_xyz[1];
      let th = j.origin_rpy[2]||0;
      let px=ox, py=oy;
      const qq = q[j.name]||0;
      if (j.joint_type==='revolute' || j.joint_type==='continuous') {{
        // prefer Z axis yaw for planar jog
        th += qq * (Math.abs(j.axis[2])>=Math.abs(j.axis[0]) && Math.abs(j.axis[2])>=Math.abs(j.axis[1]) ? 1 : (j.axis[1]||j.axis[0]||1));
      }} else if (j.joint_type==='prismatic') {{
        px += qq * j.axis[0]; py += qq * j.axis[1];
      }}
      poses[j.child] = mul(base, T(px, py, th));
    }}
  }}
  // frame
  const scale = 400; // m -> px
  const cx = canvas.width/2, cy = canvas.height*0.75;
  ctx.strokeStyle = '#3c4043'; ctx.beginPath(); ctx.moveTo(40,cy); ctx.lineTo(canvas.width-40,cy); ctx.stroke();
  // draw joints as segments parent->child origin
  ctx.lineWidth = 4; ctx.lineCap='round';
  for (const j of robot.joints) {{
    const Mp = poses[j.parent], Mc = poses[j.child];
    if (!Mp || !Mc) continue;
    const [x0,y0]=apply(Mp,0,0);
    const [x1,y1]=apply(Mc,0,0);
    ctx.strokeStyle = j.movable ? '#8ab4f8' : '#9aa0a6';
    ctx.beginPath();
    ctx.moveTo(cx+x0*scale, cy-y0*scale);
    ctx.lineTo(cx+x1*scale, cy-y1*scale);
    ctx.stroke();
    ctx.fillStyle = '#fdd663';
    ctx.beginPath(); ctx.arc(cx+x1*scale, cy-y1*scale, 5, 0, Math.PI*2); ctx.fill();
  }}
  // root
  const Mr = poses[robot.root];
  if (Mr) {{
    const [x,y]=apply(Mr,0,0);
    ctx.fillStyle='#81c995';
    ctx.fillRect(cx+x*scale-8, cy-y*scale-8, 16, 16);
  }}
}}
main().catch(e => {{ document.body.insertAdjacentHTML('beforeend','<pre>'+e+'</pre>'); }});
</script>
</body></html>"#,
        title = ent.path.display(),
        base = base,
        idx = idx,
        meta = meta,
    )
}
