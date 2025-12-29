pub mod frozen;
pub mod dynamic;
pub mod freeze;
pub mod compact;
pub mod remap;
pub mod delta;
pub mod arity;
pub mod signature;

pub use frozen::HypergraphFrozen;

use std::ops::ControlFlow;

pub use core::ids::{EdgeIx as EdgeIndex, NodeIx as NodeIndex};

pub trait FrozenHypergraphView<'a> {
    type Iter: Iterator<Item = (NodeIndex, EdgeIndex, f64)> + 'a;

    fn new_view(hypergraph: &'a frozen::HypergraphFrozen) -> Self;

    fn iter_view(&self) -> Self::Iter;

    fn for_each<F>(&self, f: F) -> ControlFlow<()>
    where
        F: FnMut((NodeIndex, EdgeIndex, f64)) -> ControlFlow<()>;
}

pub trait Vertex {}

pub trait Operator<V: Vertex> {
    fn eval(&self, vertices: &[V]) -> V;
}
