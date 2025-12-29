use core::ids::{EdgeIx, NodeIx};

#[derive(Clone, Copy, Debug)]
pub enum Delta<Tangent> {
    PointUpdate(NodeIx, Tangent),
    Constraint(NodeIx, u32, f32),
}

#[derive(Debug)]
pub struct DeltaBuf<Tangent> {
    pub deltas: Vec<Delta<Tangent>>,
}

impl<T: Copy> DeltaBuf<T> {
    pub fn with_capacity(cap: usize) -> Self {
        Self { deltas: Vec::with_capacity(cap) }
    }
    #[inline] pub fn clear(&mut self) { self.deltas.clear(); }
    #[inline] pub fn push(&mut self, d: Delta<T>) { self.deltas.push(d); }
}
