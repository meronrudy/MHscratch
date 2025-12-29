use crate::delta::Delta;
use crate::manifold_ops::ManifoldMut;
use core::ids::{EdgeIx, NodeIx};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeltaKind {
    Point = 0,
    Constraint = 1,
}

fn kind_of<T>(d: &Delta<T>) -> DeltaKind {
    match d {
        Delta::PointUpdate { .. } => DeltaKind::Point,
        Delta::Constraint { .. } => DeltaKind::Constraint,
    }
}

fn node_of<T>(d: &Delta<T>) -> NodeIx {
    match d {
        Delta::PointUpdate(node, _) => *node,
        Delta::Constraint(node, _, _) => *node,
    }
}



pub fn apply_deltas<M: ManifoldMut>(
    mani: &mut M,
    deltas: &mut [Delta<M::Tangent>],
) {
    // Canonical order: (node, kind)
    deltas.sort_by(|a, b| {
        node_of(a)
            .cmp(&node_of(b))
            .then((kind_of(a) as u8).cmp(&(kind_of(b) as u8)))
    });

    let mut mutated = false;

    // Reduce/apply in deterministic order.
    for d in deltas.iter() {
        match *d {
            Delta::PointUpdate(node, tangent) => {
                mani.exp_map_in_place(node, tangent);
                mutated = true;
            }
            Delta::Constraint(node, constraint_id, strength) => {
                mani.relax_constraint(node, constraint_id, strength);
                mutated = true;
            }
        }
    }

    if mutated {
        mani.bump_epoch();
    }
}
