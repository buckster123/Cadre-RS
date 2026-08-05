//! Stdio + shared JSON-RPC dispatch.

use std::io::{self, BufReader, Write};

use serde_json::{json, Value};

use crate::protocol::{read_message, write_message, JsonRpcRequest, JsonRpcResponse};
use crate::tools::{call_tool, tool_defs};

/// Run until stdin EOF. Logs go to stderr only.
pub fn run_stdio() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdin = BufReader::new(stdin.lock());
    let mut stdout = io::stdout().lock();

    eprintln!("cadre-mcp {} ready (stdio)", crate::VERSION);

    while let Some(body) = read_message(&mut stdin)? {
        let req: JsonRpcRequest = match serde_json::from_slice(&body) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse::err(None, -32700, format!("parse error: {e}"));
                write_response(&mut stdout, &resp)?;
                continue;
            }
        };

        let resp = dispatch(req);
        if let Some(resp) = resp {
            if resp.id.is_some() || resp.error.is_some() {
                write_response(&mut stdout, &resp)?;
            }
        }
    }
    Ok(())
}

fn write_response(stdout: &mut impl Write, resp: &JsonRpcResponse) -> io::Result<()> {
    let bytes =
        serde_json::to_vec(resp).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    write_message(stdout, &bytes)
}

/// Handle one JSON-RPC request (shared by stdio + HTTP).
pub fn dispatch(req: JsonRpcRequest) -> Option<JsonRpcResponse> {
    let id = req.id.clone();
    match req.method.as_str() {
        "initialize" => Some(JsonRpcResponse::ok(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "cadre",
                    "version": crate::VERSION,
                    "transports": ["stdio", "streamable-http"]
                }
            }),
        )),
        "notifications/initialized" | "initialized" => None,
        "ping" => Some(JsonRpcResponse::ok(id, json!({}))),
        "tools/list" => Some(JsonRpcResponse::ok(id, json!({ "tools": tool_defs() }))),
        "tools/call" => {
            let params = req.params.unwrap_or(Value::Null);
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match call_tool(name, &args) {
                Ok(result) => Some(JsonRpcResponse::ok(id, result)),
                Err(e) => Some(JsonRpcResponse::ok(
                    id,
                    json!({
                        "content": [{"type": "text", "text": format!("error: {e}")}],
                        "isError": true
                    }),
                )),
            }
        }
        "resources/list" => Some(JsonRpcResponse::ok(id, json!({ "resources": [] }))),
        "prompts/list" => Some(JsonRpcResponse::ok(id, json!({ "prompts": [] }))),
        other => {
            if id.is_some() {
                Some(JsonRpcResponse::err(
                    id,
                    -32601,
                    format!("method not found: {other}"),
                ))
            } else {
                None
            }
        }
    }
}

/// Parse body as one JSON-RPC request and return response JSON value.
pub fn handle_http_body(body: &[u8]) -> Result<Option<Value>, String> {
    let req: JsonRpcRequest =
        serde_json::from_slice(body).map_err(|e| format!("parse error: {e}"))?;
    Ok(dispatch(req).map(|r| serde_json::to_value(r).unwrap_or(Value::Null)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::call_tool;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn write_build_inspect_snapshot_loop() {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "cadre-mcp-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("block.cad.star");
        let content = r#"
P = params(w=40.0, d=20.0, h=10.0)
def gen_step():
    return solid(box(P.w, P.d, P.h, at=CENTER), label="block")
"#;
        call_tool(
            "write_source",
            &json!({"path": path.to_str().unwrap(), "content": content}),
        )
        .unwrap();
        let b = call_tool("build", &json!({"path": path.to_str().unwrap()})).unwrap();
        let text = b["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("\"ok\": true") || text.contains("\"ok\":true"));
        let r = call_tool(
            "inspect_refs",
            &json!({"path": path.to_str().unwrap(), "facts": true}),
        )
        .unwrap();
        assert!(r["content"][0]["text"].as_str().unwrap().contains("#o1"));
        let s = call_tool(
            "snapshot",
            &json!({
                "path": path.to_str().unwrap(),
                "size": 64,
                "include_images": false
            }),
        )
        .unwrap();
        assert!(s["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("orbit.gif"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn initialize_dispatch() {
        let resp = dispatch(JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "initialize".into(),
            params: Some(json!({})),
        })
        .unwrap();
        assert!(resp.error.is_none());
        let tools = dispatch(JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(2)),
            method: "tools/list".into(),
            params: None,
        })
        .unwrap();
        let n = tools.result.unwrap()["tools"].as_array().unwrap().len();
        assert_eq!(n, 6);
    }

    #[test]
    fn http_body_tools_list() {
        let body = br#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
        let v = handle_http_body(body).unwrap().unwrap();
        assert_eq!(v["result"]["tools"].as_array().unwrap().len(), 6);
    }
}
