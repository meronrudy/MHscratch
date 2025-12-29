use core::ids::{NodeIx, EdgeIx};
use manifold::store::ManifoldStore;

/// Specifies whether to fire an edge in push or pull mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireMode {
    /// Triggers edges from dirty tails.
    Push,
    /// Triggers edges from a requested head.
    Pull,
}

/// A tangent vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tangent {
    pub dx: f32,
    pub dy: f32,
}

/// A change to the hypergraph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Delta {
    /// A point update to a node.
    PointUpdate(NodeIx, Tangent),
    // A constraint to be applied to a node.
    Constraint(NodeIx, u32, f32),
}

// Operator: move head halfway toward midpoint of its two tails.
pub fn eval_edge(
    _edge: EdgeIx,
    tails: &[NodeIx],
    head: NodeIx,
    m: &ManifoldStore,
) -> Delta {
    let a = tails[0];
    let b = tails[1];

    let mid = m.midpoint(a, b);
    let ph_x = m.point_x[head as usize];
    let ph_y = m.point_y[head as usize];

    let dx = 0.5 * (mid.x - ph_x);
    let dy = 0.5 * (mid.y - ph_y);
    Delta::PointUpdate(head, Tangent { dx, dy })
}
