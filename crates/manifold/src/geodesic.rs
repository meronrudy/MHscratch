use crate::store::{Point, Vecf};

/// Manifold type for geodesic operations
#[derive(Clone, Copy, Debug)]
pub enum ManifoldType {
    Euclidean,
    Spherical,
}

/// Exponential map for different manifold types
pub fn exp(manifold: ManifoldType, base: &Point, tangent: &Vecf) -> Point {
    match manifold {
        ManifoldType::Euclidean => exp_euclidean(base, tangent),
        ManifoldType::Spherical => exp_spherical(base, tangent),
    }
}

/// Logarithmic map for different manifold types
pub fn log(manifold: ManifoldType, base: &Point, target: &Point) -> Vecf {
    match manifold {
        ManifoldType::Euclidean => log_euclidean(base, target),
        ManifoldType::Spherical => log_spherical(base, target),
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