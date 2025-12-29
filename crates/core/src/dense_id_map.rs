use std::collections::HashMap;
use crate::ids::NodeIx;

pub type ExtNodeId = u64;

pub struct DenseIdMapSnapshot {
    pub ext_to_dense: HashMap<ExtNodeId, NodeIx>,
    pub dense_to_ext: Vec<ExtNodeId>,
    pub next_dense_id: u32,
    pub free_list: Vec<NodeIx>,
}

pub struct DenseIdMap {
    ext_to_dense: HashMap<ExtNodeId, NodeIx>,
    dense_to_ext: Vec<ExtNodeId>,
    next_dense_id: u32,
    free_list: Vec<NodeIx>,
}

impl DenseIdMap {
    pub fn new() -> Self {
        Self {
            ext_to_dense: HashMap::new(),
            dense_to_ext: Vec::new(),
            next_dense_id: 0,
            free_list: Vec::new(),
        }
    }

    pub fn get_or_insert(&mut self, ext_id: ExtNodeId) -> NodeIx {
        if let Some(dense_id) = self.ext_to_dense.get(&ext_id) {
            return *dense_id;
        }

        if let Some(recycled_id) = self.free_list.pop() {
            self.ext_to_dense.insert(ext_id, recycled_id);
            self.dense_to_ext[recycled_id as usize] = ext_id;
            return recycled_id;
        }

        let dense_id = self.next_dense_id;
        self.ext_to_dense.insert(ext_id, dense_id);
        self.dense_to_ext.push(ext_id);
        self.next_dense_id += 1;
        dense_id
    }

    pub fn recycle(&mut self, ext_id: ExtNodeId) {
        if let Some(dense_id) = self.ext_to_dense.remove(&ext_id) {
            self.free_list.push(dense_id);
            // Optionally, mark the entry in dense_to_ext as invalid
        }
    }

    pub fn snapshot(&self) -> DenseIdMapSnapshot {
        DenseIdMapSnapshot {
            ext_to_dense: self.ext_to_dense.clone(),
            dense_to_ext: self.dense_to_ext.clone(),
            next_dense_id: self.next_dense_id,
            free_list: self.free_list.clone(),
        }
    }

    pub fn restore(&mut self, snapshot: DenseIdMapSnapshot) {
        self.ext_to_dense = snapshot.ext_to_dense;
        self.dense_to_ext = snapshot.dense_to_ext;
        self.next_dense_id = snapshot.next_dense_id;
        self.free_list = snapshot.free_list;
    }
}
