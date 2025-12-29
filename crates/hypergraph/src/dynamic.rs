use crate::signature::EdgeSignature;
use bit_vec::BitVec;
use core::ids::{EdgeIx, NodeIx, Epoch};
use manifold::store::{ManifoldStore, Integrator};
use manifold::footprint::EdgeFootprint;

pub struct HypergraphDyn {
    pub graph_epoch: Epoch,
    pub dirty_nodes: BitVec,
    pub dirty_edges: BitVec,

    // Edge payload (common to all arities)
    pub head: Vec<NodeIx>,
    pub edge_kind: Vec<u16>,
    pub signatures: Vec<EdgeSignature>,
    pub footprints: Vec<EdgeFootprint>,

    // Arity-specific storage
    // For k=2, tails are stored as [t0, t1, t0, t1, ...]
    pub tails_k2: Vec<NodeIx>,
    pub edges_k2: Vec<EdgeIx>,

    // For k=3, tails are stored as [t0, t1, t2, t0, t1, t2, ...]
    pub tails_k3: Vec<NodeIx>,
    pub edges_k3: Vec<EdgeIx>,

    // Variable-arity edges (packed CSR)
    pub tail_off_var: Vec<u32>,
    pub tails_var: Vec<NodeIx>,
    pub edges_var: Vec<EdgeIx>,
}

impl HypergraphDyn {
    pub fn new(graph_epoch: Epoch) -> Self {
        Self {
            graph_epoch,
            dirty_nodes: BitVec::new(),
            dirty_edges: BitVec::new(),
            head: Vec::new(),
            edge_kind: Vec::new(),
            signatures: Vec::new(),
            footprints: Vec::new(),
            tails_k2: Vec::new(),
            edges_k2: Vec::new(),
            tails_k3: Vec::new(),
            edges_k3: Vec::new(),
            tail_off_var: vec![0],
            tails_var: Vec::new(),
            edges_var: Vec::new(),
        }
    }

        fn ensure_node_capacity(&mut self, node_ix: NodeIx) {
        let node_ix = node_ix as usize;
        if node_ix >= self.dirty_nodes.len() {
            self.dirty_nodes.grow(node_ix + 1 - self.dirty_nodes.len(), false);
        }
    }

    pub fn add_edge(&mut self, tails: &[NodeIx], head: NodeIx, footprint: EdgeFootprint) -> EdgeIx {
        let e = self.head.len() as EdgeIx;
        self.head.push(head);
        self.footprints.push(footprint);
        self.dirty_edges.grow(e as usize + 1 - self.dirty_edges.len(), false);
        self.dirty_edges.set(e as usize, true);

        self.ensure_node_capacity(head);
        self.dirty_nodes.set(head as usize, true);
        for &t in tails {
            self.ensure_node_capacity(t);
            self.dirty_nodes.set(t as usize, true);
        }
        // self.edge_kind.push(kind);

        match tails.len() {
            2 => {
                self.tails_k2.extend_from_slice(tails);
                self.edges_k2.push(e);
            }
            3 => {
                self.tails_k3.extend_from_slice(tails);
                self.edges_k3.push(e);
            }
            _ => {
                self.tails_var.extend_from_slice(tails);
                self.tail_off_var.push(self.tails_var.len() as u32);
                self.edges_var.push(e);
            }
        }

        self.graph_epoch = self.graph_epoch.wrapping_add(1);
        e
    }

    pub fn add_edge_checked<I: Integrator>(
        &mut self,
        _manifold: &ManifoldStore<I>,
        signature: EdgeSignature,
        tails: &[NodeIx],
        head: NodeIx,
        footprint: EdgeFootprint,
    ) -> Result<EdgeIx, ()> {
        if signature.tail_manifolds.len() != tails.len() {
            return Err(());
        }

        // for (i, &tail) in tails.iter().enumerate() {
        //     if manifold.node_manifold[tail as usize] != signature.tail_manifolds[i] {
        //         return Err(());
        //     }
        // }

        // if manifold.node_manifold[head as usize] != signature.head_manifold {
        //     return Err(());
        // }

        let e = self.add_edge(tails, head, footprint);
        self.signatures.push(signature);
        Ok(e)
    }

    pub fn clear_dirty(&mut self) {
        self.dirty_nodes.clear();
        self.dirty_edges.clear();
    }
}
