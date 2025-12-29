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

    // Example: move head toward average tail point (toy, replace later)
    let tails = graph.edge_tails(e);
    if tails.is_empty() {
        return Delta::PointUpdate(head, M::Tangent::default());
    }

    // You define Tangent algebra; this stays placeholder.
    // Use a separate trait if you want generic tangent ops.
    let _dummy = M::Tangent::default();

    // Emit a delta (placeholder tangent)
    Delta::PointUpdate(head, _dummy)
}
