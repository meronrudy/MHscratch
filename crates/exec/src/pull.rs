use crate::queue::EventQueue;
use crate::views::FrozenGraphView;
use std::vec::Vec;
use core::ids::{EdgeIx, NodeIx};

pub fn schedule_pull<G: FrozenGraphView>(
    graph: &G,
    head: NodeIx,
    time: u64,
    priority: u32,
    q: &mut EventQueue,
    scratch_edges: &mut Vec<EdgeIx>,
) {
    scratch_edges.clear();
    graph.incoming_edges(head, scratch_edges);

    for &e in scratch_edges.iter() {
        q.push(time, priority, head, Some(e));
    }
}
