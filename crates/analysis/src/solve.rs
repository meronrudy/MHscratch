use traits::LinearOperator;
use ndarray::{Array1, ArrayView1};

/// Solves the linear system `Ax = b` using the Conjugate Gradient method.
///
/// The method is an iterative algorithm, so a max number of iterations is required.
pub fn conjugate_gradient<L>(
    op: &L,
    b: &Array1<f64>,
    x: &mut Array1<f64>,
    max_iters: usize,
    tolerance: f64,
) -> Result<usize, &'static str>
where
    L: LinearOperator,
{
    // Helper function to compute dot product without using BLAS
    fn dot_product(a: &Array1<f64>, b: &Array1<f64>) -> f64 {
        a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum()
    }
    
    let mut r = b.clone();
    let mut temp = Array1::zeros(op.rows());
    op.mul_into(x.view(), &mut temp);
    r = b - &temp;

    let mut p = r.clone();
    let mut rs_old = dot_product(&r, &r);

    if rs_old.sqrt() < tolerance {
        return Ok(0);
    }

    for i in 0..max_iters {
        let mut ap = Array1::zeros(op.rows());
        op.mul_into(p.view(), &mut ap);

        let alpha = rs_old / dot_product(&p, &ap);
        *x += &(p.mapv(|v| v * alpha));
        r -= &(ap.mapv(|v| v * alpha));

        let rs_new = dot_product(&r, &r);
        if rs_new.sqrt() < tolerance {
            return Ok(i + 1);
        }

        p = &r + &(p.mapv(|v| v * (rs_new / rs_old)));
        rs_old = rs_new;
    }

    Err("Conjugate Gradient did not converge")
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra_sparse::{CsrMatrix, CooMatrix};
    use nalgebra::DVector;
    use ndarray::{arr1, Array, ArrayView1};
    use approx::assert_relative_eq;

    /// A simple linear operator for a sparse matrix.
    struct SparseLinearOperator {
        matrix: CsrMatrix<f64>,
    }

    impl LinearOperator for SparseLinearOperator {
        fn apply(&self, v: ArrayView1<f64>) -> Array1<f64> {
            let mut y = Array1::zeros(self.rows());
            self.mul_into(v, &mut y);
            y
        }

        fn rows(&self) -> usize {
            self.matrix.nrows()
        }

        fn cols(&self) -> usize {
            self.matrix.ncols()
        }

                            fn mul_into(&self, x: ArrayView1<f64>, y: &mut Array1<f64>) {
            let x_vec = DVector::from_row_slice(x.as_slice().unwrap());
            
            // Create a new DVector to store the result
            let mut y_vec = DVector::zeros(self.rows());
            
            // Get the raw data from the matrix
            let (row_offsets, col_indices, values) = self.matrix.csr_data();
            
            // Perform the multiplication manually
            for row_idx in 0..self.rows() {
                let start = row_offsets[row_idx] as usize;
                let end = row_offsets[row_idx + 1] as usize;
                
                let mut sum = 0.0;
                for i in start..end {
                    let col_idx = col_indices[i] as usize;
                    let val = values[i];
                    sum += val * x_vec[col_idx];
                }
                y_vec[row_idx] = sum;
            }
            
            y.assign(&Array1::from_vec(y_vec.as_slice().to_vec()));
        }

        fn mul_transpose_into(&self, x: ArrayView1<f64>, y: &mut Array1<f64>) {
            let x_vec = DVector::from_row_slice(x.as_slice().unwrap());
            
            // Create a new DVector to store the result
            let mut y_vec = DVector::zeros(self.cols());
            
            // Get the raw data from the matrix
            let (row_offsets, col_indices, values) = self.matrix.csr_data();
            
            // Perform the transpose multiplication manually
            for row_idx in 0..self.rows() {
                let start = row_offsets[row_idx] as usize;
                let end = row_offsets[row_idx + 1] as usize;
                
                for i in start..end {
                    let col_idx = col_indices[i] as usize;
                    let val = values[i];
                    y_vec[col_idx] += val * x_vec[row_idx];
                }
            }
            
            y.assign(&Array1::from_vec(y_vec.as_slice().to_vec()));
        }
    }

    #[test]
    fn test_conjugate_gradient() {
        // 1. Create a known positive-definite matrix `A`.
        let mut coo = CooMatrix::new(3, 3);
        coo.push(0, 0, 4.0);
        coo.push(0, 1, 1.0);
        coo.push(1, 0, 1.0);
        coo.push(1, 1, 3.0);
        coo.push(1, 2, 2.0);
        coo.push(2, 1, 2.0);
        coo.push(2, 2, 5.0);
        let a_matrix = CsrMatrix::from(&coo);
        let a_op = SparseLinearOperator { matrix: a_matrix };

        // 2. Create a known vector `b`.
        let b = arr1(&[1.0, 2.0, 3.0]);

        // The exact solution.
        let x_exact = arr1(&[7.0/39.0, 11.0/39.0, 19.0/39.0]);

        // 3. Solves `Ax = b` using `conjugate_gradient`.
        let mut x = Array::zeros(3);
        let result = conjugate_gradient(&a_op, &b, &mut x, 100, 1e-9);

        // Check that the solver converged.
        assert!(result.is_ok());

        // 4. Assert that the solution is close to the expected solution.
        assert_relative_eq!(x.as_slice().unwrap(), x_exact.as_slice().unwrap(), epsilon = 1e-6);
    }
}
