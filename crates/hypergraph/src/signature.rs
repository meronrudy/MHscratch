use core::ids::ManifoldId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EdgeSignature {
    pub tail_manifolds: Vec<ManifoldId>,
    pub head_manifold: ManifoldId,
}
