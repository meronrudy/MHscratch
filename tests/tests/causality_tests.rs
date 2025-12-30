//! Causality Tests: Local invariants that defend the GoalProofCase.
//!
//! These ensure only edges that should fire can fire,
//! only nodes that are causally downstream can move,
//! and nothing else moves "by accident".

use mhg_testkit::{assert_set_eq, DemoCase, Point2, run_one_step};

fn p(x: f32, y: f32) -> Point2 {
    Point2 { x, y }
}

#[test]
fn non_firing_guarantees() {
    // Case where gate should not fire (distance > threshold)
    let case = DemoCase {
        name: "non-firing",
        points: vec![p(0.0, 0.0), p(2.0, 0.0)], // distance 2.0 > eps 1.0
        edges: vec![(vec![0], 1)],
        eps: 1.0,
        expected_fired: vec![], // should not fire
        expected_moved: vec![], // no movement
        expected_fixed: vec![0, 1], // both fixed
        expected_points: None,
    };

    let result = run_one_step(&case);
    assert_set_eq(result.fired_edges, case.expected_fired);
    assert_set_eq(result.moved_nodes, case.expected_moved);
    assert_eq!(result.manifold_epoch, 0); // epoch should not advance
}

#[test]
fn minimal_sufficiency() {
    // Case where gate should fire (distance < threshold)
    let case = DemoCase {
        name: "minimal sufficiency",
        points: vec![p(0.0, 0.0), p(0.5, 0.0)], // distance 0.5 < eps 1.0
        edges: vec![(vec![0], 1)],
        eps: 1.0,
        expected_fired: vec![0],
        expected_moved: vec![1],
        expected_fixed: vec![0],
        expected_points: None,
    };

    let result = run_one_step(&case);
    assert_set_eq(result.fired_edges, case.expected_fired);
    assert_set_eq(result.moved_nodes, case.expected_moved);
}

#[test]
fn no_side_effects() {
    // Ensure only heads of fired edges move
    let case = DemoCase {
        name: "no side effects",
        points: vec![p(0.0, 0.0), p(1.0, 0.0), p(2.0, 0.0)], // node 2 unrelated
        edges: vec![(vec![0], 1)], // only moves node 1
        eps: 1.0,
        expected_fired: vec![0],
        expected_moved: vec![1],
        expected_fixed: vec![0, 2], // node 2 must remain fixed
        expected_points: Some(vec![p(0.0, 0.0), p(1.1, 0.0), p(2.0, 0.0)]), // only node 1 changes
    };

    let result = run_one_step(&case);
    assert_set_eq(result.fired_edges, case.expected_fired);
    assert_set_eq(result.moved_nodes, case.expected_moved);
    if let Some(exp_points) = &case.expected_points {
        mhg_testkit::assert_points_close(&result.points, exp_points, case.eps);
    }
}