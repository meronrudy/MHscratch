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

# Testing v2
## 1) Reusable `GoalProofCase`

Create this in `mhg-testkit` as a single “vertical slice” runner. It encodes: **2D authoring → 3D execution**, **two-step co-evolution**, **trace**, **snapshot+replay**, **no allocations (optional)**.

```rust
// mhg-testkit/src/goal_proof.rs

pub type NodeIx = u32;
pub type EdgeIx = u32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 { pub x: f32, pub y: f32 }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 { pub x: f32, pub y: f32, pub z: f32 }

impl From<Point2> for Point3 {
    #[inline(always)]
    fn from(p: Point2) -> Self { Self { x: p.x, y: p.y, z: 0.0 } }
}

#[derive(Clone, Debug)]
pub enum FootprintSpec {
    // v1
    Influence { radius: f32, anchor: Option<NodeIx> },
    // v2 simplex proxy
    Simplex { anchors: Vec<NodeIx>, radius: f32 },
}

#[derive(Clone, Debug)]
pub struct GoalProofCase {
    pub name: &'static str,

    // expressed in 2D, embedded into 3D with z=0
    pub points_xy: Vec<Point2>,

    // minimal higher-order structure
    pub edges: Vec<(Vec<NodeIx>, NodeIx)>,  // tails -> head
    pub footprints: Vec<FootprintSpec>,     // edge-aligned

    // scheduling for the two steps (usually push on tails)
    pub step0_dirty: Vec<NodeIx>,
    pub step1_dirty: Vec<NodeIx>,

    // assertions (goal evidence)
    pub expect_step0_fired: Vec<EdgeIx>,
    pub expect_step1_fired: Vec<EdgeIx>,
    pub expect_step0_moved: Vec<NodeIx>,
    pub expect_step1_moved: Vec<NodeIx>,

    // gate evolution assertions
    pub expect_edge_active_step0: Vec<(EdgeIx, bool)>,
    pub expect_edge_active_step1: Vec<(EdgeIx, bool)>,

    // determinism / replay
    pub expect_bitwise_replay: bool,

    // numeric tolerance for XY comparisons (if used)
    pub eps_xy: f32,
}

#[derive(Clone, Debug)]
pub struct GoalProofResult {
    pub step0_fired: Vec<EdgeIx>,
    pub step1_fired: Vec<EdgeIx>,
    pub step0_moved: Vec<NodeIx>,
    pub step1_moved: Vec<NodeIx>,
    pub final_points: Vec<Point3>,
    pub final_epoch: u64,
    pub snapshot_hash: [u8; 32],
    pub replay_hash: [u8; 32],
}

fn assert_xy_plane(points: &[Point3]) {
    for (i, p) in points.iter().enumerate() {
        debug_assert!(p.z == 0.0, "z-drift at {i}: {:?}", p);
    }
}
```

Runner (generic over your actual substrate types) is below in section 2, because the trait bounds define the “ABI”.

---

## 2) Exact trait bounds this test forces

The point of the trait bounds is to make the test *compile-time enforce* the architectural commitments:

* **dual-state graph**: dynamic build → frozen read-only
* **edge-aligned footprint**: no heap per edge at runtime
* **gate cache with epoch invalidation**
* **event-driven exec**: schedule → compute (read-only) → apply (single writer)
* **trace + snapshot + replay**
* **2D-in-3D invariant** holds through actual manifold storage

### 2.1 Graph + freeze + incidence

```rust
pub trait DynGraph {
    type Frozen: FrozenGraph;

    fn new() -> Self;
    fn add_node(&mut self) -> NodeIx;
    fn add_edge(&mut self, tails: &[NodeIx], head: NodeIx) -> EdgeIx;

    // edge-aligned footprint must be storable without heap churn in hot paths
    fn set_footprint(&mut self, e: EdgeIx, fp: FootprintSpec);

    fn freeze_full(&self) -> Self::Frozen;
}

pub trait FrozenGraph {
    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;

    fn edge_tails(&self, e: EdgeIx) -> &[NodeIx];
    fn edge_head(&self, e: EdgeIx) -> NodeIx;

    // needed by Tier 3 push scheduling
    fn incident_edges(&self, node: NodeIx) -> &[EdgeIx];

    // optional (for pull); the proof case can remain push-only
    fn incoming_edges(&self, head: NodeIx) -> &[EdgeIx];

    fn footprint(&self, e: EdgeIx) -> FrozenFootprintRef<'_>;
}

pub enum FrozenFootprintRef<'a> {
    Influence { radius: f32, anchor: Option<NodeIx> },
    Simplex  { radius: f32, centroid: &'a Point3, arity: u8 },
}
```

**What this forces:**

* a **frozen representation** with slice-based access (CSR/SoA)
* an **incidence index** for push-mode scheduling (no scans)
* **footprint data** accessible edge-aligned, no per-edge heap pointers

### 2.2 Manifold state + view + deterministic update

```rust
pub trait Manifold {
    type Point: Copy;
    type Tangent: Copy;

    fn new(points: Vec<Self::Point>) -> Self;

    fn epoch(&self) -> u64;
    fn time(&self) -> f64;
    fn dt(&self) -> f32;
    fn set_dt(&mut self, dt: f32);

    fn points(&self) -> &[Self::Point];
    fn point(&self, n: NodeIx) -> Self::Point;

    // read-only geometry needed by gating and eval
    fn dist2(&self, a: Self::Point, b: Self::Point) -> f32;

    // apply-only mutation
    fn exp_map_in_place(&mut self, node: NodeIx, t: Self::Tangent);
    fn bump_epoch(&mut self);

    // deterministic integration hook (can be no-op if no velocities enabled)
    fn integrate(&mut self, dt: f32);
}
```

**What this forces:**

* epoch/time/dt are intrinsic manifold state (Goal 3)
* deterministic evolution under same dt sequence (Goal 3/4)

### 2.3 Gate cache + epoch invalidation + edge active set

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EpochKey {
    pub graph_epoch: u64,
    pub manifold_epoch: u64,
}

pub trait GateCache {
    fn new(edge_count: usize) -> Self;

    fn key(&self) -> EpochKey;
    fn is_valid_for(&self, k: EpochKey) -> bool;

    fn recompute<G: FrozenGraph, M: Manifold<Point=Point3>>(
        &mut self,
        g: &G,
        m: &M,
        graph_epoch: u64,
    );

    fn edge_active(&self, e: EdgeIx) -> bool;
    fn edge_weight(&self, e: EdgeIx) -> f32; // optional but useful
}
```

**What this forces:**

* explicit invalidation keyed by epochs (Goal 1)
* ability for geometry to gate topology (Goal 1)

### 2.4 Execution engine + trace + snapshot + replay

```rust
bitflags::bitflags! {
    pub struct TraceLevel: u32 {
        const OFF   = 0;
        const EDGES = 1<<0;
        const NODES = 1<<1;
        const GEO   = 1<<2;
        const EPOCH = 1<<3;
        const ALL   = Self::EDGES.bits() | Self::NODES.bits() | Self::GEO.bits() | Self::EPOCH.bits();
    }
}

pub trait Trace {
    fn fired_edges(&self) -> &[EdgeIx];
    fn moved_nodes(&self) -> &[NodeIx];
    fn geodesic_calls(&self) -> u64;
    fn epochs(&self) -> &[u64];
}

pub trait Engine {
    type Trace: Trace;
    type Snapshot: Clone;

    fn new_with_capacity(cap_events: usize) -> Self;

    fn enable_trace(&mut self, lvl: TraceLevel);
    fn take_trace(&mut self) -> Self::Trace;

    // deterministic scheduling
    fn schedule_push<G: FrozenGraph>(
        &mut self,
        g: &G,
        dirty_nodes: &[NodeIx],
        time: u64,
        priority: u32,
    );

    // compute+apply step (single writer apply)
    fn step<G, M, C>(
        &mut self,
        g: &G,
        gate: &mut C,
        manifold: &mut M,
        graph_epoch: u64,
        time: u64,
    )
    where
        G: FrozenGraph,
        M: Manifold<Point=Point3>,
        C: GateCache;

    // snapshot+replay
    fn snapshot<G, M, C>(
        &self,
        g: &G,
        gate: &C,
        manifold: &M,
        graph_epoch: u64,
    ) -> Self::Snapshot
    where
        G: FrozenGraph,
        M: Manifold<Point=Point3>,
        C: GateCache;

    fn replay(snapshot: Self::Snapshot) -> ReplayResult;
}

pub struct ReplayResult {
    pub manifold_hash: [u8; 32],
    pub final_epoch: u64,
}
```

**What this forces:**

* event-driven, deterministic ordering (Goal 4)
* trace is intrinsic and structured (Goal 4)
* snapshot is sufficient for bitwise replay (Goal 4)
* “compute then apply” semantics exist (Goal 3/4)
* clean hook points for performance guarantees (Goal 5)

---

## 3) The `GoalProofCase` runner (single test driver)

This is the singular runner that your “one test for all goals” uses.

```rust
pub fn run_goal_proof<G, M, C, E>(
    case: &GoalProofCase,
    mut graph_epoch: u64,
) -> GoalProofResult
where
    G: DynGraph,
    M: Manifold<Point=Point3>,
    C: GateCache,
    E: Engine,
{
    // 2D -> 3D embedding
    let points3: Vec<Point3> = case.points_xy.iter().copied().map(Into::into).collect();
    assert_xy_plane(&points3);

    // build dyn graph
    let mut hg = G::new();
    let mut nodes = Vec::with_capacity(points3.len());
    for _ in 0..points3.len() { nodes.push(hg.add_node()); }

    let mut edge_ixs = Vec::new();
    for (tails, head) in &case.edges {
        let e = hg.add_edge(tails, *head);
        edge_ixs.push(e);
    }
    for (e, fp) in edge_ixs.iter().copied().zip(case.footprints.iter().cloned()) {
        hg.set_footprint(e, fp);
    }

    let frozen = hg.freeze_full();

    // manifold + gate + engine
    let mut manifold = M::new(points3);
    manifold.set_dt(0.1);

    let mut gate = C::new(frozen.edge_count());
    gate.recompute(&frozen, &manifold, graph_epoch);

    let mut engine = E::new_with_capacity(1024);
    engine.enable_trace(TraceLevel::ALL);

    // STEP 0
    engine.schedule_push(&frozen, &case.step0_dirty, /*time*/0, /*prio*/0);
    engine.step(&frozen, &mut gate, &mut manifold, graph_epoch, 0);
    let t0 = engine.take_trace();

    // gate expectations step0 (after step0 recompute performed inside step or after)
    for (e, want) in &case.expect_edge_active_step0 {
        assert_eq!(gate.edge_active(*e), *want, "step0 gate active mismatch for edge {e}");
    }

    // STEP 1
    engine.schedule_push(&frozen, &case.step1_dirty, /*time*/1, /*prio*/0);
    engine.step(&frozen, &mut gate, &mut manifold, graph_epoch, 1);
    let t1 = engine.take_trace();

    for (e, want) in &case.expect_edge_active_step1 {
        assert_eq!(gate.edge_active(*e), *want, "step1 gate active mismatch for edge {e}");
    }

    // Validate fired/moved
    {
        let mut got = t0.fired_edges().to_vec();
        got.sort_unstable();
        let mut exp = case.expect_step0_fired.clone();
        exp.sort_unstable();
        assert_eq!(got, exp, "step0 fired mismatch");
    }
    {
        let mut got = t1.fired_edges().to_vec();
        got.sort_unstable();
        let mut exp = case.expect_step1_fired.clone();
        exp.sort_unstable();
        assert_eq!(got, exp, "step1 fired mismatch");
    }

    // XY-plane invariant remains true
    assert_xy_plane(manifold.points());

    // snapshot + replay
    let snap = engine.snapshot(&frozen, &gate, &manifold, graph_epoch);
    let snap_hash = hash_points(manifold.points());

    let replay = E::replay(snap.clone());
    let replay_hash = replay.manifold_hash;

    if case.expect_bitwise_replay {
        assert_eq!(snap_hash, replay_hash, "bitwise replay mismatch");
    }

    GoalProofResult {
        step0_fired: t0.fired_edges().to_vec(),
        step1_fired: t1.fired_edges().to_vec(),
        step0_moved: t0.moved_nodes().to_vec(),
        step1_moved: t1.moved_nodes().to_vec(),
        final_points: manifold.points().to_vec(),
        final_epoch: manifold.epoch(),
        snapshot_hash: snap_hash,
        replay_hash,
    }
}

// stable hash by raw bits (blake3 recommended, but any stable hash works)
fn hash_points(points: &[Point3]) -> [u8; 32] {
    use blake3::Hasher;
    let mut h = Hasher::new();
    for p in points {
        h.update(&p.x.to_le_bytes());
        h.update(&p.y.to_le_bytes());
        h.update(&p.z.to_le_bytes());
    }
    *h.finalize().as_bytes()
}
```

Your single “must pass all goals” test becomes:

```rust
#[test]
fn test_goal_proof_case_passes() {
    let case = make_default_goal_proof_case(); // your canonical scenario
    let _ = run_goal_proof::<HypergraphDynImpl, ManifoldStoreImpl, GateCacheImpl, EngineImpl>(&case, 0);
}
```

---

## 4) How this becomes a hardware conformance test later

You turn the exact same `GoalProofCase` into a **conformance vector** + **reference oracle**.

### 4.1 Freeze the contract: “Conformance Artifact”

Define a stable artifact format (binary or JSON + binary blobs):

* **Frozen graph blob** (CSR/SoA arrays)
* **Footprints blob** (edge-aligned)
* **Initial manifold state** (Point3 array + dt + epoch)
* **Initial gate state** (optional; or require recompute on device)
* **Event schedule** for step0 and step1 (explicit event keys)
* **Expected trace** (fired_edges list, moved_nodes list, counters)
* **Expected hashes** after each step (or after whole run)

This is your hardware “test ROM”.

### 4.2 Reference backend vs device backend

You will have:

* `cpu_ref` backend: canonical deterministic semantics
* `device` backend: GPU / SIMD / neuromorphic / FPGA

Both must implement the **same traits** (or a very similar “backend ABI”):

```rust
pub trait Backend {
    fn load_frozen(&mut self, frozen_blob: &[u8]);
    fn load_manifold(&mut self, points: &[Point3], dt: f32);
    fn load_footprints(&mut self, fp_blob: &[u8]);

    fn recompute_gate(&mut self, graph_epoch: u64, manifold_epoch: u64);

    fn run_step(&mut self, events: &[EventKey]) -> DeviceStepResult;

    fn read_points_hash(&self) -> [u8; 32];
    fn read_trace_digest(&self) -> [u8; 32]; // compact summary of fired edges, etc.
}
```

### 4.3 What “conformance” means in practice

You define tiers of conformance:

**Tier A: Semantic conformance (required)**

* fired edges match exactly
* gate active set matches exactly
* moved node set matches exactly
* epoch progression matches exactly

**Tier B: Bitwise state conformance (only if feasible)**

* point hash matches bit-for-bit

For GPU/accelerators, bitwise equality is often unrealistic unless you constrain arithmetic. In that case:

* keep Tier A strict
* allow Tier B under “strict deterministic math mode” only on supported devices

### 4.4 Deterministic replay becomes device validation

Your replay log is the portable “program”:

* same frozen graph + same event schedule + same dt sequence
* device must reproduce trace + hashes

This is exactly how ISAs get validated; you are validating a **geometric event semantics** instead of opcodes.

### 4.5 Why the exact same GoalProofCase works for hardware

Because it hits the critical properties hardware tends to break:

* ordering and reduction determinism
* cache invalidation / epoch coherence
* boundary between read-only compute and single-writer apply
* footprint prechecks vs “expensive” geometry
* trace capture without perturbing behavior

If a hardware backend passes this test artifact, it is extremely likely to be a correct implementation of your substrate contract for real workloads.

---

## 5) What you should lock in now

1. Implement the trait surfaces (or adapters) so the test compiles.
2. Implement `GoalProofCase` as a **single canonical conformance vector** in `mhg-testkit`.
3. Add a `--emit-conformance-artifact` CLI command later that writes the frozen blobs + expected hashes.
4. Treat that artifact as your future “device bring-up test”.

If you want, I can provide the concrete “default” `GoalProofCase` values (points, radius, expected step0/step1 fired sets) once you confirm your current gating predicate (centroid-distance? anchor-distance? both?) and whether `apply` updates only heads or can relax constraints affecting tails too.
