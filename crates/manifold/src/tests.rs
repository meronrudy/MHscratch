#![cfg(test)]

#[cfg(feature = "perf")]
use crate::simd::{Point, Vecf, dist_simd, SoAPoints};

#[cfg(not(feature = "perf"))]
use crate::store::{Point, Vecf};

use nalgebra::distance;

mod geodesic_tests {
    use super::*;
    use crate::geodesic::{exp, log, ManifoldType};

    #[test]
    fn test_euclidean_exp_log_roundtrip() {
        let base = Point::new(1.0, 2.0, 3.0);
        let tangent = Vecf::new(0.1, 0.2, 0.3);

        let result = exp(ManifoldType::Euclidean, &base, &tangent);
        let recovered_tangent = log(ManifoldType::Euclidean, &base, &result);

        // Should be very close for Euclidean (within floating point precision)
        let diff = tangent - recovered_tangent;
        assert!(diff.x.abs() < 1e-6 && diff.y.abs() < 1e-6 && diff.z.abs() < 1e-6);
    }

    #[test]
    fn test_spherical_exp_log_roundtrip_small_angles() {
        let base = Point::new(1.0, 0.0, 0.0); // Unit vector
        let tangent = Vecf::new(0.0, 0.1, 0.0); // Small tangent

        let result = exp(ManifoldType::Spherical, &base, &tangent);
        let recovered_tangent = log(ManifoldType::Spherical, &base, &result);

        // Should be close for small angles - check individual components
        let diff = tangent - recovered_tangent;
        assert!(diff.x.abs() < 1e-6 && diff.y.abs() < 1e-6 && diff.z.abs() < 1e-6);
    }

    #[test]
    fn test_spherical_properties() {
        let base = Point::new(1.0, 0.0, 0.0);
        let tangent = Vecf::new(0.0, 0.0, 0.0); // Zero tangent

        let result = exp(ManifoldType::Spherical, &base, &tangent);
        // Should return base point unchanged
        let diff = result - base;
        assert!(diff.x.abs() < 1e-10 && diff.y.abs() < 1e-10 && diff.z.abs() < 1e-10);
    }
}

#[cfg(feature = "perf")]
mod perf_tests {
    use super::*;
    use crate::simd::{SoAPoints, dist_simd};

    #[test]
    fn test_simd_distance_accuracy() {
        let p1 = Point::new(1.0, 2.0, 3.0);
        let p2 = Point::new(4.0, 5.0, 6.0);

        let scalar_dist = distance(&p1, &p2);
        let simd_dist = dist_simd(&p1, &p2);

        // Should be very close (within floating point precision)
        assert!((scalar_dist - simd_dist).abs() < 1e-6);
    }

    #[test]
    fn test_soa_conversion_roundtrip() {
        let aos_points = vec![
            Point::new(1.0, 2.0, 3.0),
            Point::new(4.0, 5.0, 6.0),
            Point::new(7.0, 8.0, 9.0),
        ];

        let soa = SoAPoints::from_aos(&aos_points);
        let recovered_aos = soa.to_aos();

        assert_eq!(aos_points.len(), recovered_aos.len());
        for (original, recovered) in aos_points.iter().zip(recovered_aos.iter()) {
            assert!((original - recovered).magnitude() < 1e-10);
        }
    }

    #[test]
    fn test_soa_access() {
        let mut soa = SoAPoints::new();
        let point = Point::new(1.0, 2.0, 3.0);
        soa.push(point);

        let retrieved = soa.get(0);
        assert!((point - retrieved).magnitude() < 1e-10);

        let new_point = Point::new(4.0, 5.0, 6.0);
        soa.set(0, new_point);
        let retrieved_new = soa.get(0);
        assert!((new_point - retrieved_new).magnitude() < 1e-10);
    }
}

#[cfg(not(feature = "perf"))]
mod standard_tests {
    use super::*;

    #[test]
    fn test_scalar_distance_consistency() {
        let p1 = Point::new(1.0, 2.0, 3.0);
        let p2 = Point::new(4.0, 5.0, 6.0);

        let dist1 = distance(&p1, &p2);
        let dist2 = distance(&p2, &p1);

        assert_eq!(dist1, dist2);
        assert!(dist1 >= 0.0);
    }
}

mod backend_tests {
    use crate::backend::{Backend, CpuBackend, GpuBackend};
    use nalgebra::{DVector, DMatrix};
    use approx::assert_relative_eq;

    #[test]
    fn test_backend_mat_vec_mul_consistency() {
        let cpu = CpuBackend;
        let gpu = GpuBackend;

        // Create a simple matrix
        let matrix = DMatrix::from_row_slice(3, 3, &[
            1.0, 2.0, 0.0,
            0.0, 3.0, 0.0,
            0.0, 0.0, 4.0,
        ]);

        let vector = DVector::from_vec(vec![1.0, 2.0, 3.0]);

        let cpu_result = cpu.mat_vec_mul(&matrix, &vector);
        let gpu_result = gpu.mat_vec_mul(&matrix, &vector);

        // Compare results with approximate equality
        assert_relative_eq!(cpu_result[0], gpu_result[0], epsilon = 1e-10);
        assert_relative_eq!(cpu_result[1], gpu_result[1], epsilon = 1e-10);
        assert_relative_eq!(cpu_result[2], gpu_result[2], epsilon = 1e-10);
    }

    #[test]
    fn test_backend_distance_consistency() {
        let cpu = CpuBackend;
        let gpu = GpuBackend;

        let a = DVector::from_vec(vec![1.0, 2.0, 3.0]);
        let b = DVector::from_vec(vec![4.0, 5.0, 6.0]);

        let cpu_dist = cpu.distance(&a, &b);
        let gpu_dist = gpu.distance(&a, &b);

        // Compare distances
        assert_relative_eq!(cpu_dist, gpu_dist, epsilon = 1e-10);
    }
}