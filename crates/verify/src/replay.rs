// use exec::event::EventLog;

// pub fn replay(m: &mut ManifoldStore, log: &EventLog) {
//     // Replay ignores gate + operator entirely; it applies recorded deltas.
//     // This is what makes it a robust determinism harness.
//     for d in &log.deltas {
//         m.apply_delta(d.node, d.dx(), d.dy());
//     }
// }
