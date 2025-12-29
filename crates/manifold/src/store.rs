use core::ids::{Epoch, NodeIx};
use nalgebra::{Point3, Vector3};

pub type Point = Point3<f32>;
pub type Vecf = Vector3<f32>;

pub trait Integrator {
    fn integrate(&self, points: &mut [Point], velocities: &[Vecf], dt: f32);
}

pub struct ExplicitEuler;

impl Integrator for ExplicitEuler {
    fn integrate(&self, points: &mut [Point], velocities: &[Vecf], dt: f32) {
        for (point, velocity) in points.iter_mut().zip(velocities.iter()) {
            *point += *velocity * dt;
        }
    }
}

pub struct ManifoldStore<I: Integrator> {
    pub epoch: Epoch,
    pub t: f32,
    pub dt: f32,
    pub points: Vec<Point>,
    pub velocities: Option<Vec<Vecf>>,
    integrator: I,
}

impl<I: Integrator> ManifoldStore<I> {
    pub fn new(integrator: I) -> Self {
        Self {
            epoch: Epoch(0),
            t: 0.0,
            dt: 0.0,
            points: Vec::new(),
            velocities: None,
            integrator,
        }
    }

    pub fn integrate(&mut self, dt: f32) {
        self.dt = dt;
        if let Some(velocities) = &self.velocities {
            self.integrator.integrate(&mut self.points, velocities, dt);
            self.t += dt;
            self.epoch.0 += 1;
        }
    }
}
