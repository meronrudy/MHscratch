use core::ids::Epoch;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct Invalidator {
    manifold_epoch: Epoch,
    graph_epoch: Epoch,
}

#[derive(Clone, Debug, Default)]
pub struct GateCache {
    pub edge_active: Vec<u8>,
    pub edge_weight: Vec<f32>,
    pub invalidator: Option<Invalidator>,
}

impl GateCache {
    pub fn is_valid(&self, manifold_epoch: Epoch, graph_epoch: Epoch) -> bool {
        if let Some(invalidator) = self.invalidator {
            invalidator.manifold_epoch == manifold_epoch && invalidator.graph_epoch == graph_epoch
        } else {
            false
        }
    }

    pub fn update(&mut self, edge_active: Vec<u8>, edge_weight: Vec<f32>, manifold_epoch: Epoch, graph_epoch: Epoch) {
        self.edge_active = edge_active;
        self.edge_weight = edge_weight;
        self.invalidator = Some(Invalidator {
            manifold_epoch,
            graph_epoch,
        });
    }
}
