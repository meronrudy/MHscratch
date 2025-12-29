pub mod gate;

use core::ids::Epoch;

#[derive(Clone, Debug, Default)]
pub struct GateCache {
    pub epoch_graph: Epoch,
    pub epoch_manifold: Epoch,
    pub edge_active: Vec<u8>,
    pub edge_weight: Vec<f32>,
}

impl GateCache {
    pub fn is_valid(&self, graph_epoch: Epoch, manifold_epoch: Epoch) -> bool {
        self.epoch_graph == graph_epoch && self.epoch_manifold == manifold_epoch
    }
}
