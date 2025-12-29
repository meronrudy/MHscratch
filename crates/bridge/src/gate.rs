use manifold::footprint::{EdgeFootprint, EdgeFootprintV1, EdgeFootprintV2};
use manifold::store::{ManifoldStore, Point};
use manifold::spatial::SpatialIndex;
use nalgebra::distance;
use core::ids::NodeIx;

pub struct Gate<S: SpatialIndex> {
    pub index: S,
    pub last_rebuild_cost: usize,
    rebuild_amortization: usize,
    nodes_since_rebuild: usize,
}

impl<S: SpatialIndex> Gate<S> {
    pub fn new(index: S, rebuild_amortization: usize) -> Self {
        Self {
            index,
            last_rebuild_cost: 0,
            rebuild_amortization,
            nodes_since_rebuild: 0,
        }
    }

    pub fn note_node_added(&mut self) {
        self.nodes_since_rebuild += 1;
    }

    pub fn rebuild_index_if_needed<I: manifold::store::Integrator>(
        &mut self,
        manifold: &ManifoldStore<I>,
    ) {
        if self.nodes_since_rebuild >= self.rebuild_amortization {
            self.last_rebuild_cost = self.index.rebuild(&manifold.points);
            self.nodes_since_rebuild = 0;
        }
    }

    pub fn find_candidates(
        &self,
        footprint: &EdgeFootprint,
        manifold: &ManifoldStore<impl manifold::store::Integrator>,
        candidates: &mut Vec<NodeIx>,
    ) {
        match footprint {
            EdgeFootprint::V1(fp) => self.find_candidates_v1(fp, manifold, candidates),
            EdgeFootprint::V2_2(fp) => self.find_candidates_v2(fp, candidates),
            EdgeFootprint::V2_3(fp) => self.find_candidates_v2(fp, candidates),
        }
    }

    pub fn pre_check(
        footprint: &EdgeFootprint,
        point: &Point,
        manifold: &ManifoldStore<impl manifold::store::Integrator>,
    ) -> bool {
        match footprint {
            EdgeFootprint::V1(fp) => Self::pre_check_v1(fp, point, manifold),
            EdgeFootprint::V2_2(fp) => Self::pre_check_v2(fp, point),
            EdgeFootprint::V2_3(fp) => Self::pre_check_v2(fp, point),
        }
    }

    pub fn find_candidates_v1(
        &self,
        footprint: &EdgeFootprintV1,
        manifold: &ManifoldStore<impl manifold::store::Integrator>,
        candidates: &mut Vec<NodeIx>,
    ) {
        if let Some(anchor_ix) = footprint.anchor {
            let anchor_point = &manifold.points[anchor_ix as usize];
            self.index.radius_query(anchor_point, footprint.influence_radius, candidates);
        }
    }

    pub fn find_candidates_v2<const K: usize>(
        &self,
        footprint: &EdgeFootprintV2<K>,
        candidates: &mut Vec<NodeIx>,
    ) {
        self.index.radius_query(&footprint.cached_centroid, footprint.influence_radius, candidates);
    }

    pub fn pre_check_v1<I: manifold::store::Integrator>(
        footprint: &EdgeFootprintV1,
        point: &Point,
        manifold: &ManifoldStore<I>,
    ) -> bool {
        if let Some(anchor_ix) = footprint.anchor {
            let anchor_point = &manifold.points[anchor_ix as usize];
            distance(point, anchor_point) <= footprint.influence_radius
        } else {
            true
        }
    }

    pub fn pre_check_v2<const K: usize>(
        footprint: &EdgeFootprintV2<K>,
        point: &Point,
    ) -> bool {
        distance(point, &footprint.cached_centroid) <= footprint.influence_radius
    }
}
