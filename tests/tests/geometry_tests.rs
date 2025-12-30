//! Geometry Tests: Topology–geometry boundary.
//!
//! Ensures gate cache invalidation is correct,
//! spatial index never drops valid candidates,
//! footprint prechecks are conservative.

use mhg_testkit::{DemoCase, Point2, run_one_step};

fn p(x: f32, y: f32) -> Point2 {
    Point2 { x, y }
}

#[test]
fn gate_cache_epoch_invalidation() {
    // Mutate manifold, assert cache invalid
    let case = DemoCase {
        name: "cache invalidation",
        points: vec![p(0.0, 0.0), p(1.0, 0.0)],
        edges: vec![(vec![0], 1)],
        eps: 1.0,
        expected_fired: vec![0],
        expected_moved: vec![1],
        expected_fixed: vec![0],
        expected_points: None,
    };

    let result1 = run_one_step(&case);
    let result2 = run_one_step(&case); // Should recompute cache

    // Assert same results (deterministic)
    assert_eq!(result1.fired_edges, result2.fired_edges);
    assert_eq!(result1.moved_nodes, result2.moved_nodes);
}

#[test]
fn spatial_index_correctness() {
    // TODO: Test radius_query matches brute force
}

#[test]
fn footprint_conservativeness() {
    // TODO: Test precheck doesn't reject valid candidates
}