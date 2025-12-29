use std::cmp::Ordering;

pub use core::ids::{EdgeIx, NodeIx};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventKey {
    pub time: u64,
    pub priority: u32,
    pub node: NodeIx,
    pub edge: Option<EdgeIx>,
    pub seq: u64, // tie-breaker, FIFO when everything else equals
}

impl Ord for EventKey {
    fn cmp(&self, other: &Self) -> Ordering {
        // NOTE: BinaryHeap is max-heap; we use Reverse<EventKey> later.
        // So this is the natural ascending order.
        self.time
            .cmp(&other.time)
            .then(self.priority.cmp(&other.priority))
            .then(self.node.cmp(&other.node))
            .then(self.edge.cmp(&other.edge))
            .then(self.seq.cmp(&other.seq))
    }
}

impl PartialOrd for EventKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Event {
    pub key: EventKey,
    // Keep payload tiny; add more fields later if needed.
}
