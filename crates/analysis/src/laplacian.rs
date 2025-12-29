
use hypergraph::{
    FrozenHypergraph,
    FrozenHypergraphView,
};
use nalgebra_sparse::{CsrMatrix, CooMatrix};

use crate::incidence::IncidenceView;

/// Builds the normalized Laplacian of a hypergraph.
pub struct LaplacianBuilder;

impl LaplacianBuilder {
    /// Builds the normalized Laplacian of a hypergraph.
    ///
    /// The normalized Laplacian is defined as:
    /// `L = I - D_v^(-1/2) * B * D_e^(-1) * B^T * D_v^(-1/2)`
    /// where:
    /// - `B` is the incidence matrix of the hypergraph.
    /// - `D_v` is the diagonal matrix of node degrees.
    /// - `D_e` is the diagonal matrix of edge degrees.
    ///
    /// The method returns a sparse CSR matrix representing the Laplacian.
    pub fn build(hypergraph: &FrozenHypergraph) -> CsrMatrix<f64> {
        let num_nodes = hypergraph.n_nodes as usize;
        let num_edges = hypergraph.n_edges as usize;

        if num_nodes == 0 || num_edges == 0 {
            let coo = CooMatrix::try_from_triplets(num_nodes, num_nodes, vec![], vec![], vec![]).unwrap();
            return CsrMatrix::from(&coo);
        }

        let incidence_view = IncidenceView::new_view(hypergraph);

        let mut node_degrees = vec![0.0; num_nodes];
        let mut edge_degrees = vec![0.0; num_edges];

        let mut coo = CooMatrix::new(num_nodes, num_edges);

        for (node, edge, sign) in incidence_view.iter_view() {
            let node_id = node as usize;
            let edge_id = edge as usize;
            let weight = f64::from(sign);

            coo.push(node_id, edge_id, weight);

            node_degrees[node_id] += 1.0;
            edge_degrees[edge_id] += 1.0;
        }

        let b = CsrMatrix::from(&coo);

        let dv_inv_sqrt = Self::diagonal_matrix(num_nodes, &node_degrees, |d| d.powf(-0.5));
        let de_inv = Self::diagonal_matrix(num_edges, &edge_degrees, |d| d.recip());

        let i = CsrMatrix::identity(num_nodes);

        // L = I - D_v^(-1/2) * B * D_e^(-1) * B^T * D_v^(-1/2)
        let temp = &b * &de_inv * &b.transpose();
        let laplacian = &i - &dv_inv_sqrt * &temp * &dv_inv_sqrt;

        laplacian
    }

    /// Creates a sparse diagonal matrix from a vector of values.
    fn diagonal_matrix<F>(size: usize, values: &[f64], f: F) -> CsrMatrix<f64>
    where
        F: Fn(f64) -> f64,
    {
        let mut coo = CooMatrix::new(size, size);
        for (i, &val) in values.iter().enumerate() {
            if val != 0.0 {
                coo.push(i, i, f(val));
            }
        }
        CsrMatrix::from(&coo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hypergraph::dynamic::HypergraphDyn as DynamicHypergraph;
    use nalgebra::DMatrix;
    use hypergraph::NodeIndex;
    use hypergraph::freeze::freeze_checked;

    #[test]
    fn test_laplacian_positive_semi_definite() {
        let mut hg = DynamicHypergraph::new(0);
        let n0 = 0 as NodeIndex;
        let n1 = 1 as NodeIndex;
        let n2 = 2 as NodeIndex;

        hg.add_edge(&[n0], n1);
        hg.add_edge(&[n1], n2);

        let (frozen_hg, _) = freeze_checked(hg, Default::default());
        let laplacian_sparse = LaplacianBuilder::build(&frozen_hg);
        let mut laplacian_dense = DMatrix::zeros(laplacian_sparse.nrows(), laplacian_sparse.ncols());
        for (r, c, v) in laplacian_sparse.triplet_iter() {
            laplacian_dense[(r, c)] = *v;
        }

        let eigenvalues = laplacian_dense.symmetric_eigen().eigenvalues;

        // Check that all eigenvalues are non-negative (within a small tolerance).
        for eigenvalue in eigenvalues.iter() {
            assert!(*eigenvalue >= -1e-9);
        }

        // The smallest eigenvalue should be close to zero.
        let min_eigenvalue = eigenvalues.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        assert!(min_eigenvalue.abs() < 1e-9);
    }
}
