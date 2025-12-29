use nalgebra::{DMatrix, DVector};

// Trait for computing backends
pub trait Backend {
    type Matrix;
    type Vector;

    fn mat_vec_mul(&self, matrix: &Self::Matrix, vector: &Self::Vector) -> Self::Vector;
    fn distance(&self, a: &Self::Vector, b: &Self::Vector) -> f64;
}

// CPU backend using nalgebra
pub struct CpuBackend;

impl Backend for CpuBackend {
    type Matrix = DMatrix<f64>;
    type Vector = DVector<f64>;

    fn mat_vec_mul(&self, matrix: &Self::Matrix, vector: &Self::Vector) -> Self::Vector {
        matrix * vector
    }

    fn distance(&self, a: &Self::Vector, b: &Self::Vector) -> f64 {
        (a - b).norm()
    }
}

// GPU backend placeholder
pub struct GpuBackend;

impl Backend for GpuBackend {
    type Matrix = DMatrix<f64>;
    type Vector = DVector<f64>;

    fn mat_vec_mul(&self, matrix: &Self::Matrix, vector: &Self::Vector) -> Self::Vector {
        // Placeholder: delegate to CPU for now
        CpuBackend.mat_vec_mul(matrix, vector)
    }

    fn distance(&self, a: &Self::Vector, b: &Self::Vector) -> f64 {
        // Placeholder: delegate to CPU for now
        CpuBackend.distance(a, b)
    }
}