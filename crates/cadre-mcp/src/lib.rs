//! Cadre MCP server — stdio JSON-RPC (MCP-shaped), tool surface budget-tight.

#![deny(unsafe_code)]

mod protocol;
mod server;
mod tools;

pub use server::run_stdio;
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
