use exec::fire::{eval_edge, Delta, Tangent};
use core::ids::{EdgeIx, NodeIx};
use manifold::store::{ManifoldStore, Tangent2};

#[test]
fn eval_edge_test() {
    let mut m = ManifoldStore {
        epoch: 0,
        t: 0.0,
        dt: 0.0,
        node_manifold: vec![0, 0, 0],
        point_x: vec![0.0, 2.0, 1.5],
        point_y: vec![0.0, 2.0, 0.5],
        velocity: vec![
            Tangent2 { dx: 0.0, dy: 0.0 },
            Tangent2 { dx: 0.0, dy: 0.0 },
            Tangent2 { dx: 0.0, dy: 0.0 },
        ],
        manifold_params: vec![],
    };

    let tails = &[0, 1];
    let head = 2;
    let edge = 0;

    let delta = eval_edge(edge, tails, head, &m);

    let expected_dx = 0.5 * ((0.0 + 2.0) * 0.5 - 1.5);
    let expected_dy = 0.5 * ((0.0 + 2.0) * 0.5 - 0.5);

    match delta {
        Delta::PointUpdate(node, tangent) => {
            assert_eq!(node, head);
            assert_eq!(tangent, Tangent { dx: expected_dx, dy: expected_dy });
        }
        _ => panic!("Expected PointUpdate delta"),
    }
}
