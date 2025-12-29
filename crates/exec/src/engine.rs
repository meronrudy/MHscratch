use crate::delta::{Delta, DeltaBuf};
use crate::fire::eval_edge;
use crate::queue::EventQueue;
use crate::views::{FrozenGraphView, GateView, ManifoldView};
use rayon::prelude::*;

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
        G: FrozenGraphView + Sync,
        M: ManifoldView + Sync,
        M::Tangent: Send,
    {
        let mut edges = Vec::new();
        while let Some(ev) = self.q.pop() {
            if let Some(e) = ev.edge {
                edges.push(e);
            }
        }

        let computed_deltas: Vec<Delta<M::Tangent>> = edges
            .into_par_iter()
            .map(|e| eval_edge(e, graph, gate, mani))
            .collect::<Vec<_>>();

        for delta in computed_deltas {
            deltas.push(delta);
        }
    }
}
