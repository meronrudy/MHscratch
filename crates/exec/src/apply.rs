use crate::fire::Delta;
use manifold::store::ManifoldStore;

pub fn apply(manifold: &mut ManifoldStore, delta: Delta) {
    match delta {
        Delta::PointUpdate(node_ix, tangent) => {
            manifold.apply_delta(node_ix, tangent.dx, tangent.dy);
        }
        Delta::Constraint(_, _, _) => {
            // TODO: Implement constraint application.
        }
    }
}