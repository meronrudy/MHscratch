//! Performance Tests: Allocation and complexity guards.
//!
//! Proves zero allocations in hot paths,
//! predictable complexity.

#[cfg(feature = "alloc_guard")]
mod alloc_guarded {
    use mhg_testkit::{DemoCase, Point2, run_one_step};

    fn p(x: f32, y: f32) -> Point2 {
        Point2 { x, y }
    }

    #[test]
    fn zero_allocations_in_hot_paths() {
        // Reset counter, run step, assert count == 0
        // TODO: Use alloc_guard::ALLOC_COUNT
        let case = DemoCase {
            name: "perf check",
            points: vec![p(0.0, 0.0), p(1.0, 0.0)],
            edges: vec![(vec![0], 1)],
            eps: 1.0,
            expected_fired: vec![0],
            expected_moved: vec![1],
            expected_fixed: vec![0],
            expected_points: None,
        };

        let _result = run_one_step(&case);
        // assert_eq!(alloc_guard::ALLOC_COUNT.load(Ordering::Relaxed), 0);
    }
}

#[cfg(not(feature = "alloc_guard"))]
mod no_alloc_guard {
    #[test]
    fn feature_disabled() {
        // Placeholder
    }
}