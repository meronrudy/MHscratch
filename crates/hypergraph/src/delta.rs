use core::ids::{EdgeIx, NodeIx};
use std::collections::BTreeMap;

// A compact representation of the difference between two frozen graphs.
#[derive(Clone, Debug, Default)]
pub struct FrozenDelta {
    // TODO: This is a simplified version. A real implementation would need to
    // handle changes to edge data, not just additions and removals.
    pub added_edges: Vec<EdgeIx>,
    pub removed_edges: Vec<EdgeIx>,

    // For nodes whose adjacency lists have changed, we store the new list.
    // This is a map from the node index to the new list of edges.
    // We use a BTreeMap to ensure that the order is deterministic.
    pub patched_in_adjacency: BTreeMap<NodeIx, Box<[EdgeIx]>>,
    pub patched_out_adjacency: BTreeMap<NodeIx, Box<[EdgeIx]>>,
}
