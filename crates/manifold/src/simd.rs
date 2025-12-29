#![cfg(feature = "perf")]

use nalgebra::{Point3, Vector3};

pub type Point = Point3<f32>;
pub type Vecf = Vector3<f32>;

/// Structure of Arrays storage for points in performance mode
#[derive(Clone, Debug)]
pub struct SoAPoints {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub z: Vec<f32>,
}

impl SoAPoints {
    pub fn new() -> Self {
        Self {
            x: Vec::new(),
            y: Vec::new(),
            z: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            x: Vec::with_capacity(capacity),
            y: Vec::with_capacity(capacity),
            z: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.x.len()
    }

    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }

    pub fn push(&mut self, point: Point) {
        self.x.push(point.x);
        self.y.push(point.y);
        self.z.push(point.z);
    }

    pub fn get(&self, index: usize) -> Point {
        Point::new(self.x[index], self.y[index], self.z[index])
    }

    pub fn set(&mut self, index: usize, point: Point) {
        self.x[index] = point.x;
        self.y[index] = point.y;
        self.z[index] = point.z;
    }

    pub fn from_aos(points: &[Point]) -> Self {
        let mut soa = Self::with_capacity(points.len());
        for &point in points {
            soa.push(point);
        }
        soa
    }

    pub fn to_aos(&self) -> Vec<Point> {
        let mut aos = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            aos.push(self.get(i));
        }
        aos
    }
}

/// SIMD-accelerated Euclidean distance computation (placeholder for now)
pub fn dist_simd(p1: &Point, p2: &Point) -> f32 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    let dz = p1.z - p2.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Batch distance computation (placeholder for SIMD implementation)
pub fn batch_dist_simd(soa: &SoAPoints, center: &Point, radii: &[f32], results: &mut Vec<usize>) {
    results.clear();

    // Current implementation processes points individually
    // TODO: Implement SIMD vectorization for better performance
    for i in 0..soa.len() {
        let point = soa.get(i);
        let dist = dist_simd(&point, center);
        if dist <= radii[i] {
            results.push(i);
        }
    }
}

/// Euclidean manifold exponential map (identity for Euclidean)
pub fn exp_euclidean(base: &Point, tangent: &Vecf) -> Point {
    *base + tangent
}

/// Euclidean manifold logarithmic map (identity for Euclidean)
pub fn log_euclidean(base: &Point, target: &Point) -> Vecf {
    target - base
}

/// Spherical manifold exponential map
pub fn exp_spherical(base: &Point, tangent: &Vecf) -> Point {
    // Normalize base point to unit sphere
    let base_norm = base.coords.magnitude();
    let base_unit = base.coords / base_norm;

    let tangent_norm = tangent.magnitude();
    if tangent_norm < 1e-8 {
        return *base;
    }

    // Project tangent onto tangent space of sphere
    let tangent_tangential = *tangent - base_unit * base_unit.dot(tangent);

    let angle = tangent_tangential.magnitude();
    if angle < 1e-8 {
        return *base;
    }

    let axis = tangent_tangential / angle;

    // Rodrigues' rotation formula
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();

    let rotated = base_unit * cos_angle + axis.cross(&base_unit) * sin_angle + axis * (axis.dot(&base_unit)) * (1.0 - cos_angle);

    Point::from(rotated * base_norm)
}

/// Spherical manifold logarithmic map
pub fn log_spherical(base: &Point, target: &Point) -> Vecf {
    // Normalize both points
    let base_norm = base.coords.magnitude();
    let target_norm = target.coords.magnitude();

    let base_unit = base.coords / base_norm;
    let target_unit = target.coords / target_norm;

    // Angle between points
    let cos_angle = base_unit.dot(&target_unit).min(1.0).max(-1.0);
    let angle = cos_angle.acos();

    if angle < 1e-8 {
        return Vecf::zeros();
    }

    // Axis of rotation
    let axis = base_unit.cross(&target_unit);
    let axis_norm = axis.magnitude();

    if axis_norm < 1e-8 {
        // Points are nearly antipodal or coincident, handle edge case
        return Vecf::zeros();
    }

    let axis_unit = axis / axis_norm;

    // Tangent vector in the direction of the geodesic
    axis_unit * angle * base_norm
}