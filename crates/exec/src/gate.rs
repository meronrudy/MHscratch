use crate::cache::GateCache;
use core::ids::EdgeIx;
use hypergraph::frozen::HypergraphFrozen;
use manifold::store::ManifoldStore;

#[derive(Clone, Default)]
pub struct DistanceGate {
    pub eps: f32,
    cache: GateCache,
}

impl DistanceGate {
    pub fn allow(&mut self, m: &ManifoldStore, g: &HypergraphFrozen, e: EdgeIx) -> bool {
        if !self.cache.is_valid(m.epoch, g.epoch) {
            let mut edge_active = Vec::new();
            let mut edge_weight = Vec::new();
            for edge in g.edge_indices() {
                let tails = g.edge_tails(edge);
                debug_assert_eq!(tails.len(), 2, "demo expects arity-2 edges");
                let dist = m.dist_nodes(tails[0], tails[1]);
                edge_active.push((dist < self.eps) as u8);
                edge_weight.push(dist);
            }
            self.cache.update(edge_active, edge_weight, m.epoch, g.epoch);
        }
        self.cache.edge_active[e as usize] != 0
    }
}
