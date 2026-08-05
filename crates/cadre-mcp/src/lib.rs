//! Cadre MCP server — stdio JSON-RPC (MCP-shaped), tool surface budget-tight.

#![deny(unsafe_code)]

mod protocol;
mod server;
mod tools;

pub use server::run_stdio;
pub use tools::{tool_defs, ToolError};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
