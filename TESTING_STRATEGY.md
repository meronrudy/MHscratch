# Testing Strategy: Causal Behavior and Determinism

## Core Philosophy

**Test causal behavior, not implementations.** Every test should assert what must move, what must not, and why.

## 1. Demo-Style Tests as First-Class Citizens

Small, narratable systems are the primary test unit. These tests answer:

-   Which edges fired?
-   Which nodes moved?
-   Which nodes stayed fixed?
-   What geometry changed?

### Example: Table-Driven Demo Tests

```rust
struct DemoCase {
    name: &'static str,
    points: Vec<Point2>,
    edges: Vec<(&'static [NodeIx], NodeIx)>,
    eps: f32,
    expected_fired: Vec<EdgeIx>,
    expected_positions: Vec<Point2>,
}

fn run_demo(case: &DemoCase) {
    // setup manifold + graph
    // run one step
    // assert fired edges
    // assert final geometry
}
```

## 2. Causality Tests

### A. Non-Firing Guarantees

If a gate condition is false, nothing downstream moves. Run with conditions just outside the threshold.

### B. Minimal Sufficiency Tests

The minimum change needed to fire an edge is enough. This catches accidental hysteresis, stale caches, and floating-point drift.

### C. No Side-Effects Tests

Only the head of a firing edge may change. Critical for parallelism.

## 3. Trace-Based Testing

Assert on the trace shape, not just results, to ensure explanations remain intrinsic.

```rust
assert_eq!(trace.fired_edges, vec![e0, e2]);
assert!(trace.geodesic_calls <= 1);
```

## 4. Freeze Equivalence Tests

-   **Full Freeze Equivalence**: `freeze_checked` on the same dynamic graph produces bit-for-bit identical frozen graphs.
-   **Incremental vs. Full Freeze**: A full freeze and an incremental freeze from the same state produce identical results.

## 5. Geometry–Topology Boundary Tests

-   **Epoch Invalidation**: Changes to the manifold must invalidate the gate cache.
-   **Spatial Index Correctness**: Brute-force neighbor search and spatial index search must return identical sets.

## 6. Determinism Tests

-   **Replay Equivalence**: Replaying a trace from a snapshot must reproduce the final state bit-for-bit.
-   **Seeded Fuzz with Replay**: Generate random small graphs, run execution, save the trace, and assert that replay matches.

## 7. Metamorphic Tests

Check invariants under transformation.

-   **Translation Invariance**: Translate all points, run, translate back. Geometry should match.
-   **Node Renumbering Invariance**: Permute node IDs, run, undo permutation. Results should match.

## 8. Performance Regression Tests

-   **Allocation Guards**: Assert zero allocations in hot paths and bounded allocations during freeze.
-   **Complexity Guards**: Ensure freeze time scales linearly and gating time is proportional to active edges.

## 9. Property Tests (Tier 3+)

Use for CSR integrity, no panics on random valid graphs, and remap correctness. Avoid for geometry or execution semantics, which require narrative tests.

## 10. Test Taxonomy

Organize tests explicitly by their purpose.

```
tests/
  demo/
  causality/
  freeze/
  geometry/
  exec/
```

## The One Rule

**Every bug should be reproducible with ≤ 5 nodes and ≤ 3 edges.** If a bug requires a giant graph, extract the minimal causal core and add it as a demo-style test before fixing it.
