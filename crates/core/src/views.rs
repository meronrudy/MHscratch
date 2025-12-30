use crate::ids::EdgeIx;
use crate::ids::NodeIx;

pub struct GateView<'a> {
    pub edge_active: &'a [u8],
    pub edge_weight: &'a [f32],
}

pub trait FrozenGraphView {
    fn edge_count(&self) -> usize;

    // Minimal accessors; adapt to your storage.
    fn edge_tails(&self, e: EdgeIx) -> &[NodeIx];
    fn edge_head(&self, e: EdgeIx) -> NodeIx;

    // Incidence needed for push/pull without scans.
    fn incident_edges(&self, node: NodeIx, out: &mut Vec<EdgeIx>);
    fn incoming_edges(&self, head: NodeIx, out: &mut Vec<EdgeIx>);
}

pub trait ManifoldView {
    type Point: Copy;
    type Tangent: Copy + Default;

    fn point(&self, n: NodeIx) -> Self::Point;
    fn dist2(&self, a: Self::Point, b: Self::Point) -> f32;
    fn dist_nodes(&self, a: NodeIx, b: NodeIx) -> f32 {
        let pa = self.point(a);
        let pb = self.point(b);
        // For Euclidean, sqrt(dist2)
        self.dist2(pa, pb).sqrt()
    }
    fn make_tangent(dx: f32, dy: f32, dz: f32) -> Self::Tangent;
    fn epoch(&self) -> crate::Epoch;
}

pub trait ManifoldMut: ManifoldView {
    fn exp_map_in_place(&mut self, node: NodeIx, tangent: Self::Tangent);
    fn relax_constraint(&mut self, node: NodeIx, constraint_id: u32, strength: f32);
    fn bump_epoch(&mut self);
}