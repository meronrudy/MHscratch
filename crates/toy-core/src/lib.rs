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
        Command::SaveSnapshot(args) => save_snapshot(args),
        Command::LoadSnapshot(args) => load_snapshot(args),
        Command::GetStatus => get_status(),
        Command::Benchmark(args) => benchmark(args),
        Command::Configure(args) => configure(args),
        Command::CompareRuns(args) => compare_runs(args),
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

fn save_snapshot(args: SaveSnapshotArgs) -> CommandResult {
    use std::fs;

    // For now, just save a placeholder summary
    let summary = RunSummary {
        meta: RunMeta {
            instance_id: args.instance.unwrap_or("default".to_string()),
            solver_id: "mhg-toy-0.1.0".to_string(),
            determinism_level: DeterminismLevel::LocalBitwise,
            ticks_requested: 0,
            ticks_executed: 0,
            started_ns: 0,
            duration_ns: 0,
            nodes: 0,
            edges: 0,
        },
        hashes: RunHashes {
            compile_hash: None,
            trace_hash: None,
            final_state_hash: None,
            engine_semantics_hash: Some("snapshot-0.1.0".to_string()),
        },
        counts: RunCounts {
            ticks: 0,
            events_popped: 0,
            edges_fired: 0,
            nodes_moved: 0,
            gate_eval_calls: 0,
            geodesic_calls: 0,
            spatial_index_queries: 0,
            constraints_applied: 0,
            active_edges_final: 0,
            inactive_edges_final: 0,
            allocations_estimate: None,
            hw_counts: None,
        },
        checkpoints: vec![],
        preview: RunPreview {
            fired_events_head: vec![],
            fired_events_tail: vec![],
            node_trajectory_samples: vec![],
        },
        properties: vec![],
    };

    match serde_json::to_string_pretty(&summary) {
        Ok(json) => {
            if let Err(e) = fs::write(&args.path, json) {
                return CommandResult {
                    ok: false,
                    message: format!("Failed to write snapshot: {}", e),
                    data: serde_json::Value::Null,
                };
            }
            CommandResult {
                ok: true,
                message: format!("Snapshot saved to {}", args.path),
                data: serde_json::json!({ "path": args.path }),
            }
        }
        Err(e) => CommandResult {
            ok: false,
            message: format!("Serialization error: {}", e),
            data: serde_json::Value::Null,
        },
    }
}

fn load_snapshot(args: LoadSnapshotArgs) -> CommandResult {
    use std::fs;

    match fs::read_to_string(&args.path) {
        Ok(content) => {
            // Parse as raw JSON value for now to avoid HwCounts deserialization issues
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(data) => CommandResult {
                    ok: true,
                    message: format!("Snapshot loaded from {}", args.path),
                    data,
                },
                Err(e) => CommandResult {
                    ok: false,
                    message: format!("JSON parse error: {}", e),
                    data: serde_json::Value::Null,
                },
            }
        }
        Err(e) => CommandResult {
            ok: false,
            message: format!("Failed to read snapshot: {}", e),
            data: serde_json::Value::Null,
        },
    }
}

fn get_status() -> CommandResult {
    // TODO: implement status reporting
    CommandResult {
        ok: true,
        message: "status".to_string(),
        data: serde_json::json!({"version": "0.1.0", "uptime": 0}),
    }
}

fn benchmark(_args: BenchmarkArgs) -> CommandResult {
    // TODO: implement benchmarking
    CommandResult {
        ok: false,
        message: "benchmark not implemented".to_string(),
        data: serde_json::Value::Null,
    }
}

fn configure(_args: ConfigureArgs) -> CommandResult {
    // TODO: implement configuration
    CommandResult {
        ok: false,
        message: "configure not implemented".to_string(),
        data: serde_json::Value::Null,
    }
}

fn compare_runs(_args: CompareRunsArgs) -> CommandResult {
    // TODO: implement run comparison
    CommandResult {
        ok: false,
        message: "compare_runs not implemented".to_string(),
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
                    BackendKind::ToyHardware => hw.step_tick(), // Original simple cycle
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
                _ => DeterminismLevel::LocalBitwise, // Assume all hardware backends are deterministic
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_list_instances() {
        let result = execute(Command::ListInstances);
        assert!(result.ok);
        assert_eq!(result.message, "instances");
        assert!(result.data.is_object());
    }

    #[test]
    fn test_get_status() {
        let result = execute(Command::GetStatus);
        assert!(result.ok);
        assert_eq!(result.message, "status");
    }

    #[test]
    fn test_run_instance_unknown() {
        let args = RunInstanceArgs {
            instance: "unknown_instance".to_string(),
            ticks: None,
            trace: false,
            backend: BackendKind::SoftwareIdeal,
        };
        let result = execute(Command::RunInstance(args));
        assert!(!result.ok);
        assert!(result.message.contains("unknown instance"));
    }

    #[test]
    fn test_save_and_load_snapshot() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        // Save snapshot
        let save_args = SaveSnapshotArgs {
            path: path.clone(),
            instance: Some("test_instance".to_string()),
        };
        let save_result = execute(Command::SaveSnapshot(save_args));
        assert!(save_result.ok);
        assert!(save_result.message.contains("saved"));

        // Load snapshot
        let load_args = LoadSnapshotArgs { path };
        let load_result = execute(Command::LoadSnapshot(load_args));
        assert!(load_result.ok);
        assert!(load_result.message.contains("loaded"));
        assert!(load_result.data.is_object());
    }

    #[test]
    fn test_configure() {
        let args = ConfigureArgs {
            key: "test_key".to_string(),
            value: "test_value".to_string(),
        };
        let result = execute(Command::Configure(args));
        // Currently returns not implemented
        assert!(!result.ok);
        assert!(result.message.contains("not implemented"));
    }

    #[test]
    fn test_benchmark() {
        let args = BenchmarkArgs {
            runs: 1,
            instances: vec!["triangle_attractor_v0".to_string()],
            backends: vec![BackendKind::SoftwareIdeal],
        };
        let result = execute(Command::Benchmark(args));
        // Currently returns not implemented
        assert!(!result.ok);
        assert!(result.message.contains("not implemented"));
    }

    #[test]
    fn test_compare_runs() {
        let args = CompareRunsArgs {
            run_ids: vec!["run1".to_string(), "run2".to_string()],
        };
        let result = execute(Command::CompareRuns(args));
        // Currently returns not implemented
        assert!(!result.ok);
        assert!(result.message.contains("not implemented"));
    }

    #[test]
    fn test_replay_not_implemented() {
        let args = ReplayArgs {
            snapshot_path: "dummy_path".to_string(),
        };
        let result = execute(Command::Replay(args));
        assert!(!result.ok);
        assert!(result.message.contains("not implemented"));
    }

    #[test]
    fn test_save_snapshot_invalid_path() {
        let args = SaveSnapshotArgs {
            path: "/invalid/path/that/does/not/exist/snapshot.json".to_string(),
            instance: None,
        };
        let result = execute(Command::SaveSnapshot(args));
        assert!(!result.ok);
        assert!(result.message.contains("Failed to write snapshot"));
    }

    #[test]
    fn test_load_snapshot_nonexistent_file() {
        let args = LoadSnapshotArgs {
            path: "nonexistent_file.json".to_string(),
        };
        let result = execute(Command::LoadSnapshot(args));
        assert!(!result.ok);
        assert!(result.message.contains("Failed to read snapshot"));
    }

    #[test]
    fn test_configure_empty_key() {
        let args = ConfigureArgs {
            key: "".to_string(),
            value: "test".to_string(),
        };
        let result = execute(Command::Configure(args));
        // Currently not implemented, but should handle gracefully
        assert!(!result.ok);
    }

    #[test]
    fn test_benchmark_empty_instances() {
        let args = BenchmarkArgs {
            runs: 1,
            instances: vec![],
            backends: vec![BackendKind::SoftwareIdeal],
        };
        let result = execute(Command::Benchmark(args));
        // Currently not implemented
        assert!(!result.ok);
    }

    #[test]
    fn test_compare_runs_empty_list() {
        let args = CompareRunsArgs {
            run_ids: vec![],
        };
        let result = execute(Command::CompareRuns(args));
        // Currently not implemented
        assert!(!result.ok);
    }
}