//! Deterministic parity suite — reference parts + expect.json assertions.

#![deny(unsafe_code)]

mod expect;
mod runner;

pub use expect::{Expect, FindFace, MeasureExpect};
pub use runner::{default_parity_root, run_part, run_suite, PartResult, SuiteReport};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Suite id for parts 1–4 (M1 exit).
pub const SUITE_PARTS_1_4: &str = "parts1-4";
