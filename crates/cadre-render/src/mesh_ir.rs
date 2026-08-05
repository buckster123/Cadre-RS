//! Preview mesh from feature IR (analytic tessellation; booleans are approximate).

use std::f64::consts::PI;

use cadre_kernel::Mesh;
use cadre_lang::{BooleanKind, FeatureIr, IrNode, NodeId};

/// Build a triangle mesh from IR for snapshot preview.
///
/// **Honesty:** Cut/Intersect do not perform true B-rep boolean on the mesh —
/// Cut keeps the first operand mesh (tool subtracted only in volume facts path).
/// Union concatenates meshes. Manifest should note `preview_mesh: true`.
pub fn mesh_from_ir(ir: &FeatureIr) -> Result<(Mesh, Vec<String>), String> {
    let mut notes = Vec::new();
    let mut meshes: Vec<Option<Mesh>> = vec![None; ir.nodes.len()];

    for (idx, node) in ir.nodes.iter().enumerate() {
        let m = match node {
            IrNode::Box { dx, dy, dz, at } => mesh_box(*dx, *dy, *dz, *at),
            IrNode::Cylinder { radius, height, at } => mesh_cylinder(*radius, *height, *at, 24),
            IrNode::Boolean { kind, a, b } => {
                let ma = take_mesh(&meshes, *a)?;
                let mb = take_mesh(&meshes, *b)?;
                match kind {
                    BooleanKind::Union => merge_meshes(&ma, &mb),
                    BooleanKind::Cut => {
                        notes.push(format!(
                            "preview mesh for cut keeps operand A only (node {})",
                            idx
                        ));
                        ma
                    }
                    BooleanKind::Intersect => {
                        notes.push(format!(
                            "preview mesh for intersect keeps operand A only (node {})",
                            idx
                        ));
                        ma
                    }
                }
            }
            IrNode::Translate { of, by } => {
                let mut m = take_mesh(&meshes, *of)?;
                translate_mesh(&mut m, *by);
                m
            }
            IrNode::Rotate { of, axis, deg } => {
                let mut m = take_mesh(&meshes, *of)?;
                rotate_mesh(&mut m, axis, *deg);
                notes.push(format!(
                    "preview mesh rotated about {axis} by {deg} deg (node {idx})"
                ));
                m
            }
            IrNode::Fillet { of, .. } | IrNode::Chamfer { of, .. } | IrNode::Label { of, .. } => {
                if matches!(node, IrNode::Fillet { .. } | IrNode::Chamfer { .. }) {
                    notes.push(format!(
                        "fillet/chamfer not reflected in preview mesh (node {})",
                        idx
                    ));
                }
                take_mesh(&meshes, *of)?
            }
        };
        meshes[idx] = Some(m);
    }

    let root = take_mesh(&meshes, ir.root)?;
    if root.triangle_count() == 0 {
        return Err("empty preview mesh".into());
    }
    Ok((root, notes))
}

fn take_mesh(meshes: &[Option<Mesh>], id: NodeId) -> Result<Mesh, String> {
    meshes
        .get(id.0 as usize)
        .and_then(|m| m.clone())
        .ok_or_else(|| format!("missing mesh for node {}", id.0))
}

fn merge_meshes(a: &Mesh, b: &Mesh) -> Mesh {
    let mut positions = a.positions.clone();
    let base = (a.positions.len() / 3) as u32;
    positions.extend_from_slice(&b.positions);
    let mut indices = a.indices.clone();
    indices.extend(b.indices.iter().map(|i| i + base));
    Mesh {
        positions,
        indices,
        normals: None,
    }
}

fn mesh_box(dx: f64, dy: f64, dz: f64, at: [f64; 3]) -> Mesh {
    let hx = (dx / 2.0) as f32;
    let hy = (dy / 2.0) as f32;
    let hz = (dz / 2.0) as f32;
    let (cx, cy, cz) = (at[0] as f32, at[1] as f32, at[2] as f32);
    let corners = [
        [cx - hx, cy - hy, cz - hz],
        [cx + hx, cy - hy, cz - hz],
        [cx + hx, cy + hy, cz - hz],
        [cx - hx, cy + hy, cz - hz],
        [cx - hx, cy - hy, cz + hz],
        [cx + hx, cy - hy, cz + hz],
        [cx + hx, cy + hy, cz + hz],
        [cx - hx, cy + hy, cz + hz],
    ];
    let mut positions = Vec::new();
    for c in corners {
        positions.extend_from_slice(&c);
    }
    let faces = [
        [0u32, 1, 2, 3],
        [4, 7, 6, 5],
        [0, 4, 5, 1],
        [2, 6, 7, 3],
        [0, 3, 7, 4],
        [1, 5, 6, 2],
    ];
    let mut indices = Vec::new();
    for f in faces {
        indices.extend_from_slice(&[f[0], f[1], f[2], f[0], f[2], f[3]]);
    }
    Mesh {
        positions,
        indices,
        normals: None,
    }
}

fn mesh_cylinder(radius: f64, height: f64, at: [f64; 3], segments: u32) -> Mesh {
    let n = segments.max(8);
    let (cx, cy, cz) = (at[0] as f32, at[1] as f32, at[2] as f32);
    let radius = radius as f32;
    let height = height as f32;
    let mut positions = Vec::new();
    positions.extend_from_slice(&[cx, cy, cz]);
    positions.extend_from_slice(&[cx, cy, cz + height]);
    for i in 0..n {
        let a = 2.0 * PI * (i as f64) / (n as f64);
        let x = cx + radius * a.cos() as f32;
        let y = cy + radius * a.sin() as f32;
        positions.extend_from_slice(&[x, y, cz]);
        positions.extend_from_slice(&[x, y, cz + height]);
    }
    let mut indices = Vec::new();
    let bot_c = 0u32;
    let top_c = 1u32;
    for i in 0..n {
        let i0 = 2 + i * 2;
        let i1 = 2 + ((i + 1) % n) * 2;
        let j0 = i0 + 1;
        let j1 = i1 + 1;
        indices.extend_from_slice(&[bot_c, i1, i0]);
        indices.extend_from_slice(&[top_c, j0, j1]);
        indices.extend_from_slice(&[i0, i1, j1, i0, j1, j0]);
    }
    Mesh {
        positions,
        indices,
        normals: None,
    }
}

fn translate_mesh(m: &mut Mesh, by: [f64; 3]) {
    let n = m.positions.len() / 3;
    for i in 0..n {
        m.positions[3 * i] += by[0] as f32;
        m.positions[3 * i + 1] += by[1] as f32;
        m.positions[3 * i + 2] += by[2] as f32;
    }
}

fn rotate_mesh(m: &mut Mesh, axis: &str, deg: f64) {
    let r = deg.to_radians();
    let (c, s) = (r.cos() as f32, r.sin() as f32);
    let ax = axis.to_ascii_lowercase();
    let n = m.positions.len() / 3;
    for i in 0..n {
        let x = m.positions[3 * i];
        let y = m.positions[3 * i + 1];
        let z = m.positions[3 * i + 2];
        let (nx, ny, nz) = match ax.as_str() {
            "x" => (x, y * c - z * s, y * s + z * c),
            "y" => (x * c + z * s, y, -x * s + z * c),
            _ => (x * c - y * s, x * s + y * c, z),
        };
        m.positions[3 * i] = nx;
        m.positions[3 * i + 1] = ny;
        m.positions[3 * i + 2] = nz;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadre_lang::{evaluate, EvalOptions};

    #[test]
    fn box_mesh_from_star() {
        let src = r#"
def gen_step():
    return solid(box(10.0, 20.0, 30.0, at=CENTER), label="b")
"#;
        let r = evaluate(src, &EvalOptions::new("t.cad.star"));
        assert!(r.ok);
        let (mesh, _) = mesh_from_ir(r.ir.as_ref().unwrap()).unwrap();
        assert_eq!(mesh.triangle_count(), 12);
    }
}
