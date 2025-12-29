use core::ids::EdgeIx;

#[derive(Clone, Debug)]
pub struct FrozenDelta {
    pub added_edges: Vec<EdgeIx>,
    pub removed_edges: Vec<EdgeIx>,
    // More complex changes like patched adjacency can be added later
}
