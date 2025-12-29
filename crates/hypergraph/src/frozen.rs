use crate::arity::ArityGroup;
use crate::signature::EdgeSignature;
use core::ids::{EdgeIx, NodeIx, Epoch};
use traits::footprint::Footprint;

#[derive(Clone, Debug)]
pub struct HypergraphFrozen {
    pub epoch: Epoch,
    pub n_nodes: u32,
    pub n_edges: u32,

    // Edge payload (common to all arities)
    pub edge_head: Vec<NodeIx>,
    pub edge_kind: Vec<u16>,
    pub edge_info: Vec<(ArityGroup, u32)>,
    pub signatures: Vec<EdgeSignature>,
    pub footprints: Vec<Footprint>,

    // Arity-specific storage
    pub tails_k2: Vec<NodeIx>,
    pub edges_k2: Vec<EdgeIx>,

    pub tails_k3: Vec<NodeIx>,
    pub edges_k3: Vec<EdgeIx>,

    // Variable-arity edges (packed CSR)
    pub tail_off_var: Vec<u32>,
    pub tails_var: Vec<NodeIx>,
    pub edges_var: Vec<EdgeIx>,

    // Node adjacency CSR (for all edges)
    pub out_off: Vec<u32>,
    pub out_edges: Vec<EdgeIx>,
    pub in_off: Vec<u32>,
    pub in_edges: Vec<EdgeIx>,
}

impl HypergraphFrozen {
    pub fn edge_tails(&self, e: EdgeIx) -> &[NodeIx] {
        let (group, group_index) = &self.edge_info[e as usize];
        match group {
            ArityGroup::K2 => {
                let start = *group_index as usize * 2;
                &self.tails_k2[start..start + 2]
            }
            ArityGroup::K3 => {
                let start = *group_index as usize * 3;
                &self.tails_k3[start..start + 3]
            }
            ArityGroup::Var => {
                let start = self.tail_off_var[*group_index as usize] as usize;
                let end = self.tail_off_var[*group_index as usize + 1] as usize;
                &self.tails_var[start..end]
            }
        }
    }

    pub fn edge_indices(&self) -> impl Iterator<Item = EdgeIx> {
        0..self.n_edges
    }
}
