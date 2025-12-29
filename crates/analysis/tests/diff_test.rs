use analysis::diff::{DifferentiableOperator, autodiff::check_gradients};
use traits::LinearOperator;
use ndarray::{Array1, ArrayView1};

struct ToyOperator;

impl LinearOperator for ToyOperator {
    fn apply(&self, x: ArrayView1<f64>) -> Array1<f64> {
        let mut y = Array1::zeros(self.rows());
        self.mul_into(x, &mut y);
        y
    }

    fn rows(&self) -> usize {
        1
    }

    fn cols(&self) -> usize {
        2
    }

    fn mul_into(&self, x: ArrayView1<f64>, y: &mut Array1<f64>) {
        y[0] = 2.0 * x[0] + 3.0 * x[1];
    }

    fn mul_transpose_into(&self, x: ArrayView1<f64>, y: &mut Array1<f64>) {
        y[0] = 2.0 * x[0];
        y[1] = 3.0 * x[0];
    }
}

impl DifferentiableOperator for ToyOperator {
    fn jacobian_mul_into(&self, x: ArrayView1<f64>, y: &mut Array1<f64>) {
        self.mul_into(x, y);
    }

    fn jacobian_transpose_mul_into(&self, x: ArrayView1<f64>, y: &mut Array1<f64>) {
        self.mul_transpose_into(x, y);
    }
}

#[test]
fn test_gradient_check() {
    let op = ToyOperator;
    let x = Array1::from_vec(vec![0.5, 0.8]);
    check_gradients(&op, &x).unwrap();
}
