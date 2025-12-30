# MHG Dynamic Hypergraph Engine

A high-performance geometric hypergraph engine for hardware design exploration, featuring deterministic execution across multiple backends and comprehensive testing infrastructure.

## 🎯 Overview

This project implements a dynamic hypergraph embedded in a dynamic Riemannian manifold, designed for exploring hardware architectures that combine geometric constraints with graph-based computation. The engine supports multiple execution models from software simulation to specialized hardware backends, all sharing a unified command interface.

**Key Features:**
- **Deterministic Execution**: Bit-for-bit reproducible results across runs
- **Multiple Backends**: From unlimited software to constrained hardware models
- **Comprehensive Testing**: Causal behavior testing with ≤5 nodes, ≤3 edges per case
- **Unified Interface**: Same commands work across CLI, REST API, and programmatic use
- **Hardware Insights**: Quantitative metrics revealing memory, timing, and architectural trade-offs

## 🏗️ Architecture

### Core Components

```
crates/
├── mhg-testkit/     # Shared test harness + toy instances
├── toy-core/        # Unified command dispatcher + RunSummary
├── toy-cli/         # Full-featured CLI with clap subcommands
├── toy-api/         # REST API server with axum
├── core/            # Fundamental IDs, epochs, traits
├── hypergraph/      # Dynamic/frozen graph data structures
├── manifold/        # Geometric operations + spatial indexing
├── bridge/          # Gating, caching, footprints
├── exec/            # Event-driven execution engine
├── analysis/        # Graph analysis algorithms
└── verify/          # Determinism verification + replay
```

### Execution Flow

1. **Command Dispatch**: Unified `Command` enum processed by `toy_core::execute()`
2. **Instance Loading**: Toy instances loaded from `mhg_testkit`
3. **Backend Selection**: Choose from 5 execution models
4. **Deterministic Run**: Produce comprehensive `RunSummary` with metrics
5. **Result Serialization**: Consistent JSON output across all interfaces

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ with cargo
- Optional: axum for REST API (enabled with `--features api`)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd mhg-engine

# Build all crates
cargo build --release

# Run tests
cargo test
```

### Quick Start

```bash
# List available toy instances
cargo run --bin toy-cli -- list

# Run a triangle attractor simulation
cargo run --bin toy-cli -- run --instance triangle_attractor_v0 --backend sw

# Start REST API server
cargo run --bin toy-api -- --addr 127.0.0.1:8080
```

## 🎛️ Command Architecture

The engine uses a unified command model that works identically across CLI, REST API, and programmatic interfaces.

### Commands

- **`ListInstances`**: Discover available toy instances
- **`RunInstance`**: Execute a named instance with backend selection
- **`Replay`**: Deterministic replay from saved snapshots

### Backend Kinds

| Backend | Description | Use Case |
|---------|-------------|----------|
| `SoftwareIdeal` | Unlimited resources baseline | Reference semantics |
| `ToyHardware` | Fixed-point arithmetic + memory counters | Hardware modeling |
| `ToyGescFabric` | 64-bit events + credit flow control | Event-driven architectures |
| `ToyGescBarrier` | Two-phase execution + structural barriers | Deterministic state changes |
| `ToyGesaLoci` | Field/constraint/topology loci | Intelligent adaptive systems |
| `ToyMultiLane` | Multi-lane execution with conflict detection | Parallel processing |

### RunSummary Structure

Every execution produces a comprehensive `RunSummary` containing:

```rust
RunSummary {
    meta: RunMeta,           // Instance/solver metadata
    hashes: RunHashes,       // Determinism anchors
    counts: RunCounts,       // Execution metrics
    checkpoints: Vec<..>,    // Periodic state snapshots
    preview: RunPreview,     // Trace samples
    properties: Vec<..>,     // Built-in property checks
}
```

## 💻 Dual Front-Ends

### CLI Interface (`toy-cli`)

Full-featured command-line interface with clap subcommands:

```bash
# List instances
toy-cli list

# Run with specific backend and tracing
toy-cli run --instance triangle_attractor_v0 --backend gesc-barrier --trace

# Custom tick count
toy-cli run --instance triangle_attractor_v0 --ticks 100 --backend hw
```

### REST API (`toy-api`)

Lightweight HTTP server exposing the command interface:

```bash
# Start server
toy-api --addr 127.0.0.1:8080

# POST commands
curl -X POST http://localhost:8080/v1/command \
  -H "Content-Type: application/json" \
  -d '{
    "RunInstance": {
      "instance": "triangle_attractor_v0",
      "backend": "ToyHardware",
      "ticks": 50,
      "trace": true
    }
  }'
```

## 🔬 Hardware Exploration Backends

Each backend provides different execution semantics and quantitative metrics for hardware design.

### Software Ideal (`sw`)

**Characteristics:**
- Unlimited precision floating-point
- Zero memory constraints
- Instantaneous operations

**Use Case:** Reference semantics for correctness validation

### Toy Hardware (`hw`)

**Characteristics:**
- 16.16 fixed-point arithmetic
- Bounded memory arrays (32 nodes/edges max)
- Cycle-accurate execution counting

**Metrics:** Memory reads/writes, cycle counts, allocation tracking

### GESC Fabric (`gesc-fabric`)

**Characteristics:**
- 64-bit event packets with source/destination scopes
- Credit-based flow control per destination
- Payload tables for complex data

**Event Packet Structure:**
```
63:46 src_id (18 bits) | 45:28 dst_scope (18 bits) |
27:24 class (4 bits)  | 23:8 timestamp (16 bits)   |
7:4 QoS (4 bits)     | 3:0 payload_id (4 bits)
```

**Metrics:** Event throughput, credit utilization, packet routing efficiency

### GESC Barrier (`gesc-barrier`)

**Characteristics:**
- Two-phase execution: compute phase + structural barrier
- Deterministic intent resolution
- Topology changes applied atomically

**Execution Phases:**
1. **Compute**: Process GEOM/FIELD events, queue TOPO intents
2. **Barrier**: Sort intents deterministically, apply changes

**Metrics:** Barrier frequency, intent queue depth, topology change patterns

### GESA Loci (`gesa-loci`)

**Characteristics:**
- Adaptive loci for different computational roles
- Field loci: GLI-style scalar field dynamics
- Topology loci: Stress monitoring with split intents
- Constraint loci: Geometric invariant enforcement

**Locus Types:**
```rust
enum LocusKind {
    Field,      // Scalar field evolution
    Topology,   // Edge stress monitoring
    Constraint, // Geometric invariant checks
}
```

**Metrics:** Locus utilization patterns, adaptation efficiency, constraint satisfaction rates

### Multi-Lane (`multi-lane`)

**Characteristics:**
- Parallel lane execution with conflict detection
- Memory access conflict resolution
- Utilization tracking per lane

**Conflict Types:**
- Node write conflicts (same node modified by multiple lanes)
- Edge read conflicts (same edge accessed by multiple lanes)

**Metrics:** Lane utilization, conflict rates, parallel efficiency

## 📊 Hardware Design Insights

Each backend reveals different architectural trade-offs through quantitative metrics.

### Memory Bandwidth Analysis

```
Node Memory Access Patterns:
- Read:3 per gate evaluation (x,y,z coordinates)
- Write:3 per node movement (delta application)
- Pattern: Read-heavy for gating, write-heavy for movement
```

### Queue Sizing Requirements

```
Event Queue Depth Requirements:
- Toy Hardware: 64 events (fixed)
- GESC Fabric: Variable based on credit limits
- Multi-lane: Per-lane queues with overflow tracking
```

### Timing Characteristics

```
Execution Time Breakdown:
- Gate evaluation: O(active_edges)
- Geodesic computation: O(1) per evaluation
- Apply phase: O(moved_nodes)
- Barrier resolution: O(pending_topology_changes)
```

### Determinism Guarantees

- **Bitwise**: Same inputs → identical memory contents
- **Geometric**: Within epsilon tolerance for floating-point
- **Structural**: Identical topology evolution

## 🧪 Testing Infrastructure

### Core Philosophy

**Test causal behavior, not implementations.** Every failure is reproducible with ≤5 nodes and ≤3 edges.

### Test Categories

```
tests/
├── demo/           # First-class narratable examples
├── causality/      # Non-firing guarantees, sufficiency
├── freeze/         # Graph freezing determinism
├── geometry/       # Spatial index correctness
├── exec/           # Event queue ordering
├── determinism/    # Replay equivalence
├── metamorphic/    # Invariance under transformation
├── perf/           # Allocation/complexity guards
└── property/       # Structural integrity (CSR bounds)
```

### Demo Test Structure

```rust
#[derive(Clone, Debug)]
struct DemoCase {
    name: &'static str,
    points: Vec<Point2>,
    edges: Vec<(Vec<NodeIx>, NodeIx)>, // tails → head
    eps: f32,
    expected_fired: Vec<EdgeIx>,
    expected_moved: Vec<NodeIx>,
    expected_points: Option<Vec<Point2>>,
}
```

## 🛠️ Development

### Building

```bash
# Full workspace build
cargo build --release

# Individual crate development
cargo build -p mhg-testkit
cargo build -p toy-core
```

### Testing

```bash
# Run all tests
cargo test

# Test specific crate
cargo test -p mhg-testkit

# Run performance benchmarks
cargo bench
```

### Adding New Backends

1. Add `BackendKind` variant to `toy_core::command`
2. Implement execution logic in `run_toy_instance_and_collect_summary`
3. Add CLI mapping in `toy-cli`
4. Update documentation

### Adding New Toy Instances

1. Create instance in `mhg_testkit::toy_instance`
2. Add to instance list in `toy_core::execute`
3. Add CLI mapping if needed

## 📚 Documentation

- **[Core Infrastructure](CORE_INFRASTRUCTURE.md)**: mhg-testkit, test suite, reproducible failures
- **[Command Architecture](COMMAND_ARCHITECTURE.md)**: toy-core, unified dispatcher, backend abstraction
- **[Dual Front-Ends](DUAL_FRONTENDS.md)**: toy-cli and toy-api usage patterns
- **[Hardware Backends](HARDWARE_BACKENDS.md)**: 5 execution models with detailed explanations
- **[Hardware Insights](HARDWARE_INSIGHTS.md)**: Metrics, performance analysis, design trade-offs
- **[Usage Guide](USAGE_GUIDE.md)**: Practical examples and integration patterns
- **[Architecture](ARCHITECTURE.md)**: Detailed technical architecture
- **[Testing Strategy](TESTING_STRATEGY.md)**: Comprehensive testing approach
- **[Rust Best Practices](RUST_BEST_PRACTICES.md)**: Language-specific guidelines

## 🤝 Contributing

1. Follow the testing strategy: all bugs reproducible with ≤5 nodes, ≤3 edges
2. Add demo tests for new functionality
3. Maintain deterministic execution across backends
4. Update documentation for new features

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🔬 Research Context

This engine explores the intersection of geometric constraints and graph-based computation, with applications in:
- Hardware architecture design for spatial computing
- Geometric constraint satisfaction systems
- Adaptive mesh refinement algorithms
- Multi-physics simulation frameworks