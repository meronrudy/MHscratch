use core::ids::NodeIx;
use crate::store::Point;

use nalgebra::Point3;

pub enum EdgeFootprint {
    V1(EdgeFootprintV1),
    V2_2(EdgeFootprintV2<2>),
    V2_3(EdgeFootprintV2<3>),
}

pub struct EdgeFootprintV1 {
    pub influence_radius: f32,
    pub anchor: Option<NodeIx>,
}


pub struct EdgeFootprintV2<const K: usize> {
    pub simplex_proxy: [NodeIx; K],
    pub cached_centroid: Point,
    pub influence_radius: f32,
}

impl<const K: usize> EdgeFootprintV2<K> {
    pub fn new(simplex_proxy: [NodeIx; K], points: &[Point]) -> Self {
        let mut centroid = Point3::origin();
        let mut max_dist_sq = 0.0;

        for &node_ix in simplex_proxy.iter() {
            let point = &points[node_ix as usize];
            centroid += point.coords;
        }
        let centroid = centroid / (K as f32);

        for &node_ix in simplex_proxy.iter() {
            let point = &points[node_ix as usize];
            let dist_sq = nalgebra::distance_squared(point, &centroid);
            if dist_sq > max_dist_sq {
                max_dist_sq = dist_sq;
            }
        }

        Self {
            simplex_proxy,
            cached_centroid: centroid,
            influence_radius: max_dist_sq.sqrt(),
        }
    }
}
