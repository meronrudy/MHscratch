//! Freeze Tests: Structural equivalence and determinism.
//!
//! Ensures freezing is deterministic,
//! incremental and full paths are equivalent.

use hypergraph::dynamic::HypergraphDyn;
use hypergraph::freeze::freeze_checked;
use manifold::store::{ManifoldStore, ExplicitEuler};
use manifold::footprint::EdgeFootprint;

#[test]
fn full_freeze_determinism() {
    // Build graph, freeze twice, assert identical
    let mut g = HypergraphDyn::new(0u64);
    g.add_edge(&[0], 1, EdgeFootprint::V1(Default::default()));

    let mut g2 = g.clone();
    let (frozen1, _) = freeze_checked(g, Default::default());
    let (frozen2, _) = freeze_checked(g2, Default::default());

    // TODO: Implement bit-for-bit comparison or stable hashing
    // assert_eq!(frozen1.as_bytes(), frozen2.as_bytes());
}

#[test]
fn incremental_vs_full_freeze_equivalence() {
    // Build base, apply delta, compare incremental vs full freeze
    // TODO: Implement with delta API
}