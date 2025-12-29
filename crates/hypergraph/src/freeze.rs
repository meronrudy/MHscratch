use traits::footprint::Footprint;
use crate::arity::ArityGroup;
use crate::delta::FrozenDelta;
use crate::dynamic::HypergraphDyn;
use crate::frozen::HypergraphFrozen;
use crate::FrozenBase;
use core::ids::{EdgeIx, Epoch};
use std::mem;

#[derive(Clone, Debug)]
pub struct FreezeReport {
    pub inferred_n_nodes: u32,
    pub edges_in: u32,
    pub edges_out: u32,
    pub tails_in: u32,
    pub tails_out: u32,
    pub had_tombstones: bool,
    pub had_node_remap: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct FreezeCheckedOpts {
    pub force_compact_edges: bool,
    pub densify_nodes: bool,
}
impl Default for FreezeCheckedOpts {
    fn default() -> Self {
        Self {
            force_compact_edges: false,
            densify_nodes: true,
        }
    }
}

pub fn freeze_checked(
    g: HypergraphDyn,
    opts: FreezeCheckedOpts,
) -> (HypergraphFrozen, FreezeReport) {
    let edges_in = g.head.len() as u32;
    let tails_in =
        g.tails_var.len() as u32 + g.tails_k2.len() as u32 + g.tails_k3.len() as u32;
    // let had_tombstones = !g.edge_alive.is_empty() && g.edge_alive.iter().any(|&a| a == 0);

    // // Step 1) compact edges (remove tombstones) into a tight, stable order
    // if had_tombstones || opts.force_compact_edges {
    //     g = compact_edges(g);
    // }

    // Step 2) infer max node id
    let inferred_n_nodes = infer_n_nodes(&g);

    // // Step 3) optionally remap nodes to dense [0..n_new)
    // let mut had_node_remap = false;
    // if opts.densify_nodes {
    //     if let Some(plan) = build_node_remap_plan(&g, inferred_n_nodes) {
    //         had_node_remap = true;
    //         apply_node_remap(&mut g, &plan);
    //     }
    // }

    // After remap, infer n_nodes again (now dense)
    let n_nodes = infer_n_nodes(&g);

    // Step 4) build CSR adjacency (same two-pass approach)
    let frozen = freeze_csr(g, n_nodes);

    let report = FreezeReport {
        inferred_n_nodes,
        edges_in,
        edges_out: frozen.n_edges,
        tails_in: tails_in as u32,
        tails_out:
            frozen.tails_var.len() as u32
                + frozen.tails_k2.len() as u32
                + frozen.tails_k3.len() as u32,
        had_tombstones: false, // TODO
        had_node_remap: false, // TODO
    };

    (frozen, report)
}

fn infer_n_nodes(g: &HypergraphDyn) -> u32 {
    let mut max_id: i64 = -1;
    for &h in &g.head {
        max_id = max_id.max(h as i64);
    }
    for &t in &g.tails_k2 {
        max_id = max_id.max(t as i64);
    }
    for &t in &g.tails_k3 {
        max_id = max_id.max(t as i64);
    }
    for &t in &g.tails_var {
        max_id = max_id.max(t as i64);
    }
    (max_id + 1).max(0) as u32
}

fn freeze_csr(mut g: HypergraphDyn, n_nodes: u32) -> HypergraphFrozen {
    let n_edges = g.head.len() as u32;

    let mut edge_info = vec![(ArityGroup::K2, 0); n_edges as usize];
    for (i, &e) in g.edges_k2.iter().enumerate() {
        edge_info[e as usize] = (ArityGroup::K2, i as u32);
    }
    for (i, &e) in g.edges_k3.iter().enumerate() {
        edge_info[e as usize] = (ArityGroup::K3, i as u32);
    }
    for (i, &e) in g.edges_var.iter().enumerate() {
        edge_info[e as usize] = (ArityGroup::Var, i as u32);
    }

    // Pass 1: degree counts
    let mut out_deg = vec![0u32; n_nodes as usize];
    let mut in_deg = vec![0u32; n_nodes as usize];

    for e in 0..n_edges {
        let head = g.head[e as usize];
        in_deg[head as usize] += 1;
    }
    for i in 0..g.edges_k2.len() {
        let t0 = g.tails_k2[i * 2];
        let t1 = g.tails_k2[i * 2 + 1];
        out_deg[t0 as usize] += 1;
        out_deg[t1 as usize] += 1;
    }
    for i in 0..g.edges_k3.len() {
        let t0 = g.tails_k3[i * 3];
        let t1 = g.tails_k3[i * 3 + 1];
        let t2 = g.tails_k3[i * 3 + 2];
        out_deg[t0 as usize] += 1;
        out_deg[t1 as usize] += 1;
        out_deg[t2 as usize] += 1;
    }
    for i in 0..g.edges_var.len() {
        let t0 = g.tail_off_var[i] as usize;
        let t1 = g.tail_off_var[i + 1] as usize;
        for &t in &g.tails_var[t0..t1] {
            out_deg[t as usize] += 1;
        }
    }

    // Offsets
    let mut out_off = vec![0u32; (n_nodes as usize) + 1];
    let mut in_off = vec![0u32; (n_nodes as usize) + 1];
    prefix_sum_into(&out_deg, &mut out_off);
    prefix_sum_into(&in_deg, &mut in_off);

    let out_total = out_off[n_nodes as usize] as usize;
    let in_total = in_off[n_nodes as usize] as usize;

    let mut out_edges = vec![0u32; out_total];
    let mut in_edges = vec![0u32; in_total];

    // Fill cursors
    let mut out_cur = out_off[..n_nodes as usize].to_vec();
    let mut in_cur = in_off[..n_nodes as usize].to_vec();

    for e in 0..n_edges {
        let eix = e as EdgeIx;
        let head = g.head[e as usize];

        let pos_in = in_cur[head as usize] as usize;
        in_edges[pos_in] = eix;
        in_cur[head as usize] += 1;
    }

    for i in 0..g.edges_k2.len() {
        let e = g.edges_k2[i];
        let t0 = g.tails_k2[i * 2];
        let t1 = g.tails_k2[i * 2 + 1];

        let pos_out = out_cur[t0 as usize] as usize;
        out_edges[pos_out] = e;
        out_cur[t0 as usize] += 1;

        let pos_out = out_cur[t1 as usize] as usize;
        out_edges[pos_out] = e;
        out_cur[t1 as usize] += 1;
    }
    for i in 0..g.edges_k3.len() {
        let e = g.edges_k3[i];
        let t0 = g.tails_k3[i * 3];
        let t1 = g.tails_k3[i * 3 + 1];
        let t2 = g.tails_k3[i * 3 + 2];

        let pos_out = out_cur[t0 as usize] as usize;
        out_edges[pos_out] = e;
        out_cur[t0 as usize] += 1;

        let pos_out = out_cur[t1 as usize] as usize;
        out_edges[pos_out] = e;
        out_cur[t1 as usize] += 1;

        let pos_out = out_cur[t2 as usize] as usize;
        out_edges[pos_out] = e;
        out_cur[t2 as usize] += 1;
    }
    for i in 0..g.edges_var.len() {
        let e = g.edges_var[i];
        let t0 = g.tail_off_var[i] as usize;
        let t1 = g.tail_off_var[i + 1] as usize;
        for &t in &g.tails_var[t0..t1] {
            let pos_out = out_cur[t as usize] as usize;
            out_edges[pos_out] = e;
            out_cur[t as usize] += 1;
        }
    }

    let footprints = vec![
        Footprint {
            influence_radius: 0.0,
            anchor: None,
        };
        n_edges as usize
    ];

    HypergraphFrozen {
        epoch: g.epoch as Epoch,
        n_nodes,
        n_edges,
        edge_head: mem::take(&mut g.head),
        edge_kind: mem::take(&mut g.edge_kind),
        edge_info,
        signatures: mem::take(&mut g.signatures),
        footprints,
        tails_k2: mem::take(&mut g.tails_k2),
        edges_k2: mem::take(&mut g.edges_k2),
        tails_k3: mem::take(&mut g.tails_k3),
        edges_k3: mem::take(&mut g.edges_k3),
        tail_off_var: mem::take(&mut g.tail_off_var),
        tails_var: mem::take(&mut g.tails_var),
        edges_var: mem::take(&mut g.edges_var),
        out_off,
        out_edges,
        in_off,
        in_edges,
    }
}

#[inline]
fn prefix_sum_into(deg: &[u32], off: &mut [u32]) {
    debug_assert_eq!(off.len(), deg.len() + 1);
    off[0] = 0;
    for i in 0..deg.len() {
        off[i + 1] = off[i] + deg[i];
    }
}

pub fn freeze_incremental(
    _g: &HypergraphDyn,
    _base: &FrozenBase,
) -> FrozenDelta {
    // TODO: implement this
    FrozenDelta {
        added_edges: vec![],
        removed_edges: vec![],
    }
}
