use crate::signature::EdgeSignature;
use core::ids::{EdgeIx, NodeIx};
use manifold::store::ManifoldStore;

pub struct HypergraphDyn {
    pub epoch: u64,
    pub dirty_nodes: Vec<u8>,
    pub dirty_edges: Vec<u8>,

    // Edge payload (common to all arities)
    pub head: Vec<NodeIx>,
    pub edge_kind: Vec<u16>,
    pub signatures: Vec<EdgeSignature>,

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
    pub fn new(epoch: u64) -> Self {
        Self {
            epoch,
            dirty_nodes: Vec::new(),
            dirty_edges: Vec::new(),
            head: Vec::new(),
            edge_kind: Vec::new(),
            signatures: Vec::new(),
            tails_k2: Vec::new(),
            edges_k2: Vec::new(),
            tails_k3: Vec::new(),
            edges_k3: Vec::new(),
            tail_off_var: vec![0],
            tails_var: Vec::new(),
            edges_var: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, tails: &[NodeIx], head: NodeIx) -> EdgeIx {
        let e = self.head.len() as EdgeIx;
        self.head.push(head);
        self.dirty_edges.push(1);
        self.dirty_nodes[head as usize] = 1;
        for &t in tails {
            self.dirty_nodes[t as usize] = 1;
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

        self.epoch = self.epoch.wrapping_add(1);
        e
    }

    pub fn add_edge_checked(
        &mut self,
        manifold: &ManifoldStore,
        signature: EdgeSignature,
        tails: &[NodeIx],
        head: NodeIx,
    ) -> Result<EdgeIx, ()> {
        if signature.tail_manifolds.len() != tails.len() {
            return Err(());
        }

        for (i, &tail) in tails.iter().enumerate() {
            if manifold.node_manifold[tail as usize] != signature.tail_manifolds[i] {
                return Err(());
            }
        }

        if manifold.node_manifold[head as usize] != signature.head_manifold {
            return Err(());
        }

        let e = self.add_edge(tails, head);
        self.signatures.push(signature);
        Ok(e)
    }
}
