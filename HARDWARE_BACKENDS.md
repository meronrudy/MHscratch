# Hardware Exploration Backends

The MHG engine provides 5 distinct execution backends that explore different hardware architectures for geometric hypergraph processing. Each backend implements the same semantic behavior while revealing different architectural trade-offs through quantitative metrics and execution patterns.

## 🎯 Backend Overview

| Backend | Key Insight | Primary Constraints | Metrics Revealed |
|---------|-------------|-------------------|------------------|
| **Software Ideal** | Reference semantics | None (unlimited resources) | Baseline performance |
| **Toy Hardware** | Fixed-point arithmetic | Memory bounds, precision limits | Memory access patterns |
| **GESC Fabric** | Event-driven processing | Credit flow control, packet routing | Communication overhead |
| **GESC Barrier** | Structural consistency | Atomic topology changes | Synchronization costs |
| **GESA Loci** | Adaptive intelligence | Dynamic locus behavior | Learning/adaptation patterns |
| **Multi-Lane** | Parallel execution | Memory conflicts, lane coordination | Parallelization limits |

## 🔬 Software Ideal (`BackendKind::SoftwareIdeal`)

### Overview

The Software Ideal backend provides unlimited resources and serves as the reference implementation for semantic correctness.

### Characteristics

- **Precision**: Full double-precision floating-point
- **Memory**: Unlimited heap allocation
- **Timing**: Zero-cost operations
- **Determinism**: Bitwise identical results

### Use Cases

- **Correctness Validation**: Verify other backends against reference semantics
- **Performance Baseline**: Measure overhead of constrained implementations
- **Development**: Rapid prototyping without hardware considerations

### Execution Model

```rust
fn run_toy_instance_and_collect_summary(inst: &ToyInstance, /* ... */) -> RunSummary {
    // Direct call to mhg-testkit reference implementation
    run_toy_instance(inst);  // Floating-point, unlimited resources

    RunSummary {
        meta: RunMeta {
            determinism_level: DeterminismLevel::LocalBitwise,
            // ...
        },
        // No hardware counters (unlimited resources)
        counts: RunCounts { hw_counts: None, /* ... */ },
        // ...
    }
}
```

### Metrics Provided

- **Execution Time**: True computational cost without hardware artifacts
- **Memory Usage**: Natural heap allocation patterns
- **Result Quality**: Reference floating-point precision

## 🔢 Toy Hardware (`BackendKind::ToyHardware`)

### Overview

The Toy Hardware backend models constrained fixed-point arithmetic with bounded memory arrays, simulating embedded hardware limitations.

### Characteristics

- **Arithmetic**: 16.16 fixed-point Q-format
- **Memory**: Fixed-size arrays (32 nodes/edges maximum)
- **Precision**: Limited fractional accuracy
- **Counters**: Comprehensive memory access tracking

### Fixed-Point Implementation

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Q16_16(pub i32);

impl Q16_16 {
    pub const ONE: Q16_16 = Q16_16(1 << 16);  // 1.0 in fixed-point

    pub fn from_float(f: f32) -> Self {
        Q16_16((f * (1 << 16) as f32) as i32)
    }

    pub fn to_float(self) -> f32 {
        self.0 as f32 / (1 << 16) as f32
    }

    pub fn mul(self, other: Q16_16) -> Q16_16 {
        Q16_16(((self.0 as i64 * other.0 as i64) >> 16) as i32)
    }
}
```

### Memory Architecture

```rust
pub struct NodeMem {
    pub x: [Q16_16; N_NODES_MAX],  // SoA layout for SIMD potential
    pub y: [Q16_16; N_NODES_MAX],
    pub z: [Q16_16; N_NODES_MAX],
    pub count: usize,
    pub reads: u64,                // Access counters
    pub writes: u64,
}
```

### Execution Model

```rust
impl ToyHw {
    pub fn step_cycle(&mut self) {
        self.cycle += 1;

        // Evaluate all gates
        for edge in 0..self.edge_mem.count {
            if self.gate_unit.eval(/* ... */) {
                // Compute delta
                let (node, delta) = self.geom_unit.eval(/* ... */);
                // Apply movement
                self.apply_unit.apply(/* ... */);
            }
        }
    }
}
```

### Metrics Revealed

#### Memory Access Patterns
```
Node Memory Operations:
- Gate Evaluation: 3 reads per active edge (centroid calculation)
- Geometry Computation: 6 reads + 3 writes per fired edge
- Total per Step: O(active_edges * nodes_per_edge)
```

#### Precision Trade-offs
- **Range**: ±32K integer part, 1/65536 fractional precision
- **Accumulation Errors**: Fixed-point drift in iterative calculations
- **Comparison**: Epsilon-based validation vs. floating-point reference

#### Performance Characteristics
- **Deterministic**: Same inputs → identical fixed-point operations
- **Predictable**: Bounded memory access, no heap allocation
- **Measurable**: Cycle-accurate execution counting

## 📡 GESC Fabric (`BackendKind::ToyGescFabric`)

### Overview

The GESC (Geometry + Events + Structure + Constraints) Fabric backend models event-driven architectures with credit-based flow control and structured event packets.

### Event Packet Architecture

GESC uses 64-bit event packets encoding geometric updates, field changes, and structural operations:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Gescevent {
    pub raw: u64,
}

impl Gescevent {
    pub fn new(
        src_id: u32,      // Source edge/node ID
        dst_scope: u32,   // Destination scope (node/locus)
        class: u8,        // GEOM_CLASS, FIELD_CLASS, TOPO_CLASS
        tstamp: u16,      // Timestamp for ordering
        qos: u8,          // Quality of service level
        payload_id: u8,   // Payload table index
    ) -> Self {
        // Pack fields into 64-bit word
    }
}
```

#### Packet Bit Layout
```
63:46 src_id (18 bits) | 45:28 dst_scope (18 bits) |
27:24 class (4 bits)  | 23:8 timestamp (16 bits)   |
7:4 QoS (4 bits)     | 3:0 payload_id (4 bits)
```

### Credit-Based Flow Control

```rust
pub struct CreditTable {
    pub credits: [u16; N_NODES_MAX],
    pub max_credits: u16,
    pub credit_consumes: u64,  // Metrics
    pub credit_releases: u64,
    pub credit_denies: u64,
}

impl CreditTable {
    pub fn consume(&mut self, scope: u32) -> bool {
        if self.credits[scope as usize] > 0 {
            self.credits[scope as usize] -= 1;
            self.credit_consumes += 1;
            true
        } else {
            self.credit_denies += 1;
            false
        }
    }
}
```

### Event Queue Implementation

```rust
pub struct EventQueue {
    pub events: [Gescevent; EVENT_QUEUE_DEPTH],  // Fixed: 64 events
    pub head: usize,
    pub tail: usize,
    pub count: usize,
    pub pushes: u64,
    pub pops: u64,
    pub overflows: u64,
}
```

### Execution Model

```rust
impl ToyHw {
    pub fn step_tick(&mut self) {
        self.cycle += 1;

        // Process all available events
        while let Some(event) = self.event_q.pop() {
            match event.class() {
                GEOM_CLASS => self.process_geom_event(event),
                FIELD_CLASS => self.process_field_event(event),
                TOPO_CLASS => self.queue_topo_intent(event),
                _ => {} // Ignore unknown
            }

            // Release credit after processing
            self.credit_table.release(event.dst_scope());
        }
    }
}
```

### Metrics Revealed

#### Communication Overhead
- **Event Throughput**: Events processed per cycle
- **Credit Utilization**: Credit consumption/release patterns
- **Queue Pressure**: Push/pop ratios, overflow frequency

#### Packet Routing Efficiency
- **Destination Hotspots**: Which scopes receive most events
- **QoS Distribution**: Priority-based event handling
- **Payload Utilization**: How many of 16 payload slots used

#### Flow Control Effectiveness
- **Blocking Frequency**: How often credits prevent sending
- **Backpressure Patterns**: When and where flow control activates

## 🚧 GESC Barrier (`BackendKind::ToyGescBarrier`)

### Overview

The GESC Barrier backend implements two-phase execution with deterministic structural barriers, modeling architectures that must maintain topological consistency.

### Two-Phase Execution Model

#### Phase 1: Compute Phase
- Process GEOM/FIELD events
- Queue structural change intents
- No topology modifications allowed

#### Phase 2: Structural Barrier
- Deterministically resolve conflicting intents
- Apply topology changes atomically
- Advance graph epoch

### Intent System

```rust
#[derive(Clone, Debug)]
pub enum TopoIntentKind {
    AddEdge { tails: [NodeIx; 4], head: NodeIx, arity: u8 },
    RemoveEdge { edge: EdgeIx },
    SplitEdge { edge: EdgeIx },
}

#[derive(Clone, Debug)]
pub struct TopoIntent {
    pub edge_local_id: u16,
    pub kind: TopoIntentKind,
}
```

### Barrier Implementation

```rust
impl ToyHw {
    pub fn step_tick(&mut self) {
        self.cycle += 1;

        // Phase 1: Compute (read-only topology)
        self.compute_phase();

        // Phase 2: Structural barrier (topology changes)
        self.apply_structural_barrier();
    }

    fn compute_phase(&mut self) {
        while let Some(event) = self.event_q.pop() {
            match event.class() {
                GEOM_CLASS | FIELD_CLASS => {
                    self.process_geom_field_event(event);
                }
                TOPO_CLASS => {
                    self.queue_topo_intent(event);
                }
            }
            self.credit_table.release(event.dst_scope());
        }
    }

    fn apply_structural_barrier(&mut self) {
        if self.pending_topo.intents.is_empty() {
            return;
        }

        // Deterministic resolution: sort by edge_local_id
        self.pending_topo.intents.sort_by_key(|intent| intent.edge_local_id);

        // Apply changes (toy: simplified - just clear)
        self.pending_topo.intents.clear();
    }
}
```

### Metrics Revealed

#### Synchronization Costs
- **Barrier Frequency**: How often topology changes occur
- **Intent Queue Depth**: Pending structural operations
- **Resolution Overhead**: Cost of deterministic conflict resolution

#### Consistency Guarantees
- **Epoch Advancement**: Graph epoch changes per step
- **Atomicity**: All-or-nothing topology updates
- **Determinism**: Same intents → same resolution order

#### Performance Trade-offs
- **Throughput**: Reduced by barrier synchronization
- **Latency**: Bounded by worst-case intent processing
- **Scalability**: How barrier costs scale with system size

## 🧠 GESA Loci (`BackendKind::ToyGesaLoci`)

### Overview

The GESA (Geometry + Events + Structure + Adaptive) Loci backend implements intelligent loci that adapt their behavior based on system dynamics, modeling self-organizing hardware architectures.

### Locus Architecture

Loci are processing elements that monitor and respond to system behavior:

```rust
#[derive(Clone, Debug)]
pub enum LocusKind {
    Field,      // GLI-style scalar field evolution
    Topology,   // Edge stress monitoring
    Constraint, // Geometric invariant enforcement
}

#[derive(Clone, Debug)]
pub struct Locus {
    pub kind: LocusKind,
    pub params: LocusParams,
}

#[derive(Clone, Debug)]
pub enum LocusParams {
    Field {
        v: Q16_16,           // Current field value
        decay: Q16_16,       // Temporal decay factor
        bias: Q16_16,        // Baseline bias
        threshold: Q16_16,   // Trigger threshold
    },
    Topology {
        stress_threshold: Q16_16,
    },
    Constraint {
        max_dist: Q16_16,
    },
}
```

### Field Locus Dynamics

GLI-style (Generalized Linear Integrator) field evolution:

```rust
fn process_field_locus(&mut self, locus: &mut Locus, input: Q16_16) {
    if let LocusParams::Field { v, decay, bias, threshold } = &mut locus.params {
        // v = decay * v + input + bias
        *v = decay.mul(*v).add(input).add(*bias);

        // Threshold trigger
        if v.0 > threshold.0 {
            self.trigger_locus_event(locus, *v);
        }
    }
}
```

### Topology Locus Behavior

```rust
fn check_edge_stress(&mut self, edge: usize, threshold: Q16_16) -> bool {
    // Compute edge "stress" as centroid-to-head distance
    let stress_metric = compute_centroid_distance(edge);
    stress_metric.0 > threshold.0
}
```

### Constraint Locus Behavior

```rust
fn check_constraint_violation(&mut self, node: usize, max_dist: Q16_16) -> bool {
    // Distance from origin constraint
    let dist_sq = self.node_mem.get_point(node).dist_sq_from_origin();
    dist_sq.0 > max_dist.0
}
```

### Execution Model

```rust
impl ToyHw {
    pub fn step_tick(&mut self) {
        self.cycle += 1;

        // Process events through loci
        while let Some(event) = self.event_q.pop() {
            self.process_event_through_loci(event);
            self.credit_table.release(event.dst_scope());
        }

        // Apply structural barrier
        self.apply_structural_barrier();
    }
}
```

### Metrics Revealed

#### Adaptation Patterns
- **Locus Utilization**: Which loci types are most active
- **Trigger Frequency**: How often loci generate responses
- **Field Evolution**: Stability/convergence of field dynamics

#### Learning Behavior
- **Stress Detection**: Topology monitoring effectiveness
- **Constraint Satisfaction**: How well invariants are maintained
- **Response Patterns**: Loci interaction and coordination

#### Intelligence Characteristics
- **Emergent Behavior**: Self-organizing patterns
- **Stability**: How well the system converges
- **Adaptability**: Response to changing conditions

## ⚡ Multi-Lane (`BackendKind::ToyMultiLane`)

### Overview

The Multi-Lane backend explores parallel execution with conflict detection, modeling multi-core or SIMD architectures with memory access coordination.

### Lane Architecture

```rust
pub struct ToyHw {
    // ... existing fields ...
    pub lane_utilization: [u64; LANES],  // LANES = 2
    pub node_write_conflicts: u64,
    pub edge_read_conflicts: u64,
}
```

### Parallel Execution Model

```rust
impl ToyHw {
    pub fn step_lanes(&mut self) {
        self.cycle += 1;

        // Collect events for parallel processing
        let mut lane_events = Vec::new();
        for _ in 0..LANES {
            if let Some(event) = self.event_q.pop() {
                lane_events.push(event);
            } else {
                break;
            }
        }

        // Track utilization
        for i in 0..lane_events.len() {
            self.lane_utilization[i] += 1;
        }

        // Simulate parallel processing with conflict detection
        let mut lane_operations = Vec::new();
        for (lane_idx, event) in lane_events.into_iter().enumerate() {
            if let Some(op) = self.simulate_lane_operation(lane_idx, event) {
                lane_operations.push(op);
            }
        }

        // Resolve conflicts
        self.resolve_lane_conflicts(&mut lane_operations);

        // Apply non-conflicting operations
        for op in lane_operations {
            if let LaneOperation::GeomField { event, .. } = op {
                self.process_geom_field_event(event);
            }
        }

        // Apply structural barrier
        self.apply_structural_barrier();
    }
}
```

### Conflict Detection

```rust
fn resolve_lane_conflicts(&mut self, operations: &mut Vec<LaneOperation>) {
    // Group operations by memory access
    let mut node_writes: HashMap<u32, Vec<usize>> = HashMap::new();
    let mut edge_reads: HashMap<u32, Vec<usize>> = HashMap::new();

    for (op_idx, op) in operations.iter().enumerate() {
        match op {
            LaneOperation::GeomField { dst_scope, event } => {
                // Track write conflicts
                node_writes.entry(*dst_scope).or_insert(Vec::new()).push(op_idx);

                // Track read conflicts (less critical)
                if event.src_id() < self.edge_mem.count as u32 {
                    edge_reads.entry(event.src_id()).or_insert(Vec::new()).push(op_idx);
                }
            }
        }
    }

    // Resolve write conflicts (multiple lanes writing same node)
    for (_node, lane_indices) in node_writes {
        if lane_indices.len() > 1 {
            self.node_write_conflicts += 1;
            // Serialize: keep first, mark others conflicted
            for &idx in &lane_indices[1..] {
                operations[idx] = LaneOperation::Conflicted;
            }
        }
    }

    // Track read conflicts
    for (_edge, lane_indices) in edge_reads {
        if lane_indices.len() > 1 {
            self.edge_read_conflicts += 1;
        }
    }

    // Remove conflicted operations
    operations.retain(|op| !matches!(op, LaneOperation::Conflicted));
}
```

### Metrics Revealed

#### Parallelization Efficiency
- **Lane Utilization**: How evenly work is distributed
- **Conflict Rates**: Frequency of memory access conflicts
- **Serialization Overhead**: How often parallel work becomes serial

#### Scalability Characteristics
- **Amdahl's Law**: Serial fraction vs. parallel speedup
- **Memory Contention**: Hotspots that limit parallelism
- **Coordination Costs**: Overhead of conflict resolution

#### Architecture Trade-offs
- **Throughput**: Potential parallel speedup
- **Determinism**: How conflicts affect reproducible execution
- **Complexity**: Hardware cost of conflict detection/coordination

## 🔄 Backend Comparison

### Execution Semantics

| Backend | Event Processing | Topology Changes | Parallelism |
|---------|------------------|------------------|-------------|
| Software Ideal | Direct function calls | Immediate | N/A |
| Toy Hardware | Synchronous per tick | Immediate | Single-threaded |
| GESC Fabric | Async event packets | Queued intents | Single-threaded |
| GESC Barrier | Two-phase (compute + barrier) | Atomic barriers | Single-threaded |
| GESA Loci | Locus-mediated | Intent-based | Single-threaded |
| Multi-Lane | Parallel with conflicts | Intent-based | Multi-lane |

### Determinism Guarantees

- **Bitwise**: Software Ideal, Toy Hardware
- **Geometric ε**: All backends (fixed-point vs float differences)
- **Structural**: All backends (same topology evolution)
- **Event Order**: All backends (deterministic event processing)

### Hardware Insights

Each backend reveals different architectural concerns:

- **Software Ideal**: Algorithmic complexity, memory usage patterns
- **Toy Hardware**: Precision limits, memory access efficiency
- **GESC Fabric**: Communication overhead, flow control effectiveness
- **GESC Barrier**: Synchronization costs, consistency models
- **GESA Loci**: Adaptive behavior, self-organization patterns
- **Multi-Lane**: Parallelization limits, memory contention

## 🚀 Backend Selection Guide

### For Research Questions

| Research Goal | Recommended Backend | Why |
|---------------|---------------------|-----|
| Algorithm Correctness | Software Ideal | Reference semantics |
| Memory Architecture | Toy Hardware | Access pattern analysis |
| Event Systems | GESC Fabric | Communication modeling |
| Consistency Models | GESC Barrier | Synchronization design |
| Adaptive Systems | GESA Loci | Intelligent behavior |
| Parallel Processing | Multi-Lane | Scalability analysis |

### For Hardware Design

| Hardware Target | Primary Backend | Secondary Backend |
|----------------|-----------------|-------------------|
| Embedded System | Toy Hardware | GESC Fabric |
| Event-Driven ASIC | GESC Fabric | GESC Barrier |
| Multi-Core CPU | Multi-Lane | Software Ideal |
| Neuromorphic HW | GESA Loci | GESC Fabric |
| Safety-Critical | GESC Barrier | Toy Hardware |

## 📊 Quantitative Analysis

### Performance Benchmarks

```rust
// Example: Memory access comparison across backends
fn benchmark_memory_access() {
    let instance = create_test_instance(32, 64); // 32 nodes, 64 edges

    let results = vec![
        ("Software Ideal", run_on_backend(&instance, BackendKind::SoftwareIdeal)),
        ("Toy Hardware", run_on_backend(&instance, BackendKind::ToyHardware)),
        ("GESC Fabric", run_on_backend(&instance, BackendKind::ToyGescFabric)),
    ];

    for (name, result) in results {
        println!("{}: {} cycles, {} mem ops",
            name,
            result.hw_counts.as_ref().map(|h| h.cycles).unwrap_or(0),
            result.hw_counts.as_ref().map(|h| h.node_mem_reads + h.node_mem_writes).unwrap_or(0)
        );
    }
}
```

### Scaling Analysis

```rust
// Example: Scaling behavior with problem size
fn analyze_scaling() {
    for size in [8, 16, 32, 64] {
        let instance = generate_scaling_instance(size);

        // Measure key metrics vs. problem size
        let result = run_on_backend(&instance, BackendKind::ToyHardware);

        println!("Size {}: cycles={}, conflicts={}",
            size,
            result.hw_counts.as_ref().unwrap().cycles,
            result.hw_counts.as_ref().unwrap().node_write_conflicts
        );
    }
}
```

This comprehensive backend architecture enables systematic exploration of hardware design trade-offs for geometric hypergraph processing, with each backend providing unique insights into different aspects of system design.