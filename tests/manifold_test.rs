#[cfg(test)]
mod tests {
    use manifold::store::{ManifoldStore, Tangent2};

    #[test]
    fn test_integrate() {
        let mut store = ManifoldStore {
            epoch: 0,
            t: 0.0,
            dt: 0.0,
            node_manifold: vec![],
            point_x: vec![0.0],
            point_y: vec![0.0],
            velocity: vec![Tangent2 { dx: 1.0, dy: 1.0 }],
            manifold_params: vec![],
        };

        store.integrate(0.1);

        assert_eq!(store.point_x[0], 0.1);
        assert_eq!(store.point_y[0], 0.1);
    }
}
