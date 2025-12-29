use crate::linear_operator::LinearOperator;
use ndarray::{Array1, ArrayView1};

/// A trait for differentiable linear operators.
pub trait DifferentiableOperator: LinearOperator {
    /// Performs the Jacobian-vector product `y = J * x`.
    fn jacobian_mul_into(&self, x: ArrayView1<f64>, y: &mut Array1<f64>);

    /// Performs the transposed Jacobian-vector product `y = J^T * x`.
    fn jacobian_transpose_mul_into(&self, x: ArrayView1<f64>, y: &mut Array1<f64>);
}

#[cfg(feature = "autodiff")]
pub mod autodiff {
    use super::{DifferentiableOperator, LinearOperator};
    use ndarray::Array1;
    use ndarray_linalg::Norm;

    /// Checks the gradients of a differentiable operator at a given point using finite differences.
    pub fn check_gradients<T: DifferentiableOperator>(
        op: &T,
        x: &Array1<f64>,
    ) -> Result<(), String> {
        const EPS: f64 = 1e-6;

        // Compute the analytical gradient.
        let mut analytical_grad = Array1::zeros(op.cols());
        let ones = Array1::ones(op.rows());
        op.jacobian_transpose_mul_into(ones.view(), &mut analytical_grad);

        // Compute the numerical gradient using finite differences.
        let mut numerical_grad = Array1::zeros(op.cols());
        for i in 0..op.cols() {
            let mut x_plus = x.clone();
            x_plus[i] += EPS;
            let mut y_plus = Array1::zeros(op.rows());
            op.mul_into(x_plus.view(), &mut y_plus);

            let mut x_minus = x.clone();
            x_minus[i] -= EPS;
            let mut y_minus = Array1::zeros(op.rows());
            op.mul_into(x_minus.view(), &mut y_minus);

            let grad_i = (y_plus.sum() - y_minus.sum()) / (2.0 * EPS);
            numerical_grad[i] = grad_i;
        }

        // Compare the gradients.
        let diff = &analytical_grad - &numerical_grad;
        let norm = diff.norm_l2();
        if norm > 1e-5 {
            return Err(format!(
                "Gradient check failed! Norm of difference: {}.\nAnalytical: {}\nNumerical: {}",
                norm,
                analytical_grad,
                numerical_grad
            ));
        }

        Ok(())
    }
}
