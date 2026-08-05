//! Cadre MCP server — stdio + streamable HTTP JSON-RPC.

#![deny(unsafe_code)]

mod http;
mod policy;
mod protocol;
mod resources;
mod server;
mod tools;

pub use http::{serve_http, HttpMcpConfig};
pub use policy::{init_policy, policy, project_root_from_env, write_source_from_env, McpPolicy};
pub use server::{dispatch, handle_http_body, run_stdio};
pub use tools::{call_tool, tool_defs, ToolError};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Shared entry for HTTP API (returns full MCP tool result JSON).
pub fn tools_call_for_api(
    name: &str,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    call_tool(name, args).map_err(|e| e.to_string())
}
