// // golden_replay.rs
// 
// use std::fs;
// use std::path::Path;
// 
// use bridge::gate::DistanceGate;
// use core::ids::{EdgeIx, NodeIx};
// use exec::event::EventLog;
// use exec::scheduler::execute_with_trace;
// use hypergraph::frozen::HypergraphFrozen;
// use manifold::store::{ManifoldStore, Point2};
// use verify::hash::state_hash;
// use verify::replay::replay;
// 
// // A minimal scenario builder
// fn build_scenario() -> (ManifoldStore, HypergraphFrozen, DistanceGate) {
//     // Nodes: A=0, B=1, C=2
//     let manifold = ManifoldStore {
//         epoch: 0,
//         node_manifold: vec![0, 0, 0],
//         point_x: vec![0.0, 0.2, 5.0],
//         point_y: vec![0.0, 0.0, 5.0],
//         manifold_params: vec![],
//     };
// 
//     // One edge: (A,B) -> C
//     let frozen = HypergraphFrozen {
//         n_nodes: 3,
//         n_edges: 1,
//         edge_head: vec![2],
//         edge_kind: vec![],
//         edge_info: vec![(hypergraph::arity::ArityGroup::K2, 0)],
//         tails_k2: vec![0, 1],
//         edges_k2: vec![0],
//         tails_k3: vec![],
//         edges_k3: vec![],
//         tail_off_var: vec![0],
//         tails_var: vec![],
//         edges_var: vec![],
//         out_off: vec![],
//         out_edges: vec![],
//         in_off: vec![],
//         in_edges: vec![],
//     };
// 
//     let gate = DistanceGate { eps: 0.5 };
//     (manifold, frozen, gate)
// }
// 
// #[test]
// fn golden_replay_demo() {
//     let (mut m0, g, gate) = build_scenario();
//     let initial = m0.clone();
// 
//     // 1) Execute and record
//     let log = execute_with_trace(&mut m0, &g, gate);
//     let final_hash_run = state_hash(&m0);
// 
//     // 2) Golden compare
//     let golden_path = Path::new("golden/demo_step1.golden");
//     assert_or_write_golden(&golden_path, &log);
// 
//     // 3) Reset and replay the same log
//     let mut m1 = initial;
//     replay(&mut m1, &log);
//     let final_hash_replay = state_hash(&m1);
// 
//     // 4) Final state must match
//     assert_eq!(final_hash_run, final_hash_replay, "replay diverged");
// 
//     // Optional: assert a few “semantic” expectations too
//     assert!(!log.fired_edges.is_empty(), "expected edge to fire");
//     assert_eq!(log.fired_edges[0].edge, 0, "expected edge 0");
// }
// 
// // Golden harness helpers
// fn assert_or_write_golden(path: &Path, got: &EventLog) {
//     let text = got.to_golden_text();
// 
//     if path.exists() {
//         let expected_text = fs::read_to_string(path).unwrap();
//         let expected = EventLog::from_golden_text(&expected_text);
//         assert_eq!(&expected, got, "golden mismatch: {}", path.display());
//     } else {
//         // In CI you typically forbid writes; locally you can allow it.
//         fs::write(path, text).unwrap();
//         panic!(
//             "golden file did not exist; wrote it at {}. Re-run tests.",
//             path.display()
//         );
//     }
// }
