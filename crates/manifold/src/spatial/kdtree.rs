use crate::spatial::SpatialIndex;
use core::ids::NodeIx;
use kdtree::distance::squared_euclidean;
use kdtree::KdTree as KdTreeImpl;
use nalgebra::Point;
use std::collections::HashMap;

pub struct KdTree<const D: usize> {
    tree: KdTreeImpl<f32, NodeIx, Vec<f32>>,
    points: HashMap<NodeIx, Point<f32, D>>,
}

impl<const D: usize> Default for KdTree<D> {
    fn default() -> Self {
        Self {
            tree: KdTreeImpl::new(D),
            points: HashMap::new(),
        }
    }
}

impl<const D: usize> SpatialIndex<D> for KdTree<D> {
    fn rebuild(&mut self, points: &[(NodeIx, Point<f32, D>)]) {
        self.points.clear();
        self.tree = KdTreeImpl::new(D);
        for (node_ix, point) in points {
            self.points.insert(*node_ix, *point);
            self.tree.add(point.coords.as_slice().to_vec(), *node_ix).unwrap();
        }
    }

    fn radius_query(&self, center: &Point<f32, D>, r: f32) -> Vec<NodeIx> {
        let radius_sq = r * r;
        self.tree
            .within(center.coords.as_slice(), radius_sq, &squared_euclidean)
            .unwrap()
            .into_iter()
            .map(|(_dist, node_ix)| *node_ix)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point;

    #[test]
    fn test_radius_query() {
        let mut index = KdTree::<2>::default();
        let points = vec![
            (0, Point::from([0.0, 0.0])),
            (1, Point::from([1.0, 1.0])),
            (2, Point::from([2.0, 2.0])),
            (3, Point::from([3.0, 3.0])),
        ];
        index.rebuild(&points);

        let center = Point::from([0.5, 0.5]);
        let radius = 1.0;
        let mut result = index.radius_query(&center, radius);
        result.sort();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 1);
    }
}
