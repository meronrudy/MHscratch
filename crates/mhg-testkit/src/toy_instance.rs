//! Tiny toy instance module for playground cases.
//! "Enough to run the triangle case and one or two variants, nothing more."

use crate::{NodeIx, EdgeIx, Point2};

/// Edge influence configuration
#[derive(Clone, Debug)]
pub enum ToyFootprint {
    /// v1: just influence radius, optional explicit anchor
    Influence {
        radius: f32,
        anchor: Option<NodeIx>,
    },
    /// v2: simplex proxy via tail indices
    Simplex {
        tails: Vec<NodeIx>,
        radius: f32,
    },
}

/// Gate condition types
#[derive(Clone, Debug)]
pub enum ToyGateKind {
    /// active iff eps < dist(head, centroid(tails)) < radius
    CentroidRadiusWindow {
        eps: f32,
        radius: f32,
    },
    /// active iff dist(n0, n1) < threshold
    PairDistanceLessThan {
        n0: NodeIx,
        n1: NodeIx,
        threshold: f32,
    },
}

/// Firing behavior types
#[derive(Clone, Debug)]
pub enum ToyFireKind {
    /// head moves α toward centroid(tails)
    CentroidPull {
        alpha: f32,
    },
    /// no-op (for counterfactual variants)
    Noop,
}

/// Edge configuration combining footprint, gate, and fire
#[derive(Clone, Debug)]
pub struct ToyEdge {
    pub id: EdgeIx,
    pub tails: Vec<NodeIx>,
    pub head: NodeIx,
    pub footprint: ToyFootprint,
    pub gate: ToyGateKind,
    pub fire: ToyFireKind,
}

/// Execution schedule: fixed number of ticks with dirty nodes per tick
#[derive(Clone, Debug)]
pub struct ToySchedule {
    pub ticks: u32,
    /// map tick → dirty nodes (push mode)
    pub dirty_by_tick: Vec<Vec<NodeIx>>,
}

/// Main toy instance: name, points, edges, schedule, tolerance
#[derive(Clone, Debug)]
pub struct ToyInstance {
    pub name: &'static str,
    /// 2D authoring; will be embedded as (x, y, 0.0)
    pub points_xy: Vec<Point2>,
    pub edges: Vec<ToyEdge>,
    pub schedule: ToySchedule,
    /// generic numeric tolerance used by simple demos
    pub eps_xy: f32,
}

/// Concrete instance: triangle attractor
pub fn triangle_attractor_instance() -> ToyInstance {
    // n0, n1, n2 form an almost-equilateral triangle; n3 is below centroid
    let points_xy = vec![
        Point2 { x: 0.0, y: 0.0 },        // n0
        Point2 { x: 1.0, y: 0.0 },        // n1
        Point2 { x: 0.5, y: 0.866_025_4 },// n2
        Point2 { x: 0.5, y: 0.20 },       // n3
    ];

    let n0: NodeIx = 0;
    let n1: NodeIx = 1;
    let n2: NodeIx = 2;
    let n3: NodeIx = 3;

    let e0: EdgeIx = 0;

    let edges = vec![
        ToyEdge {
            id: e0,
            tails: vec![n0, n1, n2],
            head: n3,
            footprint: ToyFootprint::Simplex {
                tails: vec![n0, n1, n2],
                radius: 0.6,
            },
            gate: ToyGateKind::CentroidRadiusWindow {
                eps: 0.01,
                radius: 0.6,
            },
            fire: ToyFireKind::CentroidPull {
                alpha: 0.5,
            },
        }
    ];

    // 10 ticks; for now, treat the first 3 nodes as dirty at each step
    let ticks = 10;
    let mut dirty_by_tick = Vec::with_capacity(ticks as usize);
    for _ in 0..ticks {
        dirty_by_tick.push(vec![n0, n1, n2]);
    }

    let schedule = ToySchedule {
        ticks,
        dirty_by_tick,
    };

    ToyInstance {
        name: "triangle_attractor_v0",
        points_xy,
        edges,
        schedule,
        eps_xy: 0.01,
    }
}

/// Thin adapter: ToyInstance → real engine
pub fn run_toy_instance(inst: &ToyInstance) {
    use hypergraph::dynamic::HypergraphDyn;
    use hypergraph::freeze::freeze_checked;
    use hypergraph::frozen::FrozenIncidence;
    use manifold::store::{ManifoldStore, ExplicitEuler};
    use manifold::footprint::EdgeFootprint;
    use exec::gate::DistanceGate;
    use exec::engine::Engine;
    use exec::delta::DeltaBuf;
    use exec::views::GateView;

    // 1) embed 2D → 3D
    let pts3: Vec<manifold::store::Point> = inst.points_xy.iter()
        .map(|p| manifold::store::Point::new(p.x, p.y, 0.0))
        .collect();

    // 2) build dynamic graph - nodes are created implicitly when adding edges
    let mut hg = HypergraphDyn::new(0u64);
    let node_count = pts3.len();
    let node_ixs: Vec<NodeIx> = (0..node_count).map(|i| i as NodeIx).collect();

    for edge in &inst.edges {
        let tails_internal: Vec<NodeIx> = edge.tails.iter()
            .map(|&n| node_ixs[n as usize])
            .collect();
        let head_internal = node_ixs[edge.head as usize];

        // For now, use a simple footprint - in a real implementation,
        // we'd translate ToyFootprint to the actual footprint type
        let footprint = EdgeFootprint::V1(
            manifold::footprint::EdgeFootprintV1 {
                influence_radius: 1.0, // simplified for toy
                anchor: None,
            }
        );

        let _e_internal = hg.add_edge(&tails_internal, head_internal, footprint);
    }

    let frozen = freeze_checked(hg, Default::default()).0;

    // 3) manifold
    let mut manifold = ManifoldStore::<ExplicitEuler>::new_with_points(pts3, ExplicitEuler);

    // 4) Simplified execution - for toy instances, assume all gates are active
    // TODO: Implement proper gate evaluation for different gate kinds
    let mut engine = Engine::with_capacity(256);
    let mut deltas = DeltaBuf::with_capacity(256);

    // For toy instances, create a simple gate cache where all edges are active
    let num_edges = frozen.edge_count();
    let mut edge_active = vec![1u8; num_edges]; // All edges active
    let mut edge_weight = vec![1.0f32; num_edges]; // Dummy weights

    // 5) Run compute once (simplified single-step execution)
    engine.run_compute(&frozen, &exec::views::GateView {
        edge_active: &edge_active,
        edge_weight: &edge_weight,
    }, &manifold, &mut deltas);

    // Apply deltas
    exec::apply::apply_deltas(&mut manifold, &mut deltas.deltas);

    // 7) sanity: still in XY-plane
    #[cfg(debug_assertions)]
    for (i, p) in manifold.points.iter().enumerate() {
        debug_assert!(p.z == 0.0, "z drift at node {i}: {:?}", p);
    }
}