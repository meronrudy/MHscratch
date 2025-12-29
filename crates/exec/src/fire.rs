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
    event_seq: u64,
    out: &mut DeltaBuf<M::Tangent>,
) where
    G: FrozenGraphView,
    M: ManifoldView,
{
    if gate.edge_active[e as usize] == 0 {
        return;
    }

    let head = graph.edge_head(e);
    let _head_p = mani.point(head);

    // Example: move head toward average tail point (toy, replace later)
    let tails = graph.edge_tails(e);
    if tails.is_empty() {
        return;
    }

    // You define Tangent algebra; this stays placeholder.
    // Use a separate trait if you want generic tangent ops.
    let _dummy = M::Tangent::default();

    // Emit a delta (placeholder tangent)
    out.push(Delta::PointUpdate {
        node: head,
        tangent: _dummy,
        source_edge: e,
        seq: event_seq,
    });
}
