use crate::delta::DeltaBuf;
use crate::fire::eval_edge;
use crate::queue::EventQueue;
use crate::views::{FrozenGraphView, GateView, ManifoldView};

pub struct Engine {
    pub q: EventQueue,
}

impl Engine {
    pub fn with_capacity(cap: usize) -> Self {
        Self { q: EventQueue::with_capacity(cap) }
    }

    pub fn run_compute<G, M>(
        &mut self,
        graph: &G,
        gate: &GateView<'_>,
        mani: &M,
        deltas: &mut DeltaBuf<M::Tangent>,
    )
    where
        G: FrozenGraphView,
        M: ManifoldView,
    {
        while let Some(ev) = self.q.pop() {
            if let Some(e) = ev.key.edge {
                eval_edge(e, graph, gate, mani, ev.key.seq, deltas);
            }
        }
    }
}
