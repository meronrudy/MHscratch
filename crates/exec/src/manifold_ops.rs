use crate::views::ManifoldView;
use core::ids::NodeIx;

pub trait ManifoldMut: ManifoldView {
    fn exp_map_in_place(&mut self, node: NodeIx, tangent: Self::Tangent);
    fn relax_constraint(&mut self, node: NodeIx, constraint_id: u32, strength: f32);
    fn bump_epoch(&mut self);
}
