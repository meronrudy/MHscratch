//! RunSummary: what a solver run must tell the verifier

use serde::Serialize;
use mhg_testkit::HwCounts;

/// High-level summary of a solver run
#[derive(Debug, Clone, Serialize)]
pub struct RunSummary {
    pub meta: RunMeta,
    pub hashes: RunHashes,
    pub counts: RunCounts,
    pub checkpoints: Vec<CheckpointDigest>,
    pub preview: RunPreview,
    pub properties: Vec<PropertyResult>,
}

/// Metadata about what ran
#[derive(Debug, Clone, Serialize)]
pub struct RunMeta {
    pub instance_id: String,      // "triangle_attractor_v0"
    pub solver_id: String,        // "mhg-toy-0.1.0"
    pub determinism_level: DeterminismLevel,
    pub ticks_requested: u32,
    pub ticks_executed: u32,
    pub started_ns: u64,          // monotonic or wall clock, your choice
    pub duration_ns: u64,         // total run time for this instance
    pub nodes: u32,
    pub edges: u32,
}

/// Determinism guarantees
#[derive(Debug, Clone, Serialize)]
pub enum DeterminismLevel {
    // Same code + same machine + same build → bitwise same trace & state
    LocalBitwise,

    // Cross-platform, same trace hash, geometry within epsilon
    CrossPlatformEps { eps: f32 },

    // No determinism guarantee (only for very early toy runs)
    None,
}

/// Determinism anchors
#[derive(Debug, Clone, Serialize)]
pub struct RunHashes {
    /// Hash of canonicalized instance (ToyInstance → canonical bytes)
    pub compile_hash: Option<String>,

    /// Hash of the normalized trace (event ordering, edge IDs, epochs)
    pub trace_hash: Option<String>,

    /// Hash of final state (points, epoch, t, etc.) in strict mode
    pub final_state_hash: Option<String>,

    /// Hash of engine version / semantics version (optional)
    pub engine_semantics_hash: Option<String>,
}



/// Structural and causal metrics
#[derive(Debug, Clone, Serialize)]
pub struct RunCounts {
    // Execution scale
    pub ticks: u32,
    pub events_popped: u64,
    pub edges_fired: u64,
    pub nodes_moved: u64,

    // Geometry/causality costs
    pub gate_eval_calls: u64,
    pub geodesic_calls: u64,
    pub spatial_index_queries: u64,
    pub constraints_applied: u64,

    // Structural
    pub active_edges_final: u32,
    pub inactive_edges_final: u32,

    // Performance/debug
    pub allocations_estimate: Option<u64>,   // filled if you integrate an alloc guard

    // Hardware toy specific
    pub hw_counts: Option<HwCounts>,
}

/// Periodic state snapshots
#[derive(Debug, Clone, Serialize)]
pub struct CheckpointDigest {
    pub tick: u32,
    pub state_hash: String,
}

/// Small slices of trace and geometry for inspection
#[derive(Debug, Clone, Serialize)]
pub struct RunPreview {
    /// First K fired events (for debugging)
    pub fired_events_head: Vec<FiredEvent>,

    /// Last K fired events (to see where it converged/ended)
    pub fired_events_tail: Vec<FiredEvent>,

    /// For a small subset of "interesting" nodes, show their initial/final positions
    pub node_trajectory_samples: Vec<NodeTrajectorySample>,
}

/// Individual firing event
#[derive(Debug, Clone, Serialize)]
pub struct FiredEvent {
    pub tick: u32,
    pub edge: u32, // EdgeIx
    pub node: Option<u32>, // if event is node-anchored
}

/// Node position changes over time
#[derive(Debug, Clone, Serialize)]
pub struct NodeTrajectorySample {
    pub node: u32, // NodeIx
    pub initial: [f32; 3],
    pub final_: [f32; 3],
    pub moved: bool,
    pub distance: f32,
}

/// Built-in property checks
#[derive(Debug, Clone, Serialize)]
pub struct PropertyResult {
    pub name: String,          // "monotone_distance", "head_converged", etc.
    pub passed: bool,
    pub details: Option<String>,
}