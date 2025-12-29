pub mod frozen;
pub mod dynamic;
pub mod freeze;
pub mod compact;
pub mod remap;
pub mod delta;
pub mod arity;
pub mod signature;

use std::ops::ControlFlow;

pub use frozen::HypergraphFrozen as FrozenHypergraph;
pub use core::ids::{EdgeIx as EdgeIndex, NodeIx as NodeIndex};

pub type FrozenBase = frozen::HypergraphFrozen;

pub trait FrozenHypergraphView<'a> {
    type Iter: Iterator<Item = (NodeIndex, EdgeIndex, i8)> + 'a;

    fn new_view(hypergraph: &'a FrozenHypergraph) -> Self;

    fn iter_view(&self) -> Self::Iter;

    fn for_each<F>(&self, f: F) -> ControlFlow<()>
    where
        F: FnMut((NodeIndex, EdgeIndex, i8)) -> ControlFlow<()>;
}
