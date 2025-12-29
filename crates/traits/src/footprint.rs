use core::ids::NodeIx;

#[derive(Clone, Debug)]
pub struct Footprint {
    pub influence_radius: f32,
    pub anchor: Option<NodeIx>,
}
