//! Clap surface.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "cadre",
    version,
    about = "Cadre — Rust-native CAD runtime for AI agents",
    long_about = None
)]
pub struct Cli {
    /// Machine-readable JSON on stdout (human text is a rendering of the same data).
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress non-essential human output.
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Project root (default: cwd).
    #[arg(long, global = true, env = "CADRE_PROJECT")]
    pub project: Option<PathBuf>,

    /// Geometry kernel: mock (default, no OCCT) or occt (needs --features occt build).
    #[arg(long, global = true, env = "CADRE_KERNEL", default_value = "mock")]
    pub kernel: KernelId,

    /// More logs on stderr.
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum KernelId {
    Mock,
    Occt,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Evaluate `.cad.star`, execute IR, write primary artifact (STEP when kernel supports it).
    Build(BuildArgs),
    /// Numeric interrogation (refs inventory / measure).
    Inspect(InspectArgs),
    /// Secondary format export from an existing STEP (or rebuild from source).
    Export(ExportArgs),
    /// Deterministic parity suite (`parts1-4`, …).
    Bench(BenchArgs),
    /// Multi-view PNG packet (+ orbit GIF).
    Snapshot(SnapshotArgs),
    /// Local embedded viewer (deep links).
    View(ViewArgs),
    /// Print versions / feature flags.
    Version,
}

#[derive(Debug, clap::Args)]
pub struct BuildArgs {
    /// Target `.cad.star` (explicit file only — no directory scans).
    pub target: PathBuf,

    /// Output path (default: same basename as target with .step or .ir.json).
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Parameter override `key=value` (repeatable).
    #[arg(long = "set", value_name = "KEY=VAL")]
    pub set: Vec<String>,

    /// Bypass build cache.
    #[arg(long)]
    pub no_cache: bool,
}

#[derive(Debug, clap::Args)]
pub struct InspectArgs {
    #[command(subcommand)]
    pub cmd: InspectCmd,
}

#[derive(Debug, Subcommand)]
pub enum InspectCmd {
    /// List stable selector tokens (+ optional facts).
    Refs(RefsArgs),
    /// Measure distance / angle / diameter / thickness between refs.
    Measure(MeasureArgs),
}

#[derive(Debug, clap::Args)]
pub struct RefsArgs {
    /// `.cad.star` source (topology derived from IR) or path ignored if --ir given.
    pub target: PathBuf,

    /// Attach aggregate facts summary.
    #[arg(long)]
    pub facts: bool,

    /// Parameter overrides when evaluating source.
    #[arg(long = "set", value_name = "KEY=VAL")]
    pub set: Vec<String>,
}

#[derive(Debug, clap::Args)]
pub struct MeasureArgs {
    pub target: PathBuf,
    /// Selector A, e.g. `#o1.1.f1`
    pub a: String,
    /// Selector B (required for distance/angle/thickness).
    pub b: Option<String>,
    #[arg(long, value_enum, default_value_t = MeasureKindArg::Distance)]
    pub kind: MeasureKindArg,
    #[arg(long = "set", value_name = "KEY=VAL")]
    pub set: Vec<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MeasureKindArg {
    Distance,
    Angle,
    Diameter,
    Thickness,
}

#[derive(Debug, clap::Args)]
pub struct ExportArgs {
    /// Source `.cad.star` or existing `.step`.
    pub target: PathBuf,

    #[arg(long, value_enum)]
    pub format: ExportFormat,

    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    #[arg(long = "set", value_name = "KEY=VAL")]
    pub set: Vec<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportFormat {
    Step,
    Stl,
    Glb,
}

#[derive(Debug, clap::Args)]
pub struct BenchArgs {
    #[command(subcommand)]
    pub cmd: BenchCmd,
}

#[derive(Debug, Subcommand)]
pub enum BenchCmd {
    /// Run a deterministic suite (default: parts1-4).
    Run(BenchRunArgs),
}

#[derive(Debug, clap::Args)]
pub struct BenchRunArgs {
    /// Suite id: parts1-4 | parity4 | m1
    #[arg(long, default_value = "parts1-4")]
    pub suite: String,

    /// Path to parity root (default: auto-detect `parity/` from cwd or crate layout).
    #[arg(long)]
    pub parity_root: Option<PathBuf>,
}

#[derive(Debug, clap::Args)]
pub struct SnapshotArgs {
    /// Target `.cad.star` (preview mesh from IR) or existing `.snap` dir (re-render not yet).
    pub target: PathBuf,

    /// Comma-separated views: iso,front,top,right,back,left,bottom
    #[arg(long, default_value = "iso,front,top,right")]
    pub views: String,

    /// Output directory (default: `<stem>.snap` next to target).
    #[arg(long, short = 'o')]
    pub out: Option<PathBuf>,

    /// Image size (square).
    #[arg(long, default_value_t = 512)]
    pub size: u32,

    /// Skip orbit GIF.
    #[arg(long)]
    pub no_gif: bool,

    /// Orbit frame count.
    #[arg(long, default_value_t = 24)]
    pub gif_frames: u32,

    #[arg(long = "set", value_name = "KEY=VAL")]
    pub set: Vec<String>,
}

#[derive(Debug, clap::Args)]
pub struct ViewArgs {
    /// Paths to open (`.snap` dirs, images, or `.cad.star` which is snapshotted first).
    pub paths: Vec<PathBuf>,

    /// Bind port (default 7411).
    #[arg(long, default_value_t = 7411, env = "CADRE_VIEWER_PORT")]
    pub port: u16,

    /// Bind host.
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Open once and exit after printing links (still serves until Ctrl-C unless --once).
    #[arg(long)]
    pub once: bool,
}
