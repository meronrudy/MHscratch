pub mod brute_force;

use crate::store::Point;
use core::ids::NodeIx;

pub trait SpatialIndex {
    fn rebuild(&mut self, points: &[Point]) -> usize;
    fn radius_query(&self, center: &Point, r: f32, out: &mut Vec<NodeIx>);
}
