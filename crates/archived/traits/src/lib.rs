use hypergraph::prelude::*;

pub trait Operator<V: Vertex> {
    fn eval(&self, vertices: &[V]) -> V;
}
