//! Testkit convention:
//! All demo tests are expressed in 2D for clarity,
//! but are executed on the real 3D ManifoldStore
//! with z = 0.0. No test-specific geometry paths exist.

pub type NodeIx = u32;
pub type EdgeIx = u32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Point2> for Point3 {
    #[inline]
    fn from(p: Point2) -> Self {
        Point3 { x: p.x, y: p.y, z: 0.0 }
    }
}

impl From<Point3> for Point2 {
    #[inline]
    fn from(p: Point3) -> Self {
        Point2 { x: p.x, y: p.y }
    }
}

impl From<&Point2> for Point3 {
    #[inline]
    fn from(p: &Point2) -> Self {
        Point3 { x: p.x, y: p.y, z: 0.0 }
    }
}

impl From<&Point3> for Point2 {
    #[inline]
    fn from(p: &Point3) -> Self {
        Point2 { x: p.x, y: p.y }
    }
}

#[derive(Clone, Debug)]
pub struct DemoCase {
    pub name: &'static str,
    pub points: Vec<Point2>,
    pub edges: Vec<(Vec<NodeIx>, NodeIx)>, // tails, head
    pub eps: f32,

    // expectations
    pub expected_fired: Vec<EdgeIx>,
    pub expected_moved: Vec<NodeIx>,
    pub expected_fixed: Vec<NodeIx>,
    pub expected_points: Option<Vec<Point2>>, // when you want exact geometry
}

#[derive(Clone, Debug)]
pub struct RunResult {
    pub fired_edges: Vec<EdgeIx>,
    pub moved_nodes: Vec<NodeIx>,
    pub points: Vec<Point2>,
    pub manifold_epoch: u64,
}

pub fn assert_points_close(got: &[Point2], exp: &[Point2], eps: f32) {
    assert_eq!(got.len(), exp.len());
    for (i, (a, b)) in got.iter().zip(exp.iter()).enumerate() {
        let dx = (a.x - b.x).abs();
        let dy = (a.y - b.y).abs();
        assert!(dx <= eps && dy <= eps, "point {i} mismatch: got={a:?} exp={b:?}");
    }
}

pub fn assert_set_eq(mut got: Vec<u32>, mut exp: Vec<u32>) {
    got.sort_unstable();
    got.dedup();
    exp.sort_unstable();
    exp.dedup();
    assert_eq!(got, exp);
}

pub fn run_one_step(case: &DemoCase) -> RunResult {
    
    use hypergraph::dynamic::HypergraphDyn;
    use hypergraph::freeze::freeze_checked;
    use manifold::store::{ManifoldStore, ExplicitEuler};
    use manifold::footprint::EdgeFootprint;
    use exec::gate::DistanceGate;
    use exec::engine::Engine;
    use exec::delta::DeltaBuf;
    use exec::views::{GateView, FrozenGraphView};

    // 1) Build HypergraphDyn
    let mut g = HypergraphDyn::new(0u64);
    for (tails, head) in &case.edges {
        let footprint = EdgeFootprint::V1(manifold::footprint::EdgeFootprintV1 {
            influence_radius: 1.0, // simple radius for distance gates
            anchor: None,
        });
        g.add_edge(tails, *head, footprint);
    }

    // 2) Freeze
    let (frozen, _) = freeze_checked(g, Default::default());

    // 3) Build ManifoldStore from points (3D with z=0)
    let points_3d: Vec<manifold::store::Point> = case.points.iter().map(|p| manifold::store::Point::new(p.x, p.y, 0.0)).collect();
    let mut mani = ManifoldStore::<ExplicitEuler>::new_with_points(points_3d, ExplicitEuler);

    // 4) Build GateCache
    let mut gate = DistanceGate { eps: case.eps, cache: Default::default() };

    // 5) Schedule: dirty all tails
    for (tails, _) in &case.edges {
        for &tail in tails {
            mani.dirty_point(tail);
        }
    }

    // 6) Run compute
    let mut engine = Engine::with_capacity(1024);
    let mut deltas = DeltaBuf::with_capacity(1024);

    // Compute gates first
    for e in 0..frozen.edge_count() {
        gate.allow(&mani, &frozen, e as u32);
    }

    // Get fired edges from cache
    let fired_edges: Vec<EdgeIx> = gate.cache.edge_active.iter().enumerate()
        .filter(|(_, &active)| active != 0)
        .map(|(i, _)| i as EdgeIx)
        .collect();

    // Run engine
    engine.run_compute(&frozen, &GateView {
        edge_active: &gate.cache.edge_active,
        edge_weight: &gate.cache.edge_weight,
    }, &mani, &mut deltas);

    // Apply deltas
    let mut moved_nodes = Vec::new();
    for delta in &deltas.deltas {
        if let exec::delta::Delta::PointUpdate(node, _) = delta {
            moved_nodes.push(*node);
        }
    }
    exec::apply::apply_deltas(&mut mani, &mut deltas.deltas);

    // Project points back to 2D
    let points_2d: Vec<Point2> = mani.points.iter().map(|p| Point2 { x: p.x, y: p.y }).collect();

    RunResult {
        fired_edges,
        moved_nodes,
        points: points_2d,
        manifold_epoch: mani.epoch as u64,
    }
}