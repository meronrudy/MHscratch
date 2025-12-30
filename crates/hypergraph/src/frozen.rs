use crate::arity::ArityGroup;
use crate::signature::EdgeSignature;
use core::ids::{EdgeIx, NodeIx, Epoch};
use manifold::footprint::EdgeFootprint as Footprint;
use core::views::FrozenGraphView;

#[derive(Clone, Debug)]
pub struct HypergraphFrozen {
    pub epoch: Epoch,
    pub n_nodes: u32,
    pub n_edges: u32,

    // Edge payload (common to all arities)
    pub edge_head: Vec<NodeIx>,
    pub edge_kind: Vec<u16>,
    pub edge_info: Vec<(ArityGroup, u32)>,
    pub signatures: Vec<EdgeSignature>,
    pub footprints: Vec<Footprint>,

    // Arity-specific storage
    pub tails_k2: Vec<NodeIx>,
    pub edges_k2: Vec<EdgeIx>,

    pub tails_k3: Vec<NodeIx>,
    pub edges_k3: Vec<EdgeIx>,

    // Variable-arity edges (packed CSR)
    pub tail_off_var: Vec<u32>,
    pub tails_var: Vec<NodeIx>,
    pub edges_var: Vec<EdgeIx>,

    // Node adjacency CSR (for all edges)
    pub out_off: Vec<u32>,
    pub out_edges: Vec<EdgeIx>,
    pub in_off: Vec<u32>,
    pub in_edges: Vec<EdgeIx>,
}

impl HypergraphFrozen {
    pub fn edge_tails(&self, e: EdgeIx) -> &[NodeIx] {
        let (group, group_index) = &self.edge_info[e as usize];
        match group {
            ArityGroup::K2 => {
                let start = *group_index as usize * 2;
                &self.tails_k2[start..start + 2]
            }
            ArityGroup::K3 => {
                let start = *group_index as usize * 3;
                &self.tails_k3[start..start + 3]
            }
            ArityGroup::Var => {
                let start = self.tail_off_var[*group_index as usize] as usize;
                let end = self.tail_off_var[*group_index as usize + 1] as usize;
                &self.tails_var[start..end]
            }
        }
    }

    pub fn edge_indices(&self) -> impl Iterator<Item = EdgeIx> {
        0..self.n_edges
    }
}

// Incidence View for Laplacian

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IncRole {
    Tail,
    Head,
}

#[derive(Clone, Copy, Debug)]
pub struct IncEntry {
    pub node: NodeIx,
    pub edge: EdgeIx,
    pub role: IncRole,
    pub weight: f32, // use f32 for weights
}

pub trait FrozenIncidence {
    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;

    // zero-copy
    fn edge_tails(&self, e: EdgeIx) -> &[NodeIx];
    fn edge_head(&self, e: EdgeIx) -> NodeIx;
    fn node_incident_edges(&self, v: NodeIx) -> &[EdgeIx];

    // edge weight (assume 1.0 for now)
    fn edge_weight(&self, e: EdgeIx) -> f32;
}

impl FrozenIncidence for HypergraphFrozen {
    fn node_count(&self) -> usize { self.n_nodes as usize }
    fn edge_count(&self) -> usize { self.n_edges as usize }

    fn edge_tails(&self, e: EdgeIx) -> &[NodeIx] { self.edge_tails(e) }
    fn edge_head(&self, e: EdgeIx) -> NodeIx { self.edge_head[e as usize] }
    fn node_incident_edges(&self, v: NodeIx) -> &[EdgeIx] {
        let start = self.out_off[v as usize] as usize;
        let end = self.out_off[v as usize + 1] as usize;
        &self.out_edges[start..end]
    }

    fn edge_weight(&self, _e: EdgeIx) -> f32 { 1.0 } // TODO: add weights if needed
}

impl FrozenGraphView for HypergraphFrozen {
    fn edge_count(&self) -> usize { self.n_edges as usize }

    fn edge_tails(&self, e: EdgeIx) -> &[NodeIx] { self.edge_tails(e) }
    fn edge_head(&self, e: EdgeIx) -> NodeIx { self.edge_head[e as usize] }

    fn incident_edges(&self, node: NodeIx, out: &mut Vec<EdgeIx>) {
        out.clear();
        let start = self.out_off[node as usize] as usize;
        let end = self.out_off[node as usize + 1] as usize;
        out.extend_from_slice(&self.out_edges[start..end]);
    }

    fn incoming_edges(&self, head: NodeIx, out: &mut Vec<EdgeIx>) {
        out.clear();
        let start = self.in_off[head as usize] as usize;
        let end = self.in_off[head as usize + 1] as usize;
        out.extend_from_slice(&self.in_edges[start..end]);
    }
}

// Iterators

pub struct EdgeIncIter<'a, G: FrozenIncidence> {
    graph: &'a G,
    edge: EdgeIx,
    tails: std::slice::Iter<'a, NodeIx>,
    yielded_head: bool,
}

impl<'a, G: FrozenIncidence> EdgeIncIter<'a, G> {
    pub fn new(graph: &'a G, e: EdgeIx) -> Self {
        let tails = graph.edge_tails(e).iter();
        Self { graph, edge: e, tails, yielded_head: false }
    }
}

impl<'a, G: FrozenIncidence> Iterator for EdgeIncIter<'a, G> {
    type Item = IncEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&node) = self.tails.next() {
            return Some(IncEntry {
                node,
                edge: self.edge,
                role: IncRole::Tail,
                weight: self.graph.edge_weight(self.edge),
            });
        }
        if !self.yielded_head {
            self.yielded_head = true;
            return Some(IncEntry {
                node: self.graph.edge_head(self.edge),
                edge: self.edge,
                role: IncRole::Head,
                weight: self.graph.edge_weight(self.edge),
            });
        }
        None
    }
}

pub struct NodeIncIter<'a, G: FrozenIncidence> {
    graph: &'a G,
    node: NodeIx,
    edges: std::slice::Iter<'a, EdgeIx>,
}

impl<'a, G: FrozenIncidence> NodeIncIter<'a, G> {
    pub fn new(graph: &'a G, v: NodeIx) -> Self {
        let edges = graph.node_incident_edges(v).iter();
        Self { graph, node: v, edges }
    }
}

impl<'a, G: FrozenIncidence> Iterator for NodeIncIter<'a, G> {
    type Item = IncEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let &edge = self.edges.next()?;
        let role = if self.graph.edge_head(edge) == self.node {
            IncRole::Head
        } else {
            IncRole::Tail
        };
        Some(IncEntry {
            node: self.node,
            edge,
            role,
            weight: self.graph.edge_weight(edge),
        })
    }
}
