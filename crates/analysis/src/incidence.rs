use std::ops::ControlFlow;

use hypergraph::{
    frozen::HypergraphFrozen,
    EdgeIndex,
    NodeIndex,
    FrozenHypergraphView,
};

use crate::incidence_iter::{iter_incidences, IncidenceIter};

/// An incidence view of a hypergraph.
///
/// This provides an iterator over the `(node, edge, sign)` tuples of the
/// hypergraph, where `sign` is `1` for outputs and `-1` for inputs.
pub struct IncidenceView<'a> {
    hypergraph: &'a HypergraphFrozen,
}

impl<'a> FrozenHypergraphView<'a> for IncidenceView<'a> {
    type Iter = IncidenceIter<'a>;

    fn new_view(hypergraph: &'a HypergraphFrozen) -> Self {
        Self { hypergraph }
    }

    fn iter_view(&self) -> Self::Iter {
        iter_incidences(self.hypergraph)
    }

    fn for_each<F>(&self, mut f: F) -> ControlFlow<()>
    where
        F: FnMut((NodeIndex, EdgeIndex, i8)) -> ControlFlow<()>,
    {
        for item in self.iter_view() {
            if f(item).is_break() {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}
