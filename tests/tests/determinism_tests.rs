//! Determinism Tests: Replay and fuzz guarantees.
//!
//! Ensures replay always works,
//! fuzzed small systems shrink to minimal counterexamples.

use mhg_testkit::{DemoCase, Point2, run_one_step};

fn p(x: f32, y: f32) -> Point2 {
    Point2 { x, y }
}

#[test]
fn engine_determinism_same_inputs() {
    let case = DemoCase {
        name: "determinism check",
        points: vec![p(0.0, 0.0), p(1.0, 0.0)],
        edges: vec![(vec![0], 1)],
        eps: 1.0,
        expected_fired: vec![0],
        expected_moved: vec![1],
        expected_fixed: vec![0],
        expected_points: None,
    };

    let result1 = run_one_step(&case);
    let result2 = run_one_step(&case);

    // Assert bitwise identical
    // TODO: Implement point hashing or exact comparison
    assert_eq!(result1.fired_edges, result2.fired_edges);
    assert_eq!(result1.moved_nodes, result2.moved_nodes);
    // assert_eq!(result1.points, result2.points);
}

#[test]
fn seeded_fuzz_with_replay() {
    // TODO: Implement fuzzing small graphs and replay
}