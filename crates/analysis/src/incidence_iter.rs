use hypergraph::{EdgeIndex, NodeIndex, frozen::HypergraphFrozen};

pub struct IncidenceIter<'a> {
    hypergraph: &'a HypergraphFrozen,
    edge_iter: std::ops::Range<u32>,
    inner_iter: Option<(EdgeIndex, std::iter::Chain<std::slice::Iter<'a, NodeIndex>, std::slice::Iter<'a, NodeIndex>>) >,
}

impl<'a> Iterator for IncidenceIter<'a> {
    type Item = (NodeIndex, EdgeIndex, i8);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((edge_index, inner_iter)) = &mut self.inner_iter {
                if let Some(node) = inner_iter.next() {
                    let sign = if self.hypergraph.edge_head[*edge_index as usize] == *node { 1 } else { -1 };
                    return Some((*node, *edge_index, sign));
                }
            }

            if let Some(edge_index) = self.edge_iter.next().map(|i| EdgeIndex::from(i))
            {
                let inputs = self.hypergraph.edge_tails(edge_index).iter();
                let outputs = self.hypergraph.edge_head[edge_index as usize..edge_index as usize + 1].iter();
                self.inner_iter = Some((edge_index, inputs.chain(outputs)));
            } else {
                return None;
            }
        }
    }
}

pub fn iter_incidences<'a>(hypergraph: &'a HypergraphFrozen) -> IncidenceIter<'a> {
    IncidenceIter {
        hypergraph,
        edge_iter: 0..hypergraph.n_edges,
        inner_iter: None,
    }
}
