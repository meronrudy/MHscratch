use std::cmp::Ordering;

pub use core::ids::{EdgeIx, NodeIx};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Event {
    pub time: u64,
    pub priority: u32,
    pub node: NodeIx,
    pub edge: Option<EdgeIx>,
    pub seq: u64, // tie-breaker for total order
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // Deterministic ordering using (time, priority, node, edge) as total order
        self.time
            .cmp(&other.time)
            .then(self.priority.cmp(&other.priority))
            .then(self.node.cmp(&other.node))
            .then(self.edge.cmp(&other.edge))
            .then(self.seq.cmp(&other.seq))
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
