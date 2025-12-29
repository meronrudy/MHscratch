use crate::linear_operator::LinearOperator;
use ndarray::{Array1, ArrayView1};

/// A coupled Laplacian operator that combines two linear operators.
///
/// The coupled Laplacian is defined as `L = alpha * L1 + beta * L2`,
/// where `L1` and `L2` are linear operators and `alpha` and `beta` are scalar weights.
pub struct CoupledLaplacian<L1, L2>
where
    L1: LinearOperator,
    L2: LinearOperator,
{
    l1: L1,
    l2: L2,
    alpha: f64,
    beta: f64,
}

impl<L1, L2> CoupledLaplacian<L1, L2>
where
    L1: LinearOperator,
    L2: LinearOperator,
{
    /// Creates a new coupled Laplacian operator.
    pub fn new(l1: L1, l2: L2, alpha: f64, beta: f64) -> Self {
        assert_eq!(
            l1.rows(),
            l2.rows(),
            "Operators must have the same number of rows"
        );
        assert_eq!(
            l1.cols(),
            l2.cols(),
            "Operators must have the same number of columns"
        );
        Self { l1, l2, alpha, beta }
    }
}

impl<L1, L2> LinearOperator for CoupledLaplacian<L1, L2>
where
    L1: LinearOperator + Sync,
    L2: LinearOperator + Sync,
{
    fn apply(&self, v: ArrayView1<f64>) -> Array1<f64> {
        let mut y = Array1::zeros(self.rows());
        self.mul_into(v, &mut y);
        y
    }

    fn rows(&self) -> usize {
        self.l1.rows()
    }

    fn cols(&self) -> usize {
        self.l1.cols()
    }

    fn mul_into(&self, x: ArrayView1<f64>, y: &mut Array1<f64>) {
        let mut temp = Array1::zeros(self.rows());
        self.l1.mul_into(x, &mut temp);
        y.assign(&(&temp * self.alpha));
        self.l2.mul_into(x, &mut temp);
        y.zip_mut_with(&temp, |yi, &ti| *yi += self.beta * ti);
    }

    fn mul_transpose_into(&self, x: ArrayView1<f64>, y: &mut Array1<f64>) {
        let mut temp = Array1::zeros(self.cols());
        self.l1.mul_transpose_into(x, &mut temp);
        y.assign(&(&temp * self.alpha));
        self.l2.mul_transpose_into(x, &mut temp);
        y.zip_mut_with(&temp, |yi, &ti| *yi += self.beta * ti);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::laplacian::LaplacianBuilder;
    use hypergraph::dynamic::HypergraphDyn as DynamicHypergraph;
    use nalgebra::DVector;
    use ndarray::{arr1, Array1};
    use approx::assert_relative_eq;
    use hypergraph::NodeIndex;
    use hypergraph::freeze::freeze_checked;
    use nalgebra_sparse::{CsrMatrix, CooMatrix};

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
    fn test_coupled_laplacian_multiplication() {
        // 1. Create a small hypergraph.
        let mut hg = DynamicHypergraph::new(0);
        let n0 = 0 as NodeIndex;
        let n1 = 1 as NodeIndex;
        let n2 = 2 as NodeIndex;
        hg.add_edge(&[n0], n1, manifold::footprint::EdgeFootprint::V1(manifold::footprint::EdgeFootprintV1 { influence_radius: 0.0, anchor: None }));
        hg.add_edge(&[n1], n2, manifold::footprint::EdgeFootprint::V1(manifold::footprint::EdgeFootprintV1 { influence_radius: 0.0, anchor: None }));
        let (frozen_hg, _) = freeze_checked(hg, Default::default());

        // 2. Build the hypergraph Laplacian matrix.
        let l_hyper_matrix = LaplacianBuilder::build(&frozen_hg);
        let l_hyper = SparseLinearOperator { matrix: l_hyper_matrix.clone() };

        // 3. Create a dummy manifold Laplacian.
        let mut coo = CooMatrix::new(3, 3);
        coo.push(0, 0, 1.0);
        coo.push(1, 1, 1.0);
        coo.push(2, 2, 1.0);
        let l_manifold_matrix = CsrMatrix::from(&coo);
        let l_manifold = SparseLinearOperator { matrix: l_manifold_matrix.clone() };

        // 4. Create a CoupledLaplacian operator.
        let alpha = 0.5;
        let beta = 0.5;
        let coupled_op = CoupledLaplacian::new(l_hyper, l_manifold, alpha, beta);

        // 5. Compute `y = L_total * x` using the operator.
        let x = arr1(&[1.0, 2.0, 3.0]);
        let mut y = Array1::zeros(3);
        coupled_op.mul_into(x.view(), &mut y);

        // 6. Compute the explicit `L_total` matrix and `y_expected = L_total_matrix * x`.
        // Create a combined matrix
        let l_total_matrix = &l_hyper_matrix * alpha + &l_manifold_matrix * beta;
        let x_vec = DVector::from_row_slice(x.as_slice().unwrap());
        
        // Create a new DVector to store the result
        let mut y_expected_vec = DVector::zeros(3);
        
        // Get the raw data from the matrix
        let (row_offsets, col_indices, values) = l_total_matrix.csr_data();
        
        // Perform the multiplication manually
        for row_idx in 0..l_total_matrix.nrows() {
            let start = row_offsets[row_idx] as usize;
            let end = row_offsets[row_idx + 1] as usize;
            
            let mut sum = 0.0;
            for i in start..end {
                let col_idx = col_indices[i] as usize;
                let val = values[i];
                sum += val * x_vec[col_idx];
            }
            y_expected_vec[row_idx] = sum;
        }
        
        let y_expected = Array1::from_vec(y_expected_vec.as_slice().to_vec());

        // 7. Assert that `y` and `y_expected` are close.
        assert_relative_eq!(y.as_slice().unwrap(), y_expected.as_slice().unwrap(), epsilon = 1e-9);
    }
}
