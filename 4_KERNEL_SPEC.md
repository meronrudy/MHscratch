# CausalManifold Kernel Spec v0.1 (Event–Geometry Substrate)

## 0. Scope

This document specifies the **minimal kernel** required to run causal–geometric computation where:

* computation is event-driven and distributed,
* state evolves continuously (integration),
* topology and geometry can co-evolve,
* topology mutation is deterministic and barriered,
* determinism and explainability follow from **trace + replay**.

This is a **kernel contract**, not an ISA. Higher-level "theories" compile into this contract.

---

## 1. Kernel Model

### 1.1 State Spaces

The kernel maintains two coupled state spaces:

**Geometry (Manifold)**

* `ManifoldStore` containing:

  * `epoch: u64` (monotone, increments on accepted mutation)
  * `t: u64` or `t_cont: f64` (time)
  * `dt: f32` (integration step)
  * `points: SoA(Point3)` (and optional velocities/momenta)

**Topology (Hypergraph)**

* Dynamic construction form: `HypergraphDyn`
* Frozen execution form: `FrozenHG` (CSR / SoA views)
* Hyperedges:

  * bounded arity (`k ≤ ARITY_MAX` or arity-classed)
  * `tails: [NodeIx; k]`, `head: NodeIx`
  * `signature: EdgeSignature` (semantic typing)
  * `footprint: Footprint` (geometric proxy used for gating)

### 1.2 The Kernel Execution Invariant

All execution is organized as:

1. **Event Evaluation Phase** (read-only w.r.t. topology and geometry)
2. **Apply Phase** (single-writer geometry updates)
3. **Structural Barrier Phase** (apply topology intents deterministically)

Inline topology mutation is forbidden.

---

## 2. Event Fabric

### 2.1 Event

An event is the primitive causal influence packet.

Required fields:

* `time: u64` (logical time)
* `class: EventClass` (typed)
* `src: LocusId` (sender)
* `dst: ScopeId` (receiver scope, unicast or multicast)
* `priority: u32` (deterministic precedence)
* `seq: u64` (monotone tie-breaker assigned at enqueue)

Total order:
`(time, priority, dst, src, class, seq)`
No other ordering is permitted.

### 2.2 Event Classes (v0.1 minimum)

* `GEOM` — geometric influence (deltas, relaxations)
* `FIELD` — scalar/vector field updates at loci (optional but reserved)
* `TOPO` — topology intents (never applied inline)

### 2.3 Delivery Contract (Resource Explicitness)

Kernel exposes explicit resource constraints:

* bounded queues (overflow observable)
* optional credit-based flow control per `dst`

Kernel must report:

* enqueue/pop counts
* overflow/drop counts
* per-class counts

---

## 3. Gating and Footprints

### 3.1 Gate Cache

Kernel may maintain a cache:

* `edge_active[e]: u8`
* `edge_weight[e]: f32`
* invalidated by `(manifold_epoch, graph_epoch)`

Hot paths must be able to traverse edges using only:

* frozen topology views
* gate cache arrays
* read-only manifold views

### 3.2 Footprints (v0.1)

Each hyperedge has an edge-aligned footprint enabling conservative rejection without full geometry ops.

Minimum supported footprint:

* `InfluenceRadius { radius: f32, anchor: Option<NodeIx> }`

Optional v0.1 footprint:

* `SimplexProxy { tails: [NodeIx; k], centroid_cached?: Point3, radius: f32 }`

---

## 4. Evaluation Semantics

### 4.1 Pure Evaluation

`eval_edge` must be:

* allocation-free
* pure (no mutation)
* deterministic given frozen + gate + manifold view + event payload

Outputs a list of **Deltas** and optionally **TopologyIntents**.

### 4.2 Delta Types (v0.1)

Minimum:

* `PointUpdate(node: NodeIx, tangent: Tangent3, weight: f32, source: EdgeIx|LocusId)`
* `Constraint(node: NodeIx, kind: ConstraintId, strength: f32, source: ...)` (optional v0.1)

Topology intents are emitted as events of class `TOPO` and only applied at barrier.

---

## 5. Apply Semantics (Single Writer, Deterministic Reduction)

### 5.1 Deterministic Merge

When multiple deltas target the same node in the same tick:

* canonicalize deltas by sorting:
  `(node, delta_kind, source_id, seq)`
* reduce in that fixed order

If bitwise determinism is required:

* use fixed summation order and stable math mode
* prohibit non-deterministic parallel reduction

### 5.2 Geometry Update

Apply uses manifold operators, e.g.:

* `x' = exp_map(x, tangent)` or projection variant

Epoch rule:

* `manifold_epoch += 1` iff at least one accepted mutation occurs
* epoch transitions are logged

---

## 6. Structural Barrier (Topology Mutation)

### 6.1 TOPO Intents

All topology changes occur only via TOPO intents at barrier:

Minimum supported intents:

* `AddEdge`
* `RemoveEdge`
* `UpdateFootprint`

### 6.2 Canonical Conflict Resolution

Barrier must be deterministic:

* collect intents for the tick
* canonicalize (sort by `(time, src, seq, intent_kind, target_id)`)
* resolve conflicts by a fixed policy (documented)
* apply resulting changes to dynamic graph, producing new `graph_epoch`

---

## 7. Stability Contract (Kernel Boundary)

All loci update rules exposed at kernel boundary must be **bounded / dissipative**.

v0.1 enforcement:

* provide a "bounded update" trait boundary
* optionally include runtime debug assertions
* record stability-relevant signals in trace (optional)

---

## 8. Trace, Replay, and Hashing (Conformance)

### 8.1 Required Trace Items (v0.1)

Kernel must be able to emit a replay capsule consisting of:

* canonical instance hash (`compile_hash`) (recommended)
* ordered event log (after ordering)
* applied deltas (after canonical reduction)
* topology intents applied (after barrier)
* epoch transitions
* periodic state digests (`checkpoint_hash`)

### 8.2 Hashes

Minimum:

* `checkpoint_hash(tick)` over manifold points + epochs (+ topology digest)

Optional:

* `trace_hash` over normalized log
* `compile_hash` over canonical instance encoding

---

## 9. Implementation Requirements

* Frozen traversal is allocation-free
* Apply is the only writer during compute ticks
* Topology mutation only at barrier
* Resource limits are explicit and observable
* Determinism level is declared

---

## 10. Conformance Levels (v0.1)

* **Level 0:** final state + basic counters
* **Level 1:** event log + checkpoint digests
* **Level 2:** full normalized trace hash + replay capsule

---

# Compiling HyperFlow and MHG Against the Kernel

This section shows the compilation target: **both theories become event generators + edge evaluators + intent emitters**.

---

## A) HyperFlow → Kernel

HyperFlow (as "dynamical field theory") compiles into:

### A.1 Kernel objects

* **Field loci**: per-node state variables (potentials, flows)
* **Flow edges**: hyperedges encoding local coupling rules
* **GEOM/FIELD events**: impulses and boundary conditions
* **TOPO intents**: optional, for flow-driven topology surgery

### A.2 Compilation mapping

| HyperFlow concept          | Kernel representation                                |
| -------------------------- | ---------------------------------------------------- |
| Field variable at locus    | `FIELD` state in `ManifoldStore` or per-locus store  |
| Local flow rule            | `eval_edge` producing `PointUpdate` or `FIELD` delta |
| Metric gating of influence | `GateCache` weight + `Footprint` precheck            |
| Curvature/metric evolution | `GEOM` events that update metric parameters in Apply |
| Topology surgery           | `TOPO` intents at barrier                            |

### A.3 Minimal compiled artifact

A HyperFlow "program" is:

1. A set of edge templates (arity/signature + footprints)
2. Locus parameters (decay, coupling constants)
3. An initial event seed (boundary conditions)
4. Optional schedule policy (how events are generated)

Execution is just running the kernel.

---

## B) MHG → Kernel

MHG (as "manifold-hypergraph hybrid reasoning/learning") compiles into:

### B.1 Kernel objects

* Hyperedges represent constraints / relations (logic-like)
* Footprints represent geometric semantics of those relations
* Events represent evidence arrival, hypothesis propagation, or relaxation triggers
* Deltas represent geometric movement and/or constraint activation

### B.2 Compilation mapping

| MHG concept                  | Kernel representation                     |
| ---------------------------- | ----------------------------------------- |
| Relation (k-ary)             | hyperedge with `tails/head`               |
| Constraint surface / simplex | `Footprint` + gate function               |
| Reasoning step               | `GEOM` event + edge firing                |
| Evidence update              | `FIELD` or `GEOM` event at specific nodes |
| Structure learning           | `TOPO` intents at barrier                 |

MHG "reasoning" is not a special mode: it is a pattern of edge firings and geometric relaxations.

---

# Minimal Verifier Derived From the ADRs

The minimal verifier checks **kernel conformance** without knowing the theory.

## 1. Inputs

* `instance` (ToyInstance or canonical instance format)
* `solver_output` (RunSummary-like output)
* `conformance_level` claimed by solver

## 2. Required checks by level

### Level 0 (minimum)

* Structural validity: counts match instance (nodes/edges)
* Declared determinism level present
* Basic invariant checks:

  * no inline topology changes (must be absent from compute phase logs if provided)
  * manifold epoch monotone

### Level 1

* Event log exists and is in total order
* Checkpoint digests exist at required ticks
* Replay (partial) consistency:

  * at minimum verify digests are deterministic across two runs with same input (CI mode)

### Level 2

* Canonical `compile_hash` matches instance encoding
* `trace_hash` matches normalized trace computation
* Full replay capsule exists:

  * replayer re-executes from snapshot + event log + intents
  * validates checkpoint hashes at each tick
  * validates final state hash bit-for-bit (strict mode)

## 3. Outputs

A single machine-readable record:

```json
{
  "instance_id": "...",
  "pass": true,
  "level": 2,
  "booleans": {
    "ordering": true,
    "barrier_semantics": true,
    "replay": true,
    "hashes": true
  },
  "counts": { "ticks": 10, "events": 41, "edges_fired": 12 },
  "hashes": { "compile": "...", "trace": "...", "final_state": "..." }
}
```

## 4. Minimal verifier implementation plan

* `parse_instance()`
* `parse_solver_output()`
* `check_total_order(event_log)`
* `check_barrier_semantics(intent_log)` (if present)
* `compute_checkpoint_hashes(replay_state)`
* `compare_hashes()`
* emit verdict JSON

That is sufficient to make:

* CGRB-style benchmarking feasible,
* hardware conformance possible (same interface),
* and solver interchangeability real.