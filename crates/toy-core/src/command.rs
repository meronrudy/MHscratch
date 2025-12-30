//! Command model shared by CLI and API

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// List available toy instances
    ListInstances,

    /// Run a named toy instance with optional overrides
    RunInstance(RunInstanceArgs),

    /// Replay from a saved snapshot/event log
    Replay(ReplayArgs),

    /// Save current state to snapshot
    SaveSnapshot(SaveSnapshotArgs),

    /// Load state from snapshot
    LoadSnapshot(LoadSnapshotArgs),

    /// Get system status
    GetStatus,

    /// Run benchmarks on instances
    Benchmark(BenchmarkArgs),

    /// Configure system settings
    Configure(ConfigureArgs),

    /// Compare multiple runs
    CompareRuns(CompareRunsArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendKind {
    SoftwareIdeal,
    ToyHardware,
    ToyGescFabric,    // GESC v0: events + credits
    ToyGescBarrier,   // GESC v1: + structural barriers
    ToyGesaLoci,      // GESA v2: + loci
    ToyMultiLane,     // Multi-lane with conflict detection
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunInstanceArgs {
    pub instance: String,   // "triangle_attractor_v0"
    pub ticks: Option<u32>, // override default ticks
    pub trace: bool,        // whether to capture full trace
    pub backend: BackendKind, // which backend to use
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayArgs {
    pub snapshot_path: String,  // or inline snapshot data
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSnapshotArgs {
    pub path: String,
    pub instance: Option<String>, // optional specific instance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadSnapshotArgs {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkArgs {
    pub runs: u32,
    pub instances: Vec<String>,
    pub backends: Vec<BackendKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureArgs {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareRunsArgs {
    pub run_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub ok: bool,
    pub message: String,
    pub data: serde_json::Value, // generic payload
}