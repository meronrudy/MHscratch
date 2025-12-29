use core::ids::{EdgeIx, NodeIx, Epoch};

/// Configuration for trace verbosity
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TraceVerbosity {
    #[default]
    Off,
    Basic,
    Detailed,
}

/// Trace capture for verification and replay
#[derive(Clone, Debug, Default)]
pub struct Trace {
    pub fired_edges: Vec<EdgeIx>,
    pub visited_nodes: Vec<NodeIx>,
    pub geodesic_calls: Option<Vec<GeodesicCall>>,
    pub constraints: Vec<Constraint>,
    pub epochs: Vec<Epoch>,
    verbosity: TraceVerbosity,
}

#[derive(Clone, Debug)]
pub struct GeodesicCall {
    pub node: NodeIx,
    pub result: f32,
}

#[derive(Clone, Debug)]
pub enum Constraint {
    NodeBounds(NodeIx, f32, f32), // node, min, max
    EdgeSignature(EdgeIx, String), // edge, signature description
}

impl Trace {
    /// Create a new trace with specified verbosity
    pub fn new(verbosity: TraceVerbosity) -> Self {
        Self {
            verbosity,
            ..Default::default()
        }
    }

    /// Check if tracing is enabled
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.verbosity != TraceVerbosity::Off
    }

    /// Record a fired edge (no-op if disabled)
    #[inline]
    pub fn record_edge_fire(&mut self, edge: EdgeIx) {
        if self.is_enabled() {
            self.fired_edges.push(edge);
        }
    }

    /// Record a visited node (no-op if disabled)
    #[inline]
    pub fn record_node_visit(&mut self, node: NodeIx) {
        if self.is_enabled() {
            self.visited_nodes.push(node);
        }
    }

    /// Record a geodesic call (no-op if disabled)
    #[inline]
    pub fn record_geodesic_call(&mut self, node: NodeIx, result: f32) {
        if self.is_enabled() {
            self.geodesic_calls.get_or_insert_with(Vec::new).push(GeodesicCall { node, result });
        }
    }

    /// Record a constraint violation
    #[inline]
    pub fn record_constraint(&mut self, constraint: Constraint) {
        if self.is_enabled() {
            self.constraints.push(constraint);
        }
    }

    /// Record epoch advancement
    #[inline]
    pub fn record_epoch(&mut self, epoch: Epoch) {
        if self.is_enabled() {
            self.epochs.push(epoch);
        }
    }

    /// Clear all trace data
    pub fn clear(&mut self) {
        self.fired_edges.clear();
        self.visited_nodes.clear();
        if let Some(calls) = &mut self.geodesic_calls {
            calls.clear();
        }
        self.constraints.clear();
        self.epochs.clear();
    }
}