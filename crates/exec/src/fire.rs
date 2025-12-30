use crate::delta::{Delta, DeltaBuf};
use crate::views::{FrozenGraphView, GateView, ManifoldView};
use core::ids::EdgeIx;

#[derive(Clone, Copy, Debug)]
pub enum FireMode {
    Push,
    Pull,
}

pub fn eval_edge<G, M>(
    e: EdgeIx,
    graph: &G,
    gate: &GateView<'_>,
    mani: &M,
) -> Delta<M::Tangent> where
    G: FrozenGraphView,
    M: ManifoldView,
{
    if gate.edge_active[e as usize] == 0 {
        return Delta::PointUpdate(graph.edge_head(e), M::Tangent::default()); // placeholder
    }

    let head = graph.edge_head(e);
    let _head_p = mani.point(head);

    // Simple demo: move head toward first tail by 0.1 in x direction
    let tails = graph.edge_tails(e);
    if tails.is_empty() {
        return Delta::PointUpdate(head, M::Tangent::default());
    }

    // Create a simple tangent (move in x by 0.1)
    let tangent = M::make_tangent(0.1, 0.0, 0.0);

    Delta::PointUpdate(head, tangent)
}
