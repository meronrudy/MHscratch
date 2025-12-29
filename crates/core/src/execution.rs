
use crate::ids::{EdgeIx, NodeIx};

// TODO: Move to a more appropriate crate.
// TODO: Add a `priority` field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    pub time: u64,
    pub node: NodeIx,
    pub edge: Option<EdgeIx>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FireMode {
    Push,
    Pull,
}
