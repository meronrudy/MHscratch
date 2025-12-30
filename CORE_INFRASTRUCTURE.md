# Core Infrastructure

This document details the core infrastructure components that form the foundation of the MHG engine: the shared test harness (`mhg-testkit`), comprehensive test suite, and the principle of reproducible failures.

## 🎯 mhg-testkit: Shared Test Harness

The `mhg-testkit` crate provides the shared infrastructure for testing, benchmarking, and validation across all engine components.

### Key Components

#### Toy Instances (`toy_instance.rs`)

Predefined test cases that can be executed across all backends:

```rust
pub struct ToyInstance {
    pub name: &'static str,
    pub points_xy: Vec<Point2>,                    // 2D authoring, z=0
    pub edges: Vec<ToyEdge>,                       // Hyperedges with footprints
    pub schedule: ToySchedule,                     // Execution timeline
}

pub struct ToyEdge {
    pub tails: Vec<NodeIx>,                        // Input nodes
    pub head: NodeIx,                             // Output node
    pub footprint: ToyFootprint,                  // Geometric gating parameters
}
```

#### Run One Step (`run_one_step()`)

The canonical execution interface that all backends implement:

```rust
pub fn run_one_step(case: &DemoCase) -> RunResult {
    // 1) Build HypergraphDyn from case.edges
    // 2) Freeze to immutable representation
    // 3) Initialize ManifoldStore from case.points
    // 4) Execute single step with gate evaluation
    // 5) Return fired edges, moved nodes, final geometry
}
```

#### Assertion Helpers

Deterministic comparison utilities:

```rust
pub fn assert_points_close(got: &[Point2], exp: &[Point2], eps: f32)

pub fn assert_set_eq(mut got: Vec<u32>, mut exp: Vec<u32>)
```

### Demo Case Structure

Table-driven test cases that define complete execution scenarios:

```rust
#[derive(Clone, Debug)]
pub struct DemoCase {
    pub name: &'static str,
    pub points: Vec<Point2>,
    pub edges: Vec<(Vec<NodeIx>, NodeIx)>, // tails → head
    pub eps: f32,

    // Expectations (what must happen)
    pub expected_fired: Vec<EdgeIx>,
    pub expected_moved: Vec<NodeIx>,
    pub expected_fixed: Vec<NodeIx>,
    pub expected_points: Option<Vec<Point2>>, // Exact geometry check
}
```

## 🧪 Complete Test Suite: 9 Categories

The test suite is organized into explicit categories covering all aspects of the engine, with each category focusing on different correctness dimensions.

### 1. Demo Tests (First-Class Citizens)

**Location:** `tests/demo/`

**Purpose:** Narratable examples that demonstrate complete system behavior.

#### Demo A: Single Edge Propagation
```
2 nodes, 1 edge: [0] → 1
Gate: true
Expected: fired=[0], moved=[1], fixed=[0]
```

#### Demo B: Chain Reaction
```
3 nodes, edges: [0]→1, [1]→2
Step 1: fired=[0], moved=[1], fixed=[0,2]
Step 2: fired=[1], moved=[2], fixed=[0,1]
```

#### Demo C: Fork Pattern
```
Edges: [0]→1, [0]→2
Expected: fired=[0,1], moved=[1,2], fixed=[0]
```

#### Demo D: Diamond Convergence
```
Edges: [0]→2, [1]→2
Expected: fired=[0,1], moved=[2], fixed=[0,1]
Deterministic reduction policy for competing influences
```

#### Demo E: Threshold Boundary
```
Distance = radius + ε
Expected: fired=[], moved=[], fixed=all
```

#### Demo F: Minimal Sufficiency
```
Distance = radius - ε
Expected: fired=[edge], moved=[head], fixed=tails
```

#### Demo G: No Side Effects
```
Single edge fires, only head changes
All other nodes bit-identical
```

#### Demo H: Gate Weight Effects
```
Same topology, different gate weights
Same fired edges, different movement magnitudes
```

### 2. Causality Tests

**Location:** `tests/causality/`

**Purpose:** Validate that effects follow causes correctly.

#### Non-Firing Guarantees
```rust
// When gate condition is false
assert_eq!(result.fired_edges, []);
assert_eq!(result.moved_nodes, []);
assert_eq!(result.manifold_epoch, initial_epoch); // No change
```

#### Minimal Sufficiency Tests
```rust
// Just barely meets threshold
let threshold_minus_eps = threshold - 1e-6;
let threshold_plus_eps = threshold + 1e-6;

// Should fire
assert!(gate_active(threshold_minus_eps));
// Should not fire
assert!(!gate_active(threshold_plus_eps));
```

#### No Side Effects
```rust
let initial_state = snapshot_state();
run_single_edge_fire();
let final_state = snapshot_state();

// Only head node coordinates changed
for node in 0..total_nodes {
    if node != fired_edge.head {
        assert_eq!(initial_state[node], final_state[node]);
    }
}
```

### 3. Trace-Based Testing

**Location:** `tests/trace/`

**Purpose:** Validate execution explanations remain intrinsic.

```rust
let trace = run_with_trace(case);

// Shape assertions
assert_eq!(trace.fired_edges, vec![0, 2]);
assert!(trace.geodesic_calls <= 1); // Footprint precheck worked
assert_eq!(trace.visited_nodes.len(), 5); // Expected touched set
```

### 4. Freeze Equivalence Tests

**Location:** `tests/freeze/`

**Purpose:** Ensure graph freezing is deterministic and equivalent.

#### Full Freeze Determinism
```rust
let graph = build_complex_graph();
let frozen1 = graph.freeze_full();
let frozen2 = graph.freeze_full();

// Bit-for-bit identical
assert_eq!(frozen1.as_bytes(), frozen2.as_bytes());
```

#### Incremental vs Full Freeze
```rust
let base = graph.freeze_full();
graph.add_edges(new_edges);
let incremental = graph.freeze_delta(base);

// Results identical to fresh full freeze
assert_equivalent_adjacency(base, incremental);
```

### 5. Geometry-Topology Boundary Tests

**Location:** `tests/geometry/`

**Purpose:** Validate spatial index and cache invalidation.

#### Epoch Invalidation
```rust
let cache = GateCache::new(edge_count);
cache.recompute(&frozen, &manifold, graph_epoch);

// Modify manifold
manifold.move_node(node, delta);
assert!(!cache.is_valid_for(epoch_key(graph_epoch, manifold.epoch)));
```

#### Spatial Index Correctness
```rust
let index = SpatialIndex::build(&points);
let query_point = Point3::new(1.0, 2.0, 0.0);
let radius = 0.5;

let index_results = index.radius_query(query_point, radius);
let brute_results = brute_force_radius_query(&points, query_point, radius);

assert_eq!(index_results, brute_results);
```

### 6. Determinism Tests

**Location:** `tests/determinism/`

**Purpose:** Ensure identical inputs produce identical outputs.

#### Replay Equivalence
```rust
let snapshot = engine.snapshot();
let original_hash = hash_state(&manifold);

let replayed_state = Engine::replay(snapshot);
let replay_hash = replayed_state.manifold_hash;

// Bit-for-bit identical
assert_eq!(original_hash, replay_hash);
```

#### Seeded Fuzz with Replay
```rust
// Generate bounded random graph
let seed = 42;
let graph = generate_random_graph(seed, max_nodes=5, max_edges=3);

// Execute and capture
let result1 = run_deterministic(graph.clone());
let snapshot = result1.snapshot;

// Replay from snapshot
let result2 = replay(snapshot);

// Identical final state
assert_eq!(result1.final_hash, result2.final_hash);
```

### 7. Metamorphic Tests

**Location:** `tests/metamorphic/`

**Purpose:** Check invariants under transformation.

#### Translation Invariance
```rust
let original_result = run_case(case);

// Translate all points
let translated_case = translate_case(case, offset);
let translated_result = run_case(translated_case);

// Translate result back
let translated_back = translate_result_back(translated_result, -offset);

// Should match original (within eps)
assert_points_close(original_result.points, translated_back.points, eps);
assert_eq!(original_result.fired_edges, translated_result.fired_edges);
```

#### Node Renumbering Invariance
```rust
let permutation = generate_permutation(node_count);

// Apply permutation to graph
let permuted_case = permute_case(case, permutation);
let permuted_result = run_case(permuted_case);

// Unpermute results
let unpermuted_result = unpermute_result(permuted_result, permutation);

// Should match original
assert_equivalent_results(original_result, unpermuted_result);
```

### 8. Performance Regression Tests

**Location:** `tests/perf/`

**Purpose:** Guard against performance degradation.

#### Allocation Guards
```rust
#[cfg(feature = "alloc_guard")]
{
    ALLOC_COUNT.store(0, Ordering::Relaxed);

    run_hot_path_operation();

    // No allocations in hot path after warmup
    assert_eq!(ALLOC_COUNT.load(Ordering::Relaxed), 0);
}
```

#### Complexity Guards
```rust
let start = Instant::now();
run_freeze_operation(large_graph);
let duration = start.elapsed();

// Should scale linearly with graph size
let expected_max = Duration::from_millis(graph_size as u64);
assert!(duration < expected_max);
```

### 9. Property Tests (Tier 3+)

**Location:** `tests/property/`

**Purpose:** Structural integrity validation.

#### CSR Integrity
```rust
// Offsets are monotone increasing
for i in 1..offsets.len() {
    assert!(offsets[i] >= offsets[i-1]);
}

// Last offset equals indices length
assert_eq!(offsets.last(), indices.len());

// All indices within node bounds
for &idx in &indices {
    assert!(idx < node_count);
}
```

## 🎯 Reproducible Failures: ≤5 Nodes, ≤3 Edges

### Core Principle

**Every bug must be reproducible with tiny graphs.** If a bug requires thousands of nodes, extract the minimal causal core first.

### Failure Analysis Process

1. **Identify Failure**: Bug manifests in large system
2. **Extract Core**: Find minimal node/edge set that reproduces
3. **Create Demo**: Convert to `DemoCase` with expectations
4. **Add Test**: Integrate into test suite
5. **Fix Bug**: Implement fix validated by new test
6. **Verify**: Ensure fix doesn't break existing tests

### Example: Complex Bug → Minimal Demo

**Original:** Bug in 1000-node mesh, failure after 50 steps
**Analysis:** Root cause is edge interaction pattern
**Minimal:** 4 nodes, 3 edges, 2 steps
**Demo Case:**
```rust
DemoCase {
    name: "edge_interaction_bug_v1",
    points: vec![p0, p1, p2, p3], // 4 points
    edges: vec![
        (vec![0], 2),  // 0 → 2
        (vec![1], 2),  // 1 → 2
        (vec![2], 3),  // 2 → 3
    ],
    eps: 1e-6,
    expected_fired: vec![0, 1], // First step
    expected_moved: vec![2],    // Only node 2 moves
    expected_fixed: vec![0, 1, 3],
    // Step 2 expectations...
}
```

### Benefits

- **Fast Debugging**: Tiny cases run in milliseconds
- **Clear Causality**: Easy to understand what should happen
- **Regression Prevention**: Future changes can't break core behavior
- **Documentation**: Test cases serve as specification examples

## 🏗️ Infrastructure Architecture

### Test Execution Flow

```
DemoCase → ToyInstance → Backend Selection → RunResult
    ↓                                                ↓
Assertion Helpers ← Expectations ← Validation ← Comparison
```

### Backend-Agnostic Testing

All tests run against the `run_one_step()` interface, allowing:

- **Same Test**: Different backends validate identical behavior
- **Performance Comparison**: Measure overhead of different implementations
- **Correctness Checking**: Reference backend vs. optimized backends

### Deterministic Test Environment

- **Fixed Seeds**: All random generation uses deterministic seeds
- **No Timing Dependencies**: Tests don't rely on wall clock time
- **Isolated Execution**: Each test runs in clean environment
- **Reproducible Ordering**: Event queues use deterministic priority ordering

## 📊 Quality Metrics

### Test Coverage Goals

- **Demo Tests**: Complete behavioral coverage with narratable examples
- **Causality Tests**: All edge cases for cause→effect relationships
- **Performance Tests**: Regression protection for hot paths
- **Property Tests**: Structural invariants that must always hold

### Failure Classification

- **Logic Bugs**: Wrong edges fired, wrong nodes moved
- **Performance**: Allocation in hot paths, unexpected complexity
- **Determinism**: Different results from same inputs
- **Correctness**: Wrong geometry, wrong topology evolution

### Continuous Validation

Every commit runs the full test suite across all backends to ensure:
- No regressions in existing behavior
- New features work across all execution models
- Performance characteristics remain stable
- Determinism guarantees hold