use exec::apply::apply;
use exec::fire::{Delta, Tangent};
use manifold::store::{ManifoldStore, Tangent2};
use core::ids::NodeIx;

#[test]
fn test_apply_point_update() {
    let mut manifold = ManifoldStore {
        epoch: 0,
        t: 0.0,
        dt: 0.0,
        node_manifold: vec![],
        point_x: vec![0.0],
        point_y: vec![0.0],
        velocity: vec![Tangent2 { dx: 0.0, dy: 0.0 }],
        manifold_params: vec![],
    };

    let delta = Delta::PointUpdate(0, Tangent { dx: 1.0, dy: 1.0 });

    apply(&mut manifold, delta);

    assert_eq!(manifold.point_x[0], 1.0);
    assert_eq!(manifold.point_y[0], 1.0);
    assert_eq!(manifold.epoch, 1);
}
