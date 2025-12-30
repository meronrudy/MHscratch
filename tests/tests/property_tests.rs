//! Property Tests: Structural sanity only.
//!
//! Validates CSR validity, bounds, no panics on valid input.
//! Does NOT test geometry semantics or causality.

#[cfg(test)]
mod tests {
    use hypergraph::dynamic::HypergraphDyn;
    use hypergraph::freeze::freeze_checked;
    use manifold::store::{ManifoldStore, ExplicitEuler};

    #[test]
    fn csr_integrity() {
        // Build graph, freeze, assert offsets monotone, indices in bounds
        let mut g = HypergraphDyn::new(0u64);
        g.add_edge(&[0], 1, manifold::footprint::EdgeFootprint::V1(Default::default()));
        g.add_edge(&[1], 2, manifold::footprint::EdgeFootprint::V1(Default::default()));

        let (frozen, _) = freeze_checked(g, Default::default());

        // Check offsets are monotone
        for w in frozen.out_off.windows(2) {
            assert!(w[0] <= w[1]);
        }
        // Check indices < n_nodes
        for &idx in &frozen.out_edges {
            assert!(idx < frozen.n_nodes);
        }
    }

    #[test]
    fn compaction_invariance() {
        // TODO: Test compaction doesn't change visible adjacency
    }
}