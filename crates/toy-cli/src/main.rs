//! Toy CLI: command-line interface using clap

use clap::{Parser, Subcommand};
use toy_core::{Command, RunInstanceArgs, ReplayArgs, SaveSnapshotArgs, LoadSnapshotArgs, BenchmarkArgs, ConfigureArgs, CompareRunsArgs, CommandResult, execute};
use tokio;

fn format_output(result: &CommandResult, format: &str) -> String {
    match format {
        "json" => serde_json::to_string_pretty(result).unwrap(),
        "yaml" => serde_yaml::to_string(result).unwrap_or_else(|_| serde_json::to_string_pretty(result).unwrap()),
        "table" => {
            // Simple table format for basic results
            format!("Status: {}\nMessage: {}\nData: {}\n", result.ok, result.message, result.data)
        }
        _ => serde_json::to_string_pretty(result).unwrap(),
    }
}

#[derive(Parser)]
#[command(name = "toy")]
#[command(about = "Toy hypergraph-manifold runner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(long, default_value = "json")]
    format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// List available toy instances
    List {},

    /// Run a toy instance
    Run {
        #[arg(short, long)]
        instance: String,

        #[arg(long)]
        ticks: Option<u32>,

        #[arg(long)]
        trace: bool,

        #[arg(long, default_value = "sw")]
        backend: String,
    },

    /// Replay from snapshot
    Replay {
        #[arg(long)]
        snapshot: String,
    },

    /// Save current state to snapshot
    SaveSnapshot {
        #[arg(short, long)]
        path: String,

        #[arg(long)]
        instance: Option<String>,
    },

    /// Load state from snapshot
    LoadSnapshot {
        #[arg(short, long)]
        path: String,
    },

    /// Get system status
    Status {},

    /// Run benchmarks
    Benchmark {
        #[arg(short, long, default_value = "10")]
        runs: u32,

        #[arg(short, long)]
        instances: Vec<String>,

        #[arg(short, long)]
        backends: Vec<String>,
    },

    /// Configure settings
    Configure {
        #[arg(short, long)]
        key: String,

        #[arg(short, long)]
        value: String,
    },

    /// Compare runs
    Compare {
        #[arg(short, long)]
        run_ids: Vec<String>,
    },

    /// Start HTTP server (API)
    Serve {
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List {} => {
            let res = execute(Command::ListInstances);
            println!("{}", format_output(&res, &cli.format));
        }
        Commands::Run { instance, ticks, trace, backend } => {
            let backend_kind = match backend.as_str() {
                "sw" | "software" => toy_core::BackendKind::SoftwareIdeal,
                "hw" | "hardware" => toy_core::BackendKind::ToyHardware,
                "gesc-fabric" => toy_core::BackendKind::ToyGescFabric,
                "gesc-barrier" => toy_core::BackendKind::ToyGescBarrier,
                "gesa-loci" => toy_core::BackendKind::ToyGesaLoci,
                "multi-lane" => toy_core::BackendKind::ToyMultiLane,
                _ => {
                    eprintln!("Invalid backend: {}. Use 'sw', 'hw', 'gesc-fabric', 'gesc-barrier', 'gesa-loci', or 'multi-lane'", backend);
                    std::process::exit(1);
                }
            };

            let res = execute(Command::RunInstance(RunInstanceArgs {
                instance,
                ticks,
                trace,
                backend: backend_kind,
            }));
            println!("{}", format_output(&res, &cli.format));
        }
        Commands::Replay { snapshot } => {
            let res = execute(Command::Replay(ReplayArgs {
                snapshot_path: snapshot,
            }));
            println!("{}", format_output(&res, &cli.format));
        }
        Commands::SaveSnapshot { path, instance } => {
            let res = execute(Command::SaveSnapshot(SaveSnapshotArgs {
                path,
                instance,
            }));
            println!("{}", format_output(&res, &cli.format));
        }
        Commands::LoadSnapshot { path } => {
            let res = execute(Command::LoadSnapshot(LoadSnapshotArgs {
                path,
            }));
            println!("{}", format_output(&res, &cli.format));
        }
        Commands::Status {} => {
            let res = execute(Command::GetStatus);
            println!("{}", format_output(&res, &cli.format));
        }
        Commands::Benchmark { runs, instances, backends } => {
            let backend_kinds: Vec<toy_core::BackendKind> = backends
                .iter()
                .map(|b| match b.as_str() {
                    "sw" | "software" => toy_core::BackendKind::SoftwareIdeal,
                    "hw" | "hardware" => toy_core::BackendKind::ToyHardware,
                    "gesc-fabric" => toy_core::BackendKind::ToyGescFabric,
                    "gesc-barrier" => toy_core::BackendKind::ToyGescBarrier,
                    "gesa-loci" => toy_core::BackendKind::ToyGesaLoci,
                    "multi-lane" => toy_core::BackendKind::ToyMultiLane,
                    _ => {
                        eprintln!("Invalid backend: {}. Use 'sw', 'hw', 'gesc-fabric', 'gesc-barrier', 'gesa-loci', or 'multi-lane'", b);
                        std::process::exit(1);
                    }
                })
                .collect();

            let res = execute(Command::Benchmark(BenchmarkArgs {
                runs,
                instances,
                backends: backend_kinds,
            }));
            println!("{}", format_output(&res, &cli.format));
        }
        Commands::Configure { key, value } => {
            let res = execute(Command::Configure(ConfigureArgs {
                key,
                value,
            }));
            println!("{}", format_output(&res, &cli.format));
        }
        Commands::Compare { run_ids } => {
            let res = execute(Command::CompareRuns(CompareRunsArgs {
                run_ids,
            }));
            println!("{}", format_output(&res, &cli.format));
        }
        Commands::Serve { addr } => {
            println!("Starting HTTP server on {}...", addr);
            toy_api::serve(addr).await.expect("Server failed");
        }
    }
}