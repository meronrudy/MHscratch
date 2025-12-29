use core::ids::{EdgeIx, NodeIx};
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub time: u64,
    pub priority: u32,
    pub node: NodeIx,
    pub edge: Option<EdgeIx>,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // Note: this is a min-heap, so we reverse the comparison.
        other.time.cmp(&self.time)
            .then_with(|| other.priority.cmp(&self.priority))
            .then_with(|| other.node.cmp(&self.node))
            .then_with(|| other.edge.cmp(&self.edge))
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

