use core::ids::Epoch;
use nalgebra::{Point3, Vector3};
use core::views::ManifoldView;
use core::views::ManifoldMut;

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

    pub fn new_with_points(points: Vec<Point>, integrator: I) -> Self {
        Self {
            epoch: 0,
            t: 0.0,
            dt: 0.0,
            points,
            velocities: None,
            integrator,
        }
    }

    pub fn dist_nodes(&self, a: u32, b: u32) -> f32 {
        let pa = self.point(a as usize);
        let pb = self.point(b as usize);
        nalgebra::distance(&pa, &pb)
    }

    pub fn point(&self, i: usize) -> Point {
        self.points[i]
    }

    pub fn dirty_point(&mut self, i: u32) {
        // For demo, just mark epoch
        self.epoch += 1;
    }
}

impl<I: Integrator> ManifoldView for ManifoldStore<I> {
    type Point = Point;
    type Tangent = Vecf;

    fn point(&self, n: u32) -> Self::Point {
        self.point(n as usize)
    }

    fn dist2(&self, a: Self::Point, b: Self::Point) -> f32 {
        nalgebra::distance_squared(&a, &b)
    }

    fn make_tangent(dx: f32, dy: f32, dz: f32) -> Self::Tangent {
        Vecf::new(dx, dy, dz)
    }

    fn epoch(&self) -> core::Epoch {
        self.epoch
    }
}

impl<I: Integrator> ManifoldMut for ManifoldStore<I> {
    fn exp_map_in_place(&mut self, node: u32, tangent: Self::Tangent) {
        let i = node as usize;
        self.points[i] += tangent;
    }

    fn relax_constraint(&mut self, _node: u32, _constraint_id: u32, _strength: f32) {
        // For demo, do nothing
    }

    fn bump_epoch(&mut self) {
        self.epoch += 1;
    }
}
