use core::ids::Epoch;
use nalgebra::{Point3, Vector3};

pub type Point = Point3<f32>;
pub type Vecf = Vector3<f32>;

#[cfg(feature = "perf")]
use crate::simd::SoAPoints;

pub trait Integrator: Clone {
    #[cfg(feature = "perf")]
    fn integrate(&self, points: &mut SoAPoints, velocities: &[Vecf], dt: f32);
    #[cfg(not(feature = "perf"))]
    fn integrate(&self, points: &mut [Point], velocities: &[Vecf], dt: f32);
}

#[derive(Clone)]
pub struct ExplicitEuler;

impl Integrator for ExplicitEuler {
    #[cfg(feature = "perf")]
    fn integrate(&self, points: &mut SoAPoints, velocities: &[Vecf], dt: f32) {
        for i in 0..points.len() {
            let velocity = &velocities[i];
            points.x[i] += velocity.x * dt;
            points.y[i] += velocity.y * dt;
            points.z[i] += velocity.z * dt;
        }
    }

    #[cfg(not(feature = "perf"))]
    fn integrate(&self, points: &mut [Point], velocities: &[Vecf], dt: f32) {
        for (point, velocity) in points.iter_mut().zip(velocities.iter()) {
            *point += *velocity * dt;
        }
    }
}

#[derive(Clone, Debug)]
pub struct ManifoldStore<I: Integrator> {
    pub epoch: Epoch,
    pub t: f32,
    pub dt: f32,
    #[cfg(feature = "perf")]
    pub points: SoAPoints,
    #[cfg(not(feature = "perf"))]
    pub points: Vec<Point>,
    pub velocities: Option<Vec<Vecf>>,
    integrator: I,
}

impl<I: Integrator> ManifoldStore<I> {
    pub fn new(integrator: I) -> Self {
        Self {
            epoch: 0,
            t: 0.0,
            dt: 0.0,
            #[cfg(feature = "perf")]
            points: SoAPoints::new(),
            #[cfg(not(feature = "perf"))]
            points: Vec::new(),
            velocities: None,
            integrator,
        }
    }

    pub fn integrate(&mut self, dt: f32) {
        self.dt = dt;
        if let Some(velocities) = &self.velocities {
            #[cfg(feature = "perf")]
            self.integrator.integrate(&mut self.points, velocities, dt);
            #[cfg(not(feature = "perf"))]
            self.integrator.integrate(&mut self.points, velocities, dt);
            self.t += dt;
            self.epoch += 1;
        }
    }
}
