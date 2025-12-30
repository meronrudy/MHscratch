//! Toy CLI: command-line interface using clap

use clap::{Parser, Subcommand};
use toy_core::{Command, RunInstanceArgs, ReplayArgs, execute};

#[derive(Parser)]
#[command(name = "toy")]
#[command(about = "Toy hypergraph-manifold runner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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

    /// Start HTTP server (API)
    Serve {
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List {} => {
            let res = execute(Command::ListInstances);
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
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
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
        }
        Commands::Replay { snapshot } => {
            let res = execute(Command::Replay(ReplayArgs {
                snapshot_path: snapshot,
            }));
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
        }
        Commands::Serve { addr } => {
            println!("Starting HTTP server on {}...", addr);
            // This will be implemented when we create toy-api
            println!("HTTP API not yet implemented - use toy-api binary instead");
        }
    }
}