use crate::store::ManifoldStore;

// A simple Euler integrator.
pub fn integrate(store: &mut ManifoldStore) {
    let dt = store.dt;
    for i in 0..store.point_x.len() {
        store.point_x[i] += store.velocity[i].dx * dt;
        store.point_y[i] += store.velocity[i].dy * dt;
    }
}
