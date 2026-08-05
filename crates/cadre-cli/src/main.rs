//! Cadre CLI binary.

mod bench_cmd;
mod build_cmd;
mod cli;
mod export_cmd;
mod inspect_cmd;
mod kernel_pick;
mod mcp_cmd;
mod output;
mod snapshot_cmd;
mod topo_from_ir;
mod view_cmd;

use clap::Parser;
use cli::{Cli, Commands};
use output::{emit, ExitCode};

fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let code = match &cli.command {
        Commands::Build(args) => build_cmd::run(&cli, args),
        Commands::Inspect(args) => inspect_cmd::run(&cli, args),
        Commands::Export(args) => export_cmd::run(&cli, args),
        Commands::Bench(args) => bench_cmd::run(&cli, args),
        Commands::Snapshot(args) => snapshot_cmd::run(&cli, args),
        Commands::View(args) => view_cmd::run(&cli, args),
        Commands::Mcp(args) => mcp_cmd::run_mcp(&cli, args),
        Commands::Skills(args) => mcp_cmd::run_skills(&cli, args),
        Commands::Version => {
            let v = serde_json::json!({
                "ok": true,
                "cadre": env!("CARGO_PKG_VERSION"),
                "kernel_default": kernel_pick::default_kernel_id(),
                "features": {
                    "occt": cfg!(feature = "occt"),
                }
            });
            emit(cli.json, &v, true);
            ExitCode::Ok
        }
    };
    std::process::exit(code as i32);
}

fn init_tracing(verbose: bool) {
    let filter = if verbose { "debug" } else { "warn" };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
