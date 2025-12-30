# Command Architecture

This document details the unified command architecture that powers all front-ends (CLI, REST API, programmatic) and enables consistent execution across all backends.

## 🎯 Unified Command Model

The engine uses a single `Command` enum that defines all possible operations, processed by a unified dispatcher that works identically regardless of invocation method.

### Command Definition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Discover available toy instances
    ListInstances,

    /// Execute a named instance with optional overrides
    RunInstance(RunInstanceArgs),

    /// Deterministic replay from saved snapshot/event log
    Replay(ReplayArgs),
}
```

### Command Result

Every command returns a structured result with consistent error handling:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub ok: bool,
    pub message: String,
    pub data: serde_json::Value, // Generic payload for flexibility
}
```

## 🚀 Command Dispatcher

### Core Execution Function

All commands flow through a single entry point:

```rust
pub fn execute(cmd: Command) -> CommandResult {
    match cmd {
        Command::ListInstances => list_instances(),
        Command::RunInstance(args) => run_instance(args),
        Command::Replay(args) => replay(args),
    }
}
```

This ensures **identical behavior** across CLI, REST API, and programmatic usage.

### Instance Discovery

```rust
fn list_instances() -> CommandResult {
    let instances = vec![
        "triangle_attractor_v0",
        // Future instances...
    ];

    CommandResult {
        ok: true,
        message: "instances".to_string(),
        data: json!({ "instances": instances }),
    }
}
```

### Instance Execution

```rust
fn run_instance(args: RunInstanceArgs) -> CommandResult {
    // 1. Load toy instance from mhg-testkit
    let inst = match args.instance.as_str() {
        "triangle_attractor_v0" => triangle_attractor_instance(),
        _ => return error_result("unknown instance"),
    };

    // 2. Execute with selected backend
    let summary = run_toy_instance_and_collect_summary(
        &inst,
        args.ticks,
        args.trace,
        &args.backend,
    );

    // 3. Serialize comprehensive RunSummary
    match serde_json::to_value(summary) {
        Ok(data) => CommandResult {
            ok: true,
            message: "run_complete".to_string(),
            data,
        },
        Err(e) => error_result(&format!("serialization error: {}", e)),
    }
}
```

## 🎛️ Backend Abstraction

### Backend Kind Enumeration

The system supports 5 distinct execution models:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendKind {
    /// Unlimited resources baseline
    SoftwareIdeal,

    /// Fixed-point arithmetic + memory counters
    ToyHardware,

    /// 64-bit events + credit flow control
    ToyGescFabric,

    /// Two-phase execution + structural barriers
    ToyGescBarrier,

    /// Field/constraint/topology loci
    ToyGesaLoci,

    /// Multi-lane execution with conflict detection
    ToyMultiLane,
}
```

### Backend Execution Logic

Each backend implements different execution semantics while producing identical `RunSummary` structure:

```rust
fn run_toy_instance_and_collect_summary(
    inst: &ToyInstance,
    ticks_override: Option<u32>,
    trace: bool,
    backend: &BackendKind,
) -> RunSummary {
    let ticks = ticks_override.unwrap_or(inst.schedule.ticks);

    let (hw_counts, duration) = match backend {
        BackendKind::SoftwareIdeal => {
            // Reference implementation - unlimited precision
            run_toy_instance(inst);
            (None, start.elapsed())
        }
        BackendKind::ToyHardware |
        BackendKind::ToyGescFabric |
        BackendKind::ToyGescBarrier |
        BackendKind::ToyGesaLoci |
        BackendKind::ToyMultiLane => {
            // Hardware-constrained execution
            let mut hw = ToyHw::new();
            hw.load_instance(inst);

            for _ in 0..ticks {
                match backend {
                    BackendKind::ToyHardware => hw.step_cycle(),
                    BackendKind::ToyGescBarrier | BackendKind::ToyGescFabric => hw.step_tick(),
                    BackendKind::ToyMultiLane => hw.step_lanes(),
                    _ => hw.step_tick(), // GESA uses barrier semantics
                }
            }

            (Some(hw.get_hw_counts()), start.elapsed())
        }
    };

    // Construct comprehensive RunSummary...
}
```

## 📊 RunSummary: Comprehensive Execution Report

Every execution produces a detailed summary enabling analysis, verification, and hardware insights.

### Run Metadata

```rust
#[derive(Debug, Clone, Serialize)]
pub struct RunMeta {
    pub instance_id: String,           // "triangle_attractor_v0"
    pub solver_id: String,             // "mhg-toy-0.1.0"
    pub determinism_level: DeterminismLevel,
    pub ticks_requested: u32,
    pub ticks_executed: u32,
    pub started_ns: u64,               // Monotonic timing
    pub duration_ns: u64,              // Total execution time
    pub nodes: u32,                    // Graph scale
    pub edges: u32,
}
```

### Determinism Levels

```rust
pub enum DeterminismLevel {
    /// Same code + machine + build → bitwise identical traces
    LocalBitwise,

    /// Cross-platform with epsilon geometry tolerance
    CrossPlatformEps { eps: f32 },

    /// No determinism guarantees (early development)
    None,
}
```

### Execution Hashes

Determinism anchors for verification and replay:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct RunHashes {
    /// Hash of canonicalized instance
    pub compile_hash: Option<String>,

    /// Hash of normalized execution trace
    pub trace_hash: Option<String>,

    /// Hash of final geometric state
    pub final_state_hash: Option<String>,

    /// Engine semantics version
    pub engine_semantics_hash: Option<String>,
}
```

### Execution Counts

Quantitative metrics from execution:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct RunCounts {
    // Execution scale
    pub ticks: u32,
    pub events_popped: u64,
    pub edges_fired: u64,
    pub nodes_moved: u64,

    // Computational costs
    pub gate_eval_calls: u64,
    pub geodesic_calls: u64,
    pub spatial_index_queries: u64,
    pub constraints_applied: u64,

    // Structural state
    pub active_edges_final: u32,
    pub inactive_edges_final: u32,

    // Performance monitoring
    pub allocations_estimate: Option<u64>,

    // Hardware-specific counters
    pub hw_counts: Option<HwCounts>,
}
```

### Hardware Counters

Detailed counters from hardware-constrained backends:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct HwCounts {
    // Execution cycles
    pub cycles: u64,

    // Memory access patterns
    pub node_mem_reads: u64,
    pub node_mem_writes: u64,
    pub edge_mem_reads: u64,

    // Event system utilization
    pub event_q_pushes: u64,
    pub event_q_pops: u64,
    pub event_q_overflows: u64,

    // Credit flow control
    pub credit_consumes: u64,
    pub credit_releases: u64,
    pub credit_denies: u64,

    // Topology management
    pub pending_topo_events: u64,

    // Parallel execution (multi-lane)
    pub lane_utilization: [u64; 2],     // Per-lane cycle counts
    pub node_write_conflicts: u64,      // Memory conflicts
    pub edge_read_conflicts: u64,
}
```

### Periodic Checkpoints

State snapshots for long-running simulations:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct CheckpointDigest {
    pub tick: u32,
    pub state_hash: String,  // Geometric state hash
}
```

### Trace Preview

Sampled execution trace for debugging:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct RunPreview {
    /// Sample of fired events (first K, last K)
    pub fired_events_head: Vec<FiredEvent>,
    pub fired_events_tail: Vec<FiredEvent>,

    /// Trajectory samples for interesting nodes
    pub node_trajectory_samples: Vec<NodeTrajectorySample>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FiredEvent {
    pub tick: u32,
    pub edge: u32,
    pub node: Option<u32>,  // Node-anchored events
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeTrajectorySample {
    pub node: u32,
    pub initial: [f32; 3],
    pub final_: [f32; 3],
    pub moved: bool,
    pub distance: f32,
}
```

### Built-in Property Checks

Automatic validation of expected behaviors:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct PropertyResult {
    pub name: String,          // "head_converged", "deterministic_trace"
    pub passed: bool,
    pub details: Option<String>,
}
```

## 🔄 Command Flow Architecture

### End-to-End Flow

```
CLI/REST/Programmatic → Command → execute() → Backend Selection
                                      ↓
                                RunSummary ← Execution ← ToyInstance
                                      ↓
                            JSON Serialization → Result
```

### Backend Selection Logic

Different backends reveal different architectural trade-offs:

| Backend | Primary Insight | Key Metrics |
|---------|----------------|-------------|
| SoftwareIdeal | Reference semantics | Execution time |
| ToyHardware | Memory access patterns | Reads/writes per operation |
| GescFabric | Event system design | Throughput, credit utilization |
| GescBarrier | Structural change overhead | Barrier frequency, intent processing |
| GesaLoci | Adaptive behavior patterns | Locus utilization, adaptation rates |
| MultiLane | Parallel execution limits | Conflict rates, lane utilization |

### Determinism Guarantees

- **Local Bitwise**: Same inputs → identical outputs (ToyHardware, SoftwareIdeal)
- **Cross-Platform Eps**: Geometric results within tolerance
- **None**: Development/testing phases

## 🧪 Testing the Command Architecture

### Command Interface Tests

```rust
#[test]
fn test_command_interface_consistency() {
    // Same command, different invocation methods
    let cmd = Command::RunInstance(RunInstanceArgs { /* ... */ });

    // CLI path
    let cli_result = execute_via_cli(cmd.clone());

    // REST API path
    let api_result = execute_via_api(cmd.clone());

    // Programmatic path
    let direct_result = execute(cmd);

    // All produce identical RunSummary
    assert_eq!(cli_result.data, api_result.data);
    assert_eq!(api_result.data, direct_result.data);
}
```

### Backend Equivalence Tests

```rust
#[test]
fn test_backend_semantic_equivalence() {
    let instance = triangle_attractor_instance();

    // Run on all backends
    let sw_result = run_on_backend(&instance, BackendKind::SoftwareIdeal);
    let hw_result = run_on_backend(&instance, BackendKind::ToyHardware);

    // Same fired edges and moved nodes
    assert_eq!(sw_result.counts.edges_fired, hw_result.counts.edges_fired);
    assert_eq!(sw_result.counts.nodes_moved, hw_result.counts.nodes_moved);

    // Geometry within epsilon (fixed-point vs floating-point)
    assert_points_close(sw_result.final_points, hw_result.final_points, 1e-3);
}
```

### RunSummary Completeness Tests

```rust
#[test]
fn test_runsummary_completeness() {
    let result = run_triangle_attractor();

    // All fields populated
    assert!(result.meta.ticks_executed > 0);
    assert!(result.counts.gate_eval_calls > 0);
    assert!(!result.preview.fired_events_head.is_empty());

    // Hardware counters when applicable
    if let Some(hw) = &result.counts.hw_counts {
        assert!(hw.node_mem_reads > 0);
        assert!(hw.cycles > 0);
    }
}
```

## 🚀 Extension Points

### Adding New Commands

1. Add variant to `Command` enum
2. Implement handler in `execute()` match
3. Update all front-ends (CLI, API)
4. Add tests for new command

### Adding New Backends

1. Add variant to `BackendKind` enum
2. Implement execution logic in `run_toy_instance_and_collect_summary`
3. Add CLI mapping in `toy-cli`
4. Add hardware counters if applicable
5. Update RunSummary construction

### Adding New Metrics

1. Extend `HwCounts` or `RunCounts` structures
2. Update backend implementations to populate metrics
3. Update serialization (derive Serialize)
4. Add tests for metric accuracy

## 📋 Command Architecture Benefits

### Consistency
- **Single Source of Truth**: One `execute()` function for all invocations
- **Identical Behavior**: CLI, API, and programmatic usage indistinguishable
- **Unified Error Handling**: Consistent `CommandResult` structure

### Extensibility
- **New Commands**: Easy to add via enum variants
- **New Backends**: Backend-specific logic isolated
- **New Metrics**: Structured addition to RunSummary

### Testability
- **Command Testing**: Test commands independently of front-ends
- **Backend Testing**: Test backends in isolation
- **Result Validation**: Comprehensive RunSummary validation

### Maintainability
- **Clear Separation**: Command model separate from execution logic
- **Type Safety**: Strongly typed commands and results
- **Documentation**: Self-documenting command structure