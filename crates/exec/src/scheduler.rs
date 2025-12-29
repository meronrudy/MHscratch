// use bridge::gate::DistanceGate;
// use crate::event::{EventLog, FiredEdge, DeltaEvent};
// use crate::fire::eval_edge_delta;

/*
pub fn execute_with_trace(m: &mut ManifoldStore, g: &HypergraphFrozen, gate: DistanceGate) -> EventLog {
    // Read-only phase: decide what fires and compute deltas.
    let mut fired = Vec::<FiredEdge>::new();
    let mut deltas = Vec::<DeltaEvent>::new();

    for e in 0..(g.n_edges as u32) {
        if gate.allow(m, g, e) {
            fired.push(FiredEdge { edge: e });

            let (node, dx, dy) = eval_edge_delta(m, g, e);
            deltas.push(DeltaEvent::new(node, dx, dy));
        }
    }

    // Apply phase (single writer).
    for d in &deltas {
        m.apply_delta(d.node, d.dx(), d.dy());
    }

    EventLog { fired_edges: fired, deltas }
}
*/
