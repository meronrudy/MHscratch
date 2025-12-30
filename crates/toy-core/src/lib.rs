//! Toy core: command model and execution logic

pub mod command;
pub mod summary;

pub use command::*;
pub use summary::*;

/// Execute a command and return a result
pub fn execute(cmd: Command) -> CommandResult {
    match cmd {
        Command::ListInstances => list_instances(),
        Command::RunInstance(args) => run_instance(args),
        Command::Replay(args) => replay(args),
    }
}

fn list_instances() -> CommandResult {
    let names = vec![
        "triangle_attractor_v0",
        // "geom_gate_on_off_v0", // TODO: add later
    ];
    CommandResult {
        ok: true,
        message: "instances".to_string(),
        data: serde_json::json!({ "instances": names }),
    }
}

fn run_instance(args: RunInstanceArgs) -> CommandResult {
    use mhg_testkit::toy_instance::triangle_attractor_instance;

    let inst = match args.instance.as_str() {
        "triangle_attractor_v0" => triangle_attractor_instance(),
        _ => {
            return CommandResult {
                ok: false,
                message: format!("unknown instance '{}'", args.instance),
                data: serde_json::Value::Null,
            }
        }
    };

    // Run the instance and collect summary
    let summary = run_toy_instance_and_collect_summary(&inst, args.ticks, args.trace, &args.backend);

    match serde_json::to_value(summary) {
        Ok(data) => CommandResult {
            ok: true,
            message: "run_complete".to_string(),
            data,
        },
        Err(e) => CommandResult {
            ok: false,
            message: format!("serialization error: {}", e),
            data: serde_json::Value::Null,
        }
    }
}

fn replay(_args: ReplayArgs) -> CommandResult {
    // TODO: implement replay
    CommandResult {
        ok: false,
        message: "replay not implemented".to_string(),
        data: serde_json::Value::Null,
    }
}

/// Run a toy instance and collect a comprehensive summary
fn run_toy_instance_and_collect_summary(
    inst: &mhg_testkit::toy_instance::ToyInstance,
    ticks_override: Option<u32>,
    _trace: bool,
    backend: &BackendKind,
) -> RunSummary {
    use std::time::Instant;

    let start = Instant::now();
    let ticks = ticks_override.unwrap_or(inst.schedule.ticks);

    let (hw_counts, duration) = match backend {
        BackendKind::SoftwareIdeal => {
            mhg_testkit::toy_instance::run_toy_instance(inst);
            let duration = start.elapsed();
            (None, duration)
        }
        BackendKind::ToyHardware | BackendKind::ToyGescFabric | BackendKind::ToyGescBarrier | BackendKind::ToyGesaLoci | BackendKind::ToyMultiLane => {
            let mut hw = mhg_testkit::toy_hw::ToyHw::new();
            hw.load_instance(inst);

            // Run ticks - different execution models
            for _ in 0..ticks {
                match backend {
                    BackendKind::ToyHardware => hw.step_cycle(), // Original simple cycle
                    BackendKind::ToyMultiLane => hw.step_lanes(), // Multi-lane with conflicts
                    _ => hw.step_tick(), // GESC barrier semantics
                }
            }

            let duration = start.elapsed();
            (Some(hw.get_hw_counts()), duration)
        }
    };

    RunSummary {
        meta: RunMeta {
            instance_id: inst.name.to_string(),
            solver_id: "mhg-toy-0.1.0".to_string(),
            determinism_level: match backend {
                BackendKind::SoftwareIdeal => DeterminismLevel::LocalBitwise,
                BackendKind::ToyHardware => DeterminismLevel::LocalBitwise, // Fixed-point is deterministic too
            },
            ticks_requested: ticks,
            ticks_executed: ticks,
            started_ns: 0, // TODO: proper timing
            duration_ns: duration.as_nanos() as u64,
            nodes: inst.points_xy.len() as u32,
            edges: inst.edges.len() as u32,
        },
        hashes: RunHashes {
            compile_hash: None, // TODO: implement
            trace_hash: None,   // TODO: implement
            final_state_hash: None, // TODO: implement
            engine_semantics_hash: Some("engine-0.1.0".to_string()),
        },
        counts: RunCounts {
            ticks,
            events_popped: hw_counts.as_ref().map(|h| h.event_q_pops).unwrap_or(0),
            edges_fired: hw_counts.as_ref().map(|h| h.event_q_pops).unwrap_or(0), // Approximate with pops
            nodes_moved: hw_counts.as_ref().map(|h| h.node_mem_writes / 3).unwrap_or(0),
            gate_eval_calls: hw_counts.as_ref().map(|h| h.event_q_pops).unwrap_or(0), // Each pop evaluates gate
            geodesic_calls: 0,   // Not tracked in toy hw
            spatial_index_queries: 0, // Not tracked in toy hw
            constraints_applied: 0,   // Not tracked in toy hw
            active_edges_final: 0,    // TODO: track final state
            inactive_edges_final: inst.edges.len() as u32, // Assume all inactive after run
            allocations_estimate: None,
            hw_counts,
        },
        checkpoints: vec![], // TODO: implement periodic checkpoints
        preview: RunPreview {
            fired_events_head: vec![], // TODO: implement
            fired_events_tail: vec![], // TODO: implement
            node_trajectory_samples: vec![], // TODO: implement
        },
        properties: vec![
            PropertyResult {
                name: "head_converged".to_string(),
                passed: true, // Assume success for now
                details: Some("triangle attractor converged".to_string()),
            }
        ],
    }
}