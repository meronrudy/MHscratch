use mhg_testkit::{assert_points_close, assert_set_eq, DemoCase, run_one_step};

fn demo_a() -> DemoCase {
    DemoCase {
        name: "single edge, head moves, tails fixed",
        points: vec![
            mhg_testkit::Point2 { x: 0.0, y: 0.0 }, // node 0 (tail)
            mhg_testkit::Point2 { x: 1.0, y: 0.0 }, // node 1 (head)
        ],
        edges: vec![(vec![0], 1)], // edge 0: [0] -> 1
        eps: 1e-6,
        expected_fired: vec![0],
        expected_moved: vec![1],
        expected_fixed: vec![0],
        expected_points: None, // no exact geometry check
    }
}

const DEMO_B: DemoCase = DemoCase {
    name: "chain propagation",
    points: vec![
        mhg_testkit::Point2 { x: 0.0, y: 0.0 }, // node 0
        mhg_testkit::Point2 { x: 1.0, y: 0.0 }, // node 1
        mhg_testkit::Point2 { x: 2.0, y: 0.0 }, // node 2
    ],
    edges: vec![
        (vec![0], 1), // edge 0: [0] -> 1
        (vec![1], 2), // edge 1: [1] -> 2
    ],
    eps: 1e-6,
    expected_fired: vec![0, 1], // depending on push mode
    expected_moved: vec![1, 2],
    expected_fixed: vec![0],
    expected_points: None,
};

const DEMO_C: DemoCase = DemoCase {
    name: "fork one tail influences two heads",
    points: vec![
        mhg_testkit::Point2 { x: 0.0, y: 0.0 }, // node 0 (tail)
        mhg_testkit::Point2 { x: 1.0, y: 0.0 }, // node 1 (head)
        mhg_testkit::Point2 { x: 0.0, y: 1.0 }, // node 2 (head)
    ],
    edges: vec![
        (vec![0], 1), // edge 0: [0] -> 1
        (vec![0], 2), // edge 1: [0] -> 2
    ],
    eps: 1e-6,
    expected_fired: vec![0, 1],
    expected_moved: vec![1, 2],
    expected_fixed: vec![0],
    expected_points: None,
};

const DEMO_D: DemoCase = DemoCase {
    name: "diamond two causes converge",
    points: vec![
        mhg_testkit::Point2 { x: 0.0, y: 0.0 }, // node 0
        mhg_testkit::Point2 { x: 2.0, y: 0.0 }, // node 1
        mhg_testkit::Point2 { x: 1.0, y: 1.0 }, // node 2
    ],
    edges: vec![
        (vec![0], 2), // edge 0: [0] -> 2
        (vec![1], 2), // edge 1: [1] -> 2
    ],
    eps: 1e-6,
    expected_fired: vec![0, 1],
    expected_moved: vec![2],
    expected_fixed: vec![0, 1],
    expected_points: None,
};

const DEMO_E: DemoCase = DemoCase {
    name: "non-firing just outside threshold",
    points: vec![
        mhg_testkit::Point2 { x: 0.0, y: 0.0 }, // node 0
        mhg_testkit::Point2 { x: 1.1, y: 0.0 }, // node 1 (distance > threshold)
    ],
    edges: vec![(vec![0], 1)], // assume radius gate with r=1.0
    eps: 1e-6,
    expected_fired: vec![],
    expected_moved: vec![],
    expected_fixed: vec![0, 1],
    expected_points: None,
};

const DEMO_F: DemoCase = DemoCase {
    name: "minimal sufficiency at threshold",
    points: vec![
        mhg_testkit::Point2 { x: 0.0, y: 0.0 }, // node 0
        mhg_testkit::Point2 { x: 0.9, y: 0.0 }, // node 1 (distance < threshold)
    ],
    edges: vec![(vec![0], 1)], // radius gate with r=1.0
    eps: 1e-6,
    expected_fired: vec![0],
    expected_moved: vec![1],
    expected_fixed: vec![0],
    expected_points: None,
};

const DEMO_G: DemoCase = DemoCase {
    name: "no side-effects",
    points: vec![
        mhg_testkit::Point2 { x: 0.0, y: 0.0 }, // node 0
        mhg_testkit::Point2 { x: 1.0, y: 0.0 }, // node 1
        mhg_testkit::Point2 { x: 2.0, y: 0.0 }, // node 2 (unrelated)
    ],
    edges: vec![(vec![0], 1)], // edge 0: [0] -> 1
    eps: 1e-6,
    expected_fired: vec![0],
    expected_moved: vec![1],
    expected_fixed: vec![0, 2],
    expected_points: Some(vec![
        mhg_testkit::Point2 { x: 0.0, y: 0.0 },
        mhg_testkit::Point2 { x: 2.0, y: 0.0 }, // expect no change
        mhg_testkit::Point2 { x: 2.0, y: 0.0 },
    ]),
};

const DEMO_H: DemoCase = DemoCase {
    name: "gate weight affects magnitude but not topology",
    points: vec![
        mhg_testkit::Point2 { x: 0.0, y: 0.0 }, // node 0
        mhg_testkit::Point2 { x: 1.0, y: 0.0 }, // node 1
    ],
    edges: vec![(vec![0], 1)], // assume weighted gate
    eps: 1e-6,
    expected_fired: vec![0],
    expected_moved: vec![1],
    expected_fixed: vec![0],
    expected_points: None, // different weights -> different deltas, but same fired
};

const ALL_DEMOS: &[&DemoCase] = &[
    &DEMO_A,
    &DEMO_B,
    &DEMO_C,
    &DEMO_D,
    &DEMO_E,
    &DEMO_F,
    &DEMO_G,
    &DEMO_H,
];

#[test]
fn dummy_test() {
    assert_eq!(1 + 1, 2);
}

#[test]
fn demo_a() {
    let result = run_one_step(&DEMO_A);
    assert_set_eq(result.fired_edges, DEMO_A.expected_fired.clone());
    assert_set_eq(result.moved_nodes, DEMO_A.expected_moved.clone());
    assert_set_eq(result.expected_fixed, DEMO_A.expected_fixed.clone());
    if let Some(exp_points) = &DEMO_A.expected_points {
        assert_points_close(&result.points, exp_points, DEMO_A.eps);
    }
}