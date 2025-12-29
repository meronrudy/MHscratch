use ndarray::{Array1, ArrayView1};

pub trait LinearOperator {
    fn rows(&self) -> usize;
    fn cols(&self) -> usize;
    fn apply(&self, v: ArrayView1<f64>) -> Array1<f64>;
    fn mul_into(&self, x: ArrayView1<f64>, y: &mut Array1<f64>);
    fn mul_transpose_into(&self, x: ArrayView1<f64>, y: &mut Array1<f64>);
}
