//! Metamorphic Tests: Invariance under transformations.
//!
//! Ensures translation doesn't matter,
//! node IDs are labels,
//! insertion order irrelevant.

use mhg_testkit::{DemoCase, Point2, run_one_step};

fn p(x: f32, y: f32) -> Point2 {
    Point2 { x, y }
}

#[test]
fn translation_invariance() {
    // Translate all points by (10, 10), run, translate back, assert same
    let base_case = DemoCase {
        name: "translation base",
        points: vec![p(0.0, 0.0), p(1.0, 0.0)],
        edges: vec![(vec![0], 1)],
        eps: 1.0,
        expected_fired: vec![0],
        expected_moved: vec![1],
        expected_fixed: vec![0],
        expected_points: None,
    };

    let translated_points: Vec<Point2> = base_case.points.iter().map(|pt| Point2 { x: pt.x + 10.0, y: pt.y + 10.0 }).collect();
    let translated_case = DemoCase {
        points: translated_points,
        ..base_case
    };

    let base_result = run_one_step(&base_case);
    let translated_result = run_one_step(&translated_case);

    // Translate result back and compare
    let back_translated: Vec<Point2> = translated_result.points.iter().map(|pt| p(pt.x - 10.0, pt.y - 10.0)).collect();

    mhg_testkit::assert_points_close(&base_result.points, &back_translated, 1e-5);
    assert_eq!(base_result.fired_edges, translated_result.fired_edges);
    assert_eq!(base_result.moved_nodes, translated_result.moved_nodes);
}

#[test]
fn node_renumbering_invariance() {
    // TODO: Permute node IDs, remap edges, assert equivalent results
}

#[test]
fn edge_insertion_order_invariance() {
    // TODO: Insert edges in different order, assert same frozen graph
}