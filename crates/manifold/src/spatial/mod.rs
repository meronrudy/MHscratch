
use core::ids::NodeIx;
use nalgebra::Point;

pub mod kdtree;

pub trait SpatialIndex<const D: usize> {
    fn rebuild(&mut self, points: &[(NodeIx, Point<f32, D>)]);
    fn radius_query(&self, center: &Point<f32, D>, r: f32) -> Vec<NodeIx>;
}
