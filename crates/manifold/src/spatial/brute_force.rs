use crate::store::Point;
use crate::spatial::SpatialIndex;
use core::ids::NodeIx;

#[cfg(feature = "perf")]
use crate::simd::dist_simd;
#[cfg(not(feature = "perf"))]
use nalgebra::distance;

// A simple brute-force spatial index for correctness testing.
pub struct BruteForceIndex {
    points: Vec<Point>,
}

impl BruteForceIndex {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }
}

impl SpatialIndex for BruteForceIndex {
    fn rebuild(&mut self, points: &[Point]) -> usize {
        self.points = points.to_vec();
        points.len()
    }

    fn radius_query(&self, center: &Point, r: f32, out: &mut Vec<NodeIx>) {
        out.clear();
        for (i, p) in self.points.iter().enumerate() {
            #[cfg(feature = "perf")]
            let dist = dist_simd(p, center);
            #[cfg(not(feature = "perf"))]
            let dist = distance(p, center);
            if dist <= r {
                out.push(i as NodeIx);
            }
        }
    }
}
