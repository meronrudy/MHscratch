use crate::queue::EventQueue;
use crate::views::FrozenGraphView;
use std::vec::Vec;
use core::ids::{EdgeIx, NodeIx};

pub struct DirtySet {
    pub nodes: Vec<NodeIx>, // start simple; swap to BitSet later
}

pub fn schedule_push<G: FrozenGraphView>(
    graph: &G,
    dirty: &DirtySet,
    time: u64,
    priority: u32,
    q: &mut EventQueue,
    scratch_edges: &mut Vec<EdgeIx>,
) {
    for &u in &dirty.nodes {
        scratch_edges.clear();
        graph.incident_edges(u, scratch_edges);

        for &e in scratch_edges.iter() {
            // Schedule an edge firing event. node can be the dirty node or head; you choose.
            q.push(time, priority, u, Some(e));
        }
    }
}
