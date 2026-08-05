//! MCP tool implementations (thin wrappers over lang/inspect/render + fs).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use cadre_inspect::{inspect_refs, measure, MeasureKind, MeasureRequest};
use cadre_lang::{evaluate, EvalOptions};
use cadre_render::{mesh_from_ir, write_snapshot_packet, SnapshotOptions, ViewName};
use serde_json::{json, Value};

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("{0}")]
    Msg(String),
}

impl ToolError {
    fn msg(s: impl Into<String>) -> Self {
        Self::Msg(s.into())
    }
}

/// Short tool definitions for tools/list (keep under token budget).
pub fn tool_defs() -> Value {
    json!([
        {
            "name": "build",
            "description": "Evaluate .cad.star → IR (mock). Writes companion .ir.json. Returns facts JSON.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to .cad.star"},
                    "set": {"type": "object", "additionalProperties": {"type": "number"}, "description": "param overrides"}
                },
                "required": ["path"]
            }
        },
        {
            "name": "write_source",
            "description": "Write a .cad.star file (creates parents). Prefer explicit paths.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"}
                },
                "required": ["path", "content"]
            }
        },
        {
            "name": "read_source",
            "description": "Read a text source file.",
            "inputSchema": {
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"]
            }
        },
        {
            "name": "inspect_refs",
            "description": "List stable #o… selectors (+ optional facts) from .cad.star IR topology.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "facts": {"type": "boolean", "default": true}
                },
                "required": ["path"]
            }
        },
        {
            "name": "measure",
            "description": "Measure distance|angle|diameter|thickness between selectors.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "a": {"type": "string"},
                    "b": {"type": "string"},
                    "kind": {"type": "string", "enum": ["distance","angle","diameter","thickness"]}
                },
                "required": ["path", "a", "kind"]
            }
        },
        {
            "name": "snapshot",
            "description": "Render multi-view PNG + orbit GIF packet. Returns paths; PNG/GIF as base64 image content when include_images=true.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "views": {"type": "string", "default": "iso,front,top,right"},
                    "size": {"type": "integer", "default": 256},
                    "include_images": {"type": "boolean", "default": true}
                },
                "required": ["path"]
            }
        }
    ])
}

pub fn call_tool(name: &str, args: &Value) -> Result<Value, ToolError> {
    match name {
        "build" => tool_build(args),
        "write_source" => tool_write_source(args),
        "read_source" => tool_read_source(args),
        "inspect_refs" => tool_inspect_refs(args),
        "measure" => tool_measure(args),
        "snapshot" => tool_snapshot(args),
        other => Err(ToolError::msg(format!("unknown tool: {other}"))),
    }
}

fn str_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::msg(format!("missing string arg '{key}'")))
}

fn tool_write_source(args: &Value) -> Result<Value, ToolError> {
    let path = PathBuf::from(str_arg(args, "path")?);
    let content = str_arg(args, "content")?;
    if path.is_dir() {
        return Err(ToolError::msg("path is a directory"));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ToolError::msg(e.to_string()))?;
    }
    fs::write(&path, content).map_err(|e| ToolError::msg(e.to_string()))?;
    Ok(json!({
        "content": [{"type": "text", "text": serde_json::to_string_pretty(&json!({
            "ok": true,
            "path": path,
            "bytes": content.len()
        })).unwrap()}]
    }))
}

fn tool_read_source(args: &Value) -> Result<Value, ToolError> {
    let path = PathBuf::from(str_arg(args, "path")?);
    let text = fs::read_to_string(&path).map_err(|e| ToolError::msg(e.to_string()))?;
    Ok(json!({
        "content": [{"type": "text", "text": text}]
    }))
}

fn eval_path(path: &Path, set: Option<&Value>) -> Result<cadre_lang::FeatureIr, ToolError> {
    let source = fs::read_to_string(path).map_err(|e| ToolError::msg(e.to_string()))?;
    let mut opts = EvalOptions::new(
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("part.cad.star"),
    );
    if let Some(Value::Object(map)) = set {
        let mut o = BTreeMap::new();
        for (k, v) in map {
            let n = v
                .as_f64()
                .ok_or_else(|| ToolError::msg(format!("set.{k} must be number")))?;
            o.insert(k.clone(), n);
        }
        opts.overrides = o;
    }
    let r = evaluate(&source, &opts);
    if !r.ok {
        return Err(ToolError::msg(format!("eval failed: {:?}", r.diagnostics)));
    }
    Ok(r.ir.unwrap())
}

fn tool_build(args: &Value) -> Result<Value, ToolError> {
    let path = PathBuf::from(str_arg(args, "path")?);
    if path.is_dir() {
        return Err(ToolError::msg("directory builds refused"));
    }
    let ir = eval_path(&path, args.get("set"))?;
    let ir_path = {
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("part");
        let stem = name
            .strip_suffix(".cad.star")
            .or_else(|| name.strip_suffix(".star"))
            .unwrap_or(name);
        path.with_file_name(format!("{stem}.ir.json"))
    };
    let ir_json = serde_json::to_string_pretty(&ir).map_err(|e| ToolError::msg(e.to_string()))?;
    fs::write(&ir_path, &ir_json).map_err(|e| ToolError::msg(e.to_string()))?;

    // execute on mock for facts
    use cadre_kernel::{GeomKernel, MockKernel};
    use cadre_lang::execute_ir;
    let mut k = MockKernel::new();
    let shape = execute_ir(&mut k, &ir).map_err(|e| ToolError::msg(e.to_string()))?;
    let facts = k.facts(shape).map_err(|e| ToolError::msg(e.to_string()))?;

    let payload = json!({
        "ok": true,
        "ir_path": ir_path,
        "label": ir.label,
        "params": ir.params,
        "node_count": ir.node_count(),
        "facts": facts,
        "kernel": "mock",
        "note": "STEP requires --kernel occt CLI; MCP build is IR+facts on mock"
    });
    Ok(json!({
        "content": [{"type": "text", "text": serde_json::to_string_pretty(&payload).unwrap()}]
    }))
}

fn tool_inspect_refs(args: &Value) -> Result<Value, ToolError> {
    let path = PathBuf::from(str_arg(args, "path")?);
    let facts = args.get("facts").and_then(|v| v.as_bool()).unwrap_or(true);
    let ir = eval_path(&path, None)?;
    let snap = topo_from_ir(&ir)?;
    let report = inspect_refs(&snap, facts);
    Ok(json!({
        "content": [{"type": "text", "text": serde_json::to_string_pretty(&report).unwrap()}]
    }))
}

fn tool_measure(args: &Value) -> Result<Value, ToolError> {
    let path = PathBuf::from(str_arg(args, "path")?);
    let a = str_arg(args, "a")?.to_string();
    let b = args
        .get("b")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let kind = match str_arg(args, "kind")? {
        "distance" => MeasureKind::Distance,
        "angle" => MeasureKind::Angle,
        "diameter" => MeasureKind::Diameter,
        "thickness" => MeasureKind::Thickness,
        other => return Err(ToolError::msg(format!("bad kind {other}"))),
    };
    let ir = eval_path(&path, None)?;
    let snap = topo_from_ir(&ir)?;
    let r = measure(&snap, &MeasureRequest { a, b, kind })
        .map_err(|e| ToolError::msg(e.to_string()))?;
    Ok(json!({
        "content": [{"type": "text", "text": serde_json::to_string_pretty(&r).unwrap()}]
    }))
}

fn tool_snapshot(args: &Value) -> Result<Value, ToolError> {
    let path = PathBuf::from(str_arg(args, "path")?);
    let views_s = args
        .get("views")
        .and_then(|v| v.as_str())
        .unwrap_or("iso,front,top,right");
    let size = args.get("size").and_then(|v| v.as_u64()).unwrap_or(256) as u32;
    let include = args
        .get("include_images")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let ir = eval_path(&path, None)?;
    let (mesh, notes) = mesh_from_ir(&ir).map_err(ToolError::msg)?;
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("part");
    let stem = name
        .strip_suffix(".cad.star")
        .or_else(|| name.strip_suffix(".star"))
        .unwrap_or(name);
    let out = path.with_file_name(format!("{stem}.snap"));
    let opts = SnapshotOptions {
        views: ViewName::parse_list(views_s),
        width: size,
        height: size,
        gif: true,
        gif_frames: 12,
        gif_delay_cs: 6,
        notes,
    };
    let res = write_snapshot_packet(&mesh, &out, &opts).map_err(ToolError::msg)?;

    let mut content = vec![json!({
        "type": "text",
        "text": serde_json::to_string_pretty(&json!({
            "ok": true,
            "out_dir": res.manifest.out_dir,
            "views": res.manifest.views,
            "gif": res.manifest.gif,
            "notes": res.manifest.notes,
            "preview_mesh": true
        })).unwrap()
    })];

    if include {
        for v in &res.manifest.views {
            if v.name == "iso" || v.name == "front" {
                if let Ok(bytes) = fs::read(&v.path) {
                    use base64::Engine;
                    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                    content.push(json!({
                        "type": "image",
                        "data": b64,
                        "mimeType": "image/png"
                    }));
                }
            }
        }
    }

    Ok(json!({ "content": content }))
}

fn topo_from_ir(ir: &cadre_lang::FeatureIr) -> Result<cadre_inspect::TopologySnapshot, ToolError> {
    // Duplicate thin walker (same as bench) to avoid depending on cadre-cli.
    use cadre_inspect::{box_topology, cylinder_topology, SolidRec, TopologySnapshot};
    use cadre_kernel::Point3;
    use cadre_lang::{BooleanKind, IrNode};

    let mut solids: Vec<Option<SolidRec>> = vec![None; ir.nodes.len()];
    for (idx, node) in ir.nodes.iter().enumerate() {
        let rec = match node {
            IrNode::Box { dx, dy, dz, at } => {
                box_topology(*dx, *dy, *dz, Point3::new(at[0], at[1], at[2]))
            }
            IrNode::Cylinder { radius, height, at } => {
                cylinder_topology(*radius, *height, Point3::new(at[0], at[1], at[2]))
            }
            IrNode::Boolean { kind, a, b } => {
                let sa = solids
                    .get(a.0 as usize)
                    .and_then(|s| s.as_ref())
                    .ok_or_else(|| ToolError::msg("bad IR node"))?;
                let sb = solids
                    .get(b.0 as usize)
                    .and_then(|s| s.as_ref())
                    .ok_or_else(|| ToolError::msg("bad IR node"))?;
                let volume = match kind {
                    BooleanKind::Union => sa.volume_mm3 + sb.volume_mm3,
                    BooleanKind::Cut => (sa.volume_mm3 - sb.volume_mm3).max(0.0),
                    BooleanKind::Intersect => sa.volume_mm3.min(sb.volume_mm3),
                };
                SolidRec {
                    volume_mm3: volume,
                    centroid: sa.centroid,
                    faces: sa.faces.clone(),
                    edges: sa.edges.clone(),
                    vertices: sa.vertices.clone(),
                }
            }
            IrNode::Fillet { of, .. } | IrNode::Chamfer { of, .. } | IrNode::Label { of, .. } => {
                solids
                    .get(of.0 as usize)
                    .and_then(|s| s.as_ref())
                    .ok_or_else(|| ToolError::msg("bad IR node"))?
                    .clone()
            }
        };
        solids[idx] = Some(rec);
    }
    let root = solids
        .get(ir.root.0 as usize)
        .and_then(|s| s.as_ref())
        .ok_or_else(|| ToolError::msg("missing root"))?
        .clone();
    Ok(TopologySnapshot::single_solid(root))
}
