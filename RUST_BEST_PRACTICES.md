# Rust-Specific Best Practices for a High-Performance Geometric Engine

These are not generic “Rust tips”; they are the things that matter when you build dynamic graph + geometry + execution engines from scratch.

## 1. Memory Layout Pitfalls

-   **Prefer edge-centric SoA over node-centric AoS**: `Vec<Edge>` with `Vec<Vec<Node>>` tails is slow. The chosen packed-tail CSR-style layout is correct.
-   **`Vec::with_capacity` is not initialization**: Do not assume `len == capacity`. Only write via `push` or explicit indexing after `set_len` (unsafe).
-   **Avoid `Box<[T]>` until frozen**: `Vec<T>` is faster for construction. Convert to `Box<[T]>` only for immutability guarantees.

## 2. Borrow Checker Traps

-   **Design around the borrow checker**: Separate read and write phases structurally.

    ```rust
    // Bad pattern
    for e in edges {
        let p = manifold.point_mut(node); // mutable borrow
        let w = gate.weight(&manifold, e); // immutable borrow
    }

    // Good pattern
    // Phase 1: read-only gather
    let deltas = eval_all(&frozen, &manifold);
    // Phase 2: apply mutations
    apply_all(&mut manifold, deltas);
    ```

-   **Interior mutability is a trap**: `RefCell`, `Mutex`, `RwLock` will kill performance. Use them only at system boundaries, never in hot loops.

## 3. Floating-Point Determinism

-   **Parallelism breaks determinism**: Parallel reductions reorder non-associative floating-point sums.
-   **Mitigations**: Use deterministic chunking, fixed traversal order, or accept epsilon-level drift.
-   **Do not rely on `==` for floats**: Use exact bit checks only at defined sync points, otherwise use tolerance-based validation.

## 4. Trait Design Mistakes

-   **Avoid overly generic traits early**: Specialize first. Generalize only when duplication hurts. Monomorphization cost is real.
-   **Beware `dyn Trait` in hot paths**: Trait objects are vtable lookups. Geometry kernels should be monomorphized and inlined.

## 5. `unsafe` is Not Evil — But Quarantine It

-   **Justification**: CSR construction, SIMD kernels, pointer-based iteration.
-   **Rules**: One `unsafe` block per invariant, comment the invariant, and wrap in a safe function.

## 6. Epochs Are Your Best Friend

-   **Always version mutable state**: Use epochs for manifold state, graph topology, gate caches, and event queues.
-   **Why**: Cheap invalidation and replay validation without deep equality checks.

## 7. Logging and Tracing

-   **Never log inside hot loops**: Collect lightweight trace IDs (`EdgeIx`, `NodeIx`) and expand to rich logs after execution if needed.

## 8. A Testing Strategy That Works

-   **Start with golden-file and replay tests**, not property tests. You need determinism first.

## 9. Compilation Time and Ergonomics

-   Keep crate boundaries clean.
-   Avoid giant generic types crossing crate boundaries.
-   Use feature flags (`perf`, `trace`, `units`).

## 10. The Conceptual Trap to Avoid

-   **Do NOT let tensors creep in early**. Keep the core substrate focused on graph + geometry + events. Tensors belong at the analysis or optional learning layer.

## Guiding Rule

**If a design choice makes memory layout, execution order, or invariants less obvious in code, it is probably the wrong choice.**
