use core::ids::{EdgeIx, NodeIx};

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
}
