//! Task / step schema.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    /// Natural-language prompt (what an agent would see).
    pub prompt: String,
    #[serde(default = "default_max_loops")]
    pub max_loops: u32,
    /// Ordered loops; each loop is a list of steps. Runner stops at first success.
    pub loops: Vec<Vec<Step>>,
}

fn default_max_loops() -> u32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Step {
    /// Write Starlark source relative to task workdir.
    Write { path: String, content: String },
    /// Evaluate + execute mock kernel; stash facts.
    Build { path: String },
    /// inspect refs; optional facts.
    InspectRefs {
        path: String,
        #[serde(default)]
        facts: bool,
    },
    /// Software snapshot packet (no images required for score).
    Snapshot {
        path: String,
        #[serde(default = "default_snap_size")]
        size: u32,
    },
    /// Assertions against last build facts / inspect.
    Assert {
        #[serde(default)]
        volume_min: Option<f64>,
        #[serde(default)]
        volume_max: Option<f64>,
        #[serde(default)]
        faces_min: Option<u32>,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        has_selector_prefix: Option<String>,
        #[serde(default)]
        snapshot_ok: bool,
    },
}

fn default_snap_size() -> u32 {
    64
}

/// Shared assert payload (for docs / external tools).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssertSpec {
    pub volume_min: Option<f64>,
    pub volume_max: Option<f64>,
    pub faces_min: Option<u32>,
    pub label: Option<String>,
    pub has_selector_prefix: Option<String>,
    pub snapshot_ok: bool,
}
