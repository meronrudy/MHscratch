use crate::integrate::integrate;
use core::ids::{ManifoldId, NodeIx};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tangent2 {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Clone, Debug)]
pub struct ManifoldParams {
    pub metric_a: f32,
    pub metric_b: f32,
    pub curvature: f32,
}

#[derive(Clone, Debug)]
pub struct ManifoldStore {
    pub epoch: u64,
    pub t: f32,
    pub dt: f32,
    pub node_manifold: Vec<ManifoldId>,
    pub point_x: Vec<f32>,
    pub point_y: Vec<f32>,
    pub velocity: Vec<Tangent2>,
    pub manifold_params: Vec<ManifoldParams>,
}

impl ManifoldStore {
    pub fn dist_nodes(&self, a: NodeIx, b: NodeIx) -> f32 {
        let pa_x = self.point_x[a as usize];
        let pa_y = self.point_y[a as usize];
        let pb_x = self.point_x[b as usize];
        let pb_y = self.point_y[b as usize];
        let dx = pa_x - pb_x;
        let dy = pa_y - pb_y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn midpoint(&self, a: NodeIx, b: NodeIx) -> Point2 {
        let pa_x = self.point_x[a as usize];
        let pa_y = self.point_y[a as usize];
        let pb_x = self.point_x[b as usize];
        let pb_y = self.point_y[b as usize];
        Point2 {
            x: 0.5 * (pa_x + pb_x),
            y: 0.5 * (pa_y + pb_y),
        }
    }

    pub fn apply_delta(&mut self, n: NodeIx, dx: f32, dy: f32) {
        self.point_x[n as usize] += dx;
        self.point_y[n as usize] += dy;
        self.epoch = self.epoch.wrapping_add(1);
    }

    pub fn integrate(&mut self, dt: f32) {
        self.t += dt;
        self.dt = dt;
        integrate(self);
    }
}
