# Architectural Decision Exploration Checklist

**Using Toy Backends as Probes for MHG Engine Design**


> **Purpose**
> Use small, intentionally constrained toy backends to explore the design space and make irreversible architectural commitments with evidence.

Toys are **instruments**, not candidates.

---

## How to Read This Checklist

For each **architectural decision under consideration**, you:

1. Run one or more toy backends that **exercise that decision**.
2. Answer the checklist questions using **observations**, not opinions.
3. Decide whether the decision becomes:

   * **MANDATORY** (architectural invariant)
   * **OPTIONAL** (configurable / backend-specific)
   * **REJECTED** (disallowed by design)

---

## Decision 1: What is the primitive unit of computation?

**Candidates**

* Instruction (ISA-style)
* Event (causal packet)
* Continuous flow (ODE step)

### Observations from toys

* ☑ Event ordering must be explicit to guarantee replay - observed in GESC Fabric backend where timestamped events enable deterministic scheduling
* ☑ Geometry updates naturally express flow, not steps - Toy Hardware backend shows continuous geometric evolution rather than discrete instructions
* ☑ Structural change cannot be modeled as an instruction without barriers - GESC Barrier backend demonstrates that topology mutations require explicit synchronization

### Decision outcome

* ☑ **MANDATORY**: Event as primitive
* ☐ OPTIONAL
* ☐ REJECTED

**Rationale (from toys):**
Event-driven computation enables causal traceability, deterministic replay, and proper handling of both discrete events (topology changes) and continuous evolution (geometric updates). Instructions impose sequential fiction that breaks causality in adaptive systems.

---

## Decision 2: How is time represented?

**Candidates**

* Global loop index
* Logical time in event packets
* Continuous time embedded in state

### Observations from toys

* ☑ Loop index breaks causal replay - Toy Hardware's simple tick-based execution cannot reproduce event ordering across different scheduling decisions
* ☑ Event time enables deterministic scheduling - GESC Fabric's timestamped events (u16) provide total ordering for replay and verification
* ☑ Continuous integration requires explicit `dt` - All backends require explicit time step control for geometric evolution predictability

### Decision outcome

* ☑ **MANDATORY**: Intrinsic time (event + integration)
* ☐ OPTIONAL
* ☐ REJECTED

**Rationale (from toys):**
Dual time representation (logical event time + continuous integration time) enables both causal ordering and physical simulation. Pure loop indices cannot capture the temporal relationships needed for deterministic replay and convergence analysis.

---

## Decision 3: Can topology mutate inline?

**Candidates**

* Inline mutation during execution
* Deferred mutation via barrier
* Immutable topology

### Observations from toys

* ☑ Inline mutation breaks determinism - Software Ideal backend shows how immediate topology changes prevent reproducible execution traces
* ☑ Barriered TOPO resolves conflicts cleanly - GESC Barrier backend demonstrates deterministic intent resolution through explicit barriers
* ☑ Structural churn must be visible and countable - All backends track epoch changes to maintain consistency across topology mutations

### Decision outcome

* ☑ **MANDATORY**: Barriered structural updates
* ☐ OPTIONAL
* ☐ REJECTED

**Rationale (from toys):**
Inline topology mutation introduces non-determinism and race conditions, especially under parallel execution. Barriered updates provide clean conflict resolution and maintain auditability of structural evolution.

---

## Decision 4: How are higher-order relations represented?

**Candidates**

* Pairwise edges only
* Hyperedges without geometry
* Hyperedges with geometric footprint

### Observations from toys

* ☑ Pairwise reduction loses interaction semantics - Toy Hardware demonstrates that multi-tail constraints (centroid-based gating) cannot be expressed as edge pairs
* ☑ Footprints allow conservative gating - All backends use geometric footprints to avoid expensive geodesic computations when geometrically impossible
* ☑ k-ary relations must be first-class - GESC backends show that higher-order constraints (multiple nodes influencing single outcomes) are fundamental to the domain

### Decision outcome

* ☑ **MANDATORY**: Hyperedges with footprints
* ☐ OPTIONAL
* ☐ REJECTED

**Rationale (from toys):**
Higher-order relations (hyperedges) are irreducible in geometric constraint systems. Footprints enable efficient gating by providing geometric bounds for expensive computations, making the system tractable for real-time execution.

---

## Decision 5: What enforces stability?

**Candidates**

* Program logic
* Runtime checks
* Architectural constraints (dissipative loci)

### Observations from toys

* ☑ Runtime checks are late and fragile - Software Ideal shows how stability violations manifest as runtime failures rather than being prevented
* ☑ GLI-style loci guarantee BIBO stability - GESA Loci backend demonstrates how dissipative update contracts prevent runaway dynamics
* ☑ Stability must be local and enforced - All backends show that stability cannot be added as an afterthought but must be architecturally guaranteed

### Decision outcome

* ☑ **MANDATORY**: Dissipative update contracts
* ☐ OPTIONAL
* ☐ REJECTED

**Rationale (from toys):**
Runtime stability checks are reactive and can fail catastrophically. Architectural dissipative contracts (like GLI loci) prevent instability by construction, enabling reliable long-running adaptive systems.

---

## Decision 6: What must be deterministic?

**Candidates**

* Outputs only
* Event order
* Full trace (hashable)

### Observations from toys

* ☑ Output-only determinism hides bugs - Toy Hardware shows how different execution paths can produce same final geometry while having different intermediate behaviors
* ☑ Trace determinism enables replay and audit - All backends demonstrate that complete execution traces are necessary for forensic analysis and verification
* ☑ Hashable traces enable conformance testing - GESC backends show how trace hashing enables automated verification of backend correctness

### Decision outcome

* ☑ **MANDATORY**: Trace-level determinism
* ☐ OPTIONAL
* ☐ REJECTED

**Rationale (from toys):**
Output-only determinism is insufficient for debugging, verification, and audit. Full trace determinism enables replay, forensic analysis, and automated conformance testing across different implementations.

---

## Decision 7: How visible must resources be?

**Candidates**

* Abstract (framework hides details)
* Semi-visible (counters)
* Explicit (memory, queues, credits)

### Observations from toys

* ☑ Hidden queues hide failure modes - Multi-Lane backend reveals how abstracted parallelism can conceal memory conflicts and load imbalances
* ☑ Explicit credit pressure reveals bottlenecks - GESC Fabric's visible credit system exposes flow control points that would be hidden in abstracted models
* ☑ Hardware mapping requires explicit resource shape - All backends demonstrate that predictable hardware implementation requires explicit knowledge of memory layout, queue depths, and communication patterns

### Decision outcome

* ☑ **MANDATORY**: Explicit resource model
* ☐ OPTIONAL
* ☐ REJECTED

**Rationale (from toys):**
Abstracted resources hide performance characteristics and failure modes. Explicit resource models enable hardware mapping, performance optimization, and predictable scaling behavior.

---

## Decision 8: Where does geometry live?

**Candidates**

* Metadata on nodes
* External embedding
* First-class evolving manifold

### Observations from toys

* ☑ Metadata geometry cannot gate causality - Toy Hardware shows that geometry must be queryable during execution, not just stored as node attributes
* ☑ External embeddings break replay - GESC backends demonstrate that geometry must be part of the execution state to maintain deterministic evolution
* ☑ Manifold state must co-evolve - All backends show that geometric and topological evolution are interdependent and must advance together

### Decision outcome

* ☑ **MANDATORY**: First-class manifold state
* ☐ OPTIONAL
* ☐ REJECTED

**Rationale (from toys):**
Geometry cannot be treated as secondary metadata. It must be first-class state that co-evolves with topology, enabling bidirectional causality between space and structure.

---

## Decision 9: What is fixed vs extensible?

**Candidates**

* Everything fixed
* Everything dynamic
* Minimal fixed core + extensible theories

### Observations from toys

* ☑ Fixed semantics ossify design - Monolithic backends show how rigid semantics prevent adaptation to new domains
* ☑ Unbounded extensibility breaks guarantees - GESA Loci demonstrates how different theories (field, topology, constraint) can coexist while maintaining system guarantees
* ☑ Narrow waist enables coexistence - The common event/geometry/barrier substrate allows multiple theoretical frameworks to interoperate

### Decision outcome

* ☑ **MANDATORY**: Fixed substrate + extensible theories
* ☐ OPTIONAL
* ☐ REJECTED

**Rationale (from toys):**
Purely fixed systems cannot adapt to new domains. Purely dynamic systems cannot provide guarantees. A fixed substrate with extensible theories provides both reliability and adaptability.

---

# How Toys Fit (explicitly)

Toys are used to answer **specific decision questions**:

| Toy Backend | Decisions Probed | Key Insights |
|-------------|------------------|--------------|
| **Software Ideal** | Semantic ground truth, stability requirements | Reference behavior, dissipative contracts needed |
| **Toy Hardware** | Resource visibility, time representation, geometry placement | Fixed-point precision, explicit memory, first-class geometry |
| **GESC Fabric** | Event primitives, determinism level, resource visibility | Timestamped events, trace determinism, credit flow control |
| **GESC Barrier** | Structural mutation, time coupling | Barriered topology, dual time representation |
| **GESA Loci** | Stability enforcement, extensibility model | Dissipative loci, theory coexistence |
| **Multi-Lane** | Parallel execution, resource conflicts | Explicit conflicts, load balancing requirements |

A toy is **successful** if it clarifies a decision—even if it is later discarded.

---

# Final Outcome of This Process

What you are producing is:

* **A set of architectural invariants** - 9 mandatory decisions justified by toy observations
* **Each justified by concrete observations** - Not opinions, but measurable toy behaviors
* **Each independent of any single implementation** - Applicable across software/hardware boundaries

This is exactly how:

* instruction sets (x86, ARM),
* memory models (SC-DRF),
* and hardware fabrics (AXI, PCIe)

are responsibly designed.

---

# Architecture Decision Records (ADR)

## ADR-001: Computational Primitive Is Event, Not Instruction

**Status:** Accepted  
**Date:** 2025-12-30  
**Decision Maker:** Architecture Exploration via Toy Backends  

### Context

The system targets computation that is causal, distributed, continuous, and structurally adaptive. Traditional instruction-based models impose a sequential fiction and obscure causality, making deterministic replay and structural adaptation difficult or impossible.

### Decision

The primitive unit of computation is a **causal event**, not an instruction.

An event represents a bounded causal influence that may:

* trigger geometric evolution,
* gate or weight relations,
* schedule further events,
* or propose structural change.

### Consequences

* Execution is defined by event delivery and resolution, not program counters.
* Deterministic ordering must be explicit.
* Instruction-level abstractions are disallowed at the substrate level.

### GESC Mapping

* **GESC Core:** Event packet as the fundamental unit
* **GESC Layer 1:** Causal delivery semantics
* **GESC Layer 2:** Explicit event classes (GEOM, FIELD, TOPO)

---

## ADR-002: Time Is Intrinsic and Dual (Logical + Continuous)

**Status:** Accepted  
**Date:** 2025-12-30  

### Context

The system models dynamics where correctness depends on trajectories and convergence, not discrete steps.

### Decision

Time is intrinsic and represented in two coupled forms:

* **Logical time**: carried by events for ordering and replay.
* **Continuous time**: embedded in manifold state and integration.

External loop indices are forbidden as semantic time.

### Consequences

* Every execution has a reproducible temporal interpretation.
* Integration steps (`dt`) are explicit and traceable.
* Replay reproduces temporal evolution exactly under declared determinism level.

### GESC Mapping

* **GESC Layer 1:** Timestamped events
* **GESC Layer 3:** Continuous state evolution between events

---

## ADR-003: Structural Change Is Barriered

**Status:** Accepted  
**Date:** 2025-12-30  

### Context

Inline mutation of topology during execution introduces nondeterminism and race conditions, especially under parallel evaluation.

### Decision

All topology mutations are **proposed as intents** and applied only at **explicit structural barriers**.

### Consequences

* Structural changes are deterministic and auditable.
* Conflicts are resolved canonically.
* Topology becomes part of the causal trace.

### GESC Mapping

* **GESC Layer 4:** Structural intent + barrier semantics
* **GESC Contract:** Compute → Barrier → Apply phases

---

## ADR-004: Higher-Order Relations Are First-Class Geometric Objects

**Status:** Accepted  
**Date:** 2025-12-30  

### Context

Pairwise graphs cannot represent irreducible interactions or joint causation.

### Decision

Relations are represented as **hyperedges with geometric footprints**, not as sets of nodes or adjacency lists.

A hyperedge may correspond to:

* a simplex,
* a constraint surface,
* or a region of influence on the manifold.

### Consequences

* Gating and weighting are geometric operations.
* Higher-order semantics are preserved.
* Reduction to pairwise interactions is disallowed.

### GESC Mapping

* **GESC Semantics:** Events may target scopes, not just nodes
* **GESC Geometry Coupling:** Geometry gates causality

---

## ADR-005: Stability Is an Architectural Property

**Status:** Accepted  
**Date:** 2025-12-30  

### Context

Unstable dynamics invalidate long-running, adaptive systems and cannot be reliably controlled by user code alone.

### Decision

All update rules at the substrate boundary must satisfy **dissipative or bounded contracts**.

Stability is enforced by construction, not by convention.

### Consequences

* BIBO stability is guaranteed locally.
* Learning and adaptation do not cause runaway behavior.
* Stability violations are impossible, not just detected.

### GESC Mapping

* **GESC/GESA:** Dissipative endpoint requirement
* **GLI Concept:** Guaranteed local invariants

---

## ADR-006: Determinism Is Defined at the Trace Level

**Status:** Accepted  
**Date:** 2025-12-30  

### Context

Output-only determinism is insufficient for auditability, replay, and verification.

### Decision

Determinism is defined as **trace-level equivalence** under a declared determinism mode.

At minimum:

* event ordering,
* applied deltas,
* epoch transitions

must be reproducible.

### Consequences

* Replay is a first-class capability.
* Hashes can certify execution.
* Conformance testing is possible.

### GESC Mapping

* **GESC Contract:** Deterministic delivery rules
* **Verifier Model:** Trace hashing and replay capsules

---

## ADR-007: Resource Shape Must Be Explicit

**Status:** Accepted  
**Date:** 2025-12-30  

### Context

Hidden queues, memory hierarchies, and contention obscure failure modes and prevent hardware mapping.

### Decision

Memory, queues, credits, and operators must be explicit, countable, and observable.

### Consequences

* Bottlenecks are visible.
* Hardware feasibility can be assessed.
* Performance follows structure, not heuristics.

### GESC Mapping

* **GESC Fabric:** Credit-based flow control
* **Endpoint Contracts:** Explicit buffering and capacity

---

## ADR-008: Geometry Is First-Class State

**Status:** Accepted  
**Date:** 2025-12-30  

### Context

Treating geometry as metadata or an external embedding prevents bidirectional causality.

### Decision

Geometry is a **first-class, evolving manifold**, co-equal with topology.

### Consequences

* Space reshapes structure.
* Structure reshapes space.
* Computation is geometric evolution.

### GESC Mapping

* **GESC Geometry Semantics:** Geometry gates and weights events
* **GESC Layer 3:** Geometry, Events, Stability unified

---

## ADR-009: Fixed Substrate, Extensible Theories

**Status:** Accepted  
**Date:** 2025-12-30  

### Context

Monolithic semantics ossify systems; unconstrained extensibility breaks guarantees.

### Decision

The substrate defines a **minimal, fixed kernel**, while theories (flows, constraints, learning rules) are pluggable and coexist.

### Consequences

* Multiple paradigms run simultaneously.
* Guarantees are preserved.
* Long-term extensibility without architectural debt.

### GESC Mapping

* **GESC Narrow Waist:** Stable contract, extensible semantics
* **Theory Layer:** Compiled to event–geometry interactions

---

# Derived Minimal Kernel Specification

From the accepted ADRs, the **minimal kernel** must provide:

---

## 1. Event Fabric

* Timestamped causal events with total deterministic ordering
* Typed event classes: GEOM (geometric evolution), FIELD (scalar dynamics), TOPO (structural change)
* Credit-based delivery with explicit flow control
* Event queue with bounded depth and overflow protection

---

## 2. Execution Semantics

* **Compute Phase**: Event evaluation against frozen topology and current geometry (read-only)
* **Structural Barrier**: Deterministic intent resolution with canonical conflict ordering
* **Apply Phase**: Atomic state updates with epoch advancement
* Clear phase separation preventing inline topology mutation

---

## 3. Geometry Core

* First-class manifold state with explicit time tracking (t, dt, epoch)
* Deterministic integration hooks for continuous evolution
* Geometric predicates for gating (distance, containment, intersection)
* Projection and constraint operations on the manifold

---

## 4. Topology Core

* Hyperedges with explicit arity and geometric footprints
* Immutable frozen views for execution phase
* Intent-based mutation system (propose → barrier → apply)
* Edge-aligned storage for efficient access patterns

---

## 5. Stability Contracts

* Dissipative update requirements at all kernel boundaries
* Bounded amplification factors in update rules
* Local stability guarantees (BIBO - Bounded Input, Bounded Output)
* Explicit invariant enforcement mechanisms

---

## 6. Trace & Replay

* Complete event log with ordering and timestamps
* Applied delta log for all state changes
* Epoch transition records for consistency verification
* Hashable trace representation for integrity checking

---

## 7. Resource Model

* Explicit NodeMem: SoA layout (x[], y[], z[]) with counters
* Explicit EdgeMem: packed tails with footprint storage
* Explicit queues: bounded event queues with overflow tracking
* Observable counters: reads, writes, allocations, conflicts

---

## Implementation Requirements

### Memory Layout
```
NodeMem {
    x: [Q16_16; N_NODES_MAX],  // SoA for SIMD
    y: [Q16_16; N_NODES_MAX],
    z: [Q16_16; N_NODES_MAX],
    count: usize,
    reads: u64,   // Observable counters
    writes: u64,
}
```

### Event Structure
```
Gescevent {
    raw: u64,  // Packed 64-bit format
    // Fields: src_id(18), dst_scope(18), class(4), tstamp(16), qos(4), payload_id(4)
}
```

### Execution Phases
```
fn execute_barrier_cycle(&mut self) {
    // Phase 1: Compute (events → geometry updates)
    self.compute_phase();

    // Phase 2: Barrier (resolve topology intents)
    self.apply_structural_barrier();
}
```

---

## Validation Against Toy Backends

Each kernel requirement is validated by at least one toy backend:

| Requirement | Validated By | Evidence |
|-------------|--------------|----------|
| Event Fabric | GESC Fabric | Credit flow control, deterministic ordering |
| Execution Semantics | GESC Barrier | Phase separation, intent resolution |
| Geometry Core | Toy Hardware | First-class manifold, integration hooks |
| Topology Core | All backends | Hyperedges with footprints |
| Stability Contracts | GESA Loci | GLI dissipative updates |
| Trace & Replay | All backends | Reproducible failures ≤5 nodes |
| Resource Model | Toy Hardware | Explicit memory layout, counters |

---

## What This Gives You

* A **kernel specification**, not an implementation
* A **virtual causal-geometric machine** that can target multiple substrates
* A substrate that can power:

  * **Software**: High-level language implementations
  * **Hardware**: ASIC/FPGA accelerators
  * **Neuromorphic**: Event-driven cognitive architectures
  * **Hybrid**: CPU+accelerator systems

Most importantly:
every design choice is now **justified by evidence from toy backends**, not speculation.

This specification enables responsible architectural evolution while maintaining the mathematical guarantees that make the system unique.