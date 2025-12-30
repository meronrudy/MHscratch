//! Toy hardware backend: constrained cycle-model for hardware design exploration
//! Implements GESC/GESA-style architecture with event packets, credits, barriers, and loci

use crate::{NodeIx, EdgeIx, Point2, HwCounts};
use std::convert::From;

/// GESC-style 64-bit event packet
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Gescevent {
    pub raw: u64,
}

impl Gescevent {
    pub fn new(src_id: u32, dst_scope: u32, class: u8, tstamp: u16, qos: u8, payload_id: u8) -> Self {
        let mut raw = 0u64;
        raw |= ((src_id as u64) & 0x3FFFF) << 46;     // 18 bits src_id
        raw |= ((dst_scope as u64) & 0x3FFFF) << 28;   // 18 bits dst_scope
        raw |= ((class as u64) & 0xF) << 24;           // 4 bits class
        raw |= ((tstamp as u64) & 0xFFFF) << 8;        // 16 bits tstamp
        raw |= ((qos as u64) & 0xF) << 4;              // 4 bits qos
        raw |= (payload_id as u64) & 0xF;              // 4 bits payload_id
        Gescevent { raw }
    }

    pub fn src_id(self) -> u32     { ((self.raw >> 46) & 0x3FFFF) as u32 }
    pub fn dst_scope(self) -> u32  { ((self.raw >> 28) & 0x3FFFF) as u32 }
    pub fn class(self) -> u8       { ((self.raw >> 24) & 0xF) as u8 }
    pub fn tstamp(self) -> u16     { ((self.raw >> 8) & 0xFFFF) as u16 }
    pub fn qos(self) -> u8         { ((self.raw >> 4) & 0xF) as u8 }
    pub fn payload_id(self) -> u8  { (self.raw & 0xF) as u8 }
}

/// GESC event classes
pub const GEOM_CLASS: u8 = 0x0;   // Geometric updates (point deltas)
pub const FIELD_CLASS: u8 = 0x1;  // Scalar field updates
pub const TOPO_CLASS: u8 = 0x3;   // Structural operations

/// Event payload table for local interpretations
#[derive(Clone, Debug)]
pub struct PayloadTable {
    pub entries: [PayloadEntry; 16], // Fixed 16 entries
}

#[derive(Clone, Debug)]
pub enum PayloadEntry {
    Float(Q16_16),
    Index(u16),
    ParamId(u16),
    DeltaX(Q16_16),  // For geometric updates
    DeltaY(Q16_16),
    DeltaZ(Q16_16),
}

/// Credit-based flow control
#[derive(Clone, Debug)]
pub struct CreditTable {
    pub credits: [u16; N_NODES_MAX], // Per destination scope
    pub max_credits: u16,
    pub credit_consumes: u64,
    pub credit_releases: u64,
    pub credit_denies: u64,
}

impl CreditTable {
    pub fn new(max_credits: u16) -> Self {
        Self {
            credits: [max_credits; N_NODES_MAX],
            max_credits,
            credit_consumes: 0,
            credit_releases: 0,
            credit_denies: 0,
        }
    }

    pub fn can_send(&self, scope: u32) -> bool {
        self.credits[scope as usize] > 0
    }

    pub fn consume(&mut self, scope: u32) -> bool {
        if self.can_send(scope) {
            self.credits[scope as usize] -= 1;
            self.credit_consumes += 1;
            true
        } else {
            self.credit_denies += 1;
            false
        }
    }

    pub fn release(&mut self, scope: u32) {
        if self.credits[scope as usize] < self.max_credits {
            self.credits[scope as usize] += 1;
            self.credit_releases += 1;
        }
    }
}

/// Structural operation intents for TOPO events
#[derive(Clone, Debug)]
pub enum TopoIntentKind {
    AddEdge { tails: [NodeIx; 4], head: NodeIx, arity: u8 },
    RemoveEdge { edge: EdgeIx },
    SplitEdge { edge: EdgeIx },
}

#[derive(Clone, Debug)]
pub struct TopoIntent {
    pub edge_local_id: u16,
    pub kind: TopoIntentKind,
}

/// Fixed-point Q16.16 representation for hardware simulation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Q16_16(pub i32);

impl Q16_16 {
    pub const ONE: Q16_16 = Q16_16(1 << 16);
    pub const ZERO: Q16_16 = Q16_16(0);

    pub fn from_float(f: f32) -> Self {
        Q16_16((f * (1 << 16) as f32) as i32)
    }

    pub fn to_float(self) -> f32 {
        self.0 as f32 / (1 << 16) as f32
    }

    pub fn add(self, other: Q16_16) -> Q16_16 {
        Q16_16(self.0 + other.0)
    }

    pub fn sub(self, other: Q16_16) -> Q16_16 {
        Q16_16(self.0 - other.0)
    }

    pub fn mul(self, other: Q16_16) -> Q16_16 {
        Q16_16(((self.0 as i64 * other.0 as i64) >> 16) as i32)
    }

    pub fn sqrt_approx(self) -> Q16_16 {
        // Simple approximation for sqrt - in hardware this would be a dedicated unit
        if self.0 <= 0 {
            Q16_16::ZERO
        } else {
            Q16_16::from_float(self.to_float().sqrt())
        }
    }

    pub fn div_approx(self, other: Q16_16) -> Q16_16 {
        if other.0 == 0 {
            Q16_16::ZERO
        } else {
            let result = ((self.0 as i64) << 16) / other.0 as i64;
            Q16_16(result as i32)
        }
    }
}

impl From<f32> for Q16_16 {
    fn from(f: f32) -> Self {
        Q16_16::from_float(f)
    }
}

impl From<Point2> for (Q16_16, Q16_16) {
    fn from(p: Point2) -> Self {
        (Q16_16::from_float(p.x), Q16_16::from_float(p.y))
    }
}

/// Hardware constraints
pub const N_NODES_MAX: usize = 32;
pub const N_EDGES_MAX: usize = 32;
pub const ARITY_MAX: usize = 4;
pub const EVENT_QUEUE_DEPTH: usize = 64;
pub const LANES: usize = 2; // Number of parallel compute lanes

/// Node memory: SoA arrays with read/write counters
#[derive(Clone, Debug)]
pub struct NodeMem {
    pub x: [Q16_16; N_NODES_MAX],
    pub y: [Q16_16; N_NODES_MAX],
    pub z: [Q16_16; N_NODES_MAX],
    pub count: usize,
    pub reads: u64,
    pub writes: u64,
}

impl NodeMem {
    pub fn new() -> Self {
        Self {
            x: [Q16_16::ZERO; N_NODES_MAX],
            y: [Q16_16::ZERO; N_NODES_MAX],
            z: [Q16_16::ZERO; N_NODES_MAX],
            count: 0,
            reads: 0,
            writes: 0,
        }
    }

    pub fn set_point(&mut self, idx: usize, point: (Q16_16, Q16_16, Q16_16)) {
        self.x[idx] = point.0;
        self.y[idx] = point.1;
        self.z[idx] = point.2;
        self.writes += 3; // x, y, z coordinates
    }

    pub fn get_point(&mut self, idx: usize) -> (Q16_16, Q16_16, Q16_16) {
        self.reads += 3;
        (self.x[idx], self.y[idx], self.z[idx])
    }
}

/// Edge memory: packed storage with footprint parameters
#[derive(Clone, Debug)]
pub struct EdgeMem {
    pub tails: [[NodeIx; ARITY_MAX]; N_EDGES_MAX],
    pub arity: [usize; N_EDGES_MAX],
    pub head: [NodeIx; N_EDGES_MAX],
    pub radius: [Q16_16; N_EDGES_MAX],
    pub count: usize,
    pub reads: u64,
}

impl EdgeMem {
    pub fn new() -> Self {
        Self {
            tails: [[0; ARITY_MAX]; N_EDGES_MAX],
            arity: [0; N_EDGES_MAX],
            head: [0; N_EDGES_MAX],
            radius: [Q16_16::ZERO; N_EDGES_MAX],
            count: 0,
            reads: 0,
        }
    }

    pub fn set_edge(&mut self, idx: usize, tails: &[NodeIx], head: NodeIx, radius: Q16_16) {
        let arity = tails.len().min(ARITY_MAX);
        self.arity[idx] = arity;
        self.head[idx] = head;
        self.radius[idx] = radius;
        for i in 0..arity {
            self.tails[idx][i] = tails[i];
        }
    }

    pub fn get_edge(&mut self, idx: usize) -> (&[NodeIx], NodeIx, Q16_16) {
        self.reads += 1;
        let arity = self.arity[idx];
        (&self.tails[idx][..arity], self.head[idx], self.radius[idx])
    }
}

/// Bounded event queue with GESC events
#[derive(Clone, Debug)]
pub struct EventQueue {
    pub events: [Gescevent; EVENT_QUEUE_DEPTH],
    pub head: usize,
    pub tail: usize,
    pub count: usize,
    pub pushes: u64,
    pub pops: u64,
    pub overflows: u64,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            events: [Gescevent { raw: 0 }; EVENT_QUEUE_DEPTH],
            head: 0,
            tail: 0,
            count: 0,
            pushes: 0,
            pops: 0,
            overflows: 0,
        }
    }

    pub fn push(&mut self, event: Gescevent) -> bool {
        self.pushes += 1;
        if self.count >= EVENT_QUEUE_DEPTH {
            self.overflows += 1;
            return false;
        }

        self.events[self.tail] = event;
        self.tail = (self.tail + 1) % EVENT_QUEUE_DEPTH;
        self.count += 1;
        true
    }

    pub fn pop(&mut self) -> Option<Gescevent> {
        if self.count == 0 {
            return None;
        }

        self.pops += 1;
        let event = self.events[self.head];
        self.head = (self.head + 1) % EVENT_QUEUE_DEPTH;
        self.count -= 1;
        Some(event)
    }
}

/// Locus types for GESA-style computation
#[derive(Clone, Debug)]
pub enum LocusKind {
    Field,      // GLI-like scalar fields
    Topology,   // Monitors edges, emits TOPO events
    Constraint, // Enforces geometric invariants
}

#[derive(Clone, Debug)]
pub struct Locus {
    pub kind: LocusKind,
    pub params: LocusParams,
}

#[derive(Clone, Debug)]
pub enum LocusParams {
    Field { v: Q16_16, decay: Q16_16, bias: Q16_16, threshold: Q16_16 },
    Topology { stress_threshold: Q16_16 },
    Constraint { max_dist: Q16_16 },
}

/// Pending structural operations for barrier
#[derive(Clone, Debug)]
pub struct PendingTopo {
    pub intents: Vec<TopoIntent>,
}

/// Operations that lanes can perform (for conflict detection)
#[derive(Clone, Debug)]
enum LaneOperation {
    GeomField { dst_scope: u32, event: Gescevent },
    Conflicted, // Operation was conflicted out
}

/// Gate evaluation unit
#[derive(Clone, Debug)]
pub struct GateUnit {
    pub eval_calls: u64,
}

impl GateUnit {
    pub fn new() -> Self {
        Self { eval_calls: 0 }
    }

    /// Simplified gate evaluation - for toy, assume centroid-radius-window
    pub fn eval(&mut self, node_mem: &mut NodeMem, edge_mem: &mut EdgeMem, edge: EdgeIx) -> bool {
        self.eval_calls += 1;

        let (tails, head, radius) = edge_mem.get_edge(edge as usize);
        if tails.is_empty() {
            return false;
        }

        // Compute centroid
        let mut cx = Q16_16::ZERO;
        let mut cy = Q16_16::ZERO;
        let mut cz = Q16_16::ZERO;

        for &tail in tails {
            let (tx, ty, tz) = node_mem.get_point(tail as usize);
            cx = cx.add(tx);
            cy = cy.add(ty);
            cz = cz.add(tz);
        }

        let k = Q16_16::from_float(tails.len() as f32);
        cx = cx.mul(Q16_16::ONE).div_approx(k); // cx / k
        cy = cy.mul(Q16_16::ONE).div_approx(k);
        cz = cz.mul(Q16_16::ONE).div_approx(k);

        // Distance from centroid to head
        let (hx, hy, hz) = node_mem.get_point(head as usize);
        let dx = cx.sub(hx);
        let dy = cy.sub(hy);
        let dz = cz.sub(hz);

        let dist_sq = dx.mul(dx).add(dy.mul(dy)).add(dz.mul(dz));
        let dist = dist_sq.sqrt_approx();

        // For toy: active if 0.01 < dist < radius
        let eps = Q16_16::from_float(0.01);
        dist.0 > eps.0 && dist.0 < radius.0
    }
}

/// Geometry computation unit
#[derive(Clone, Debug)]
pub struct GeomUnit {
    pub eval_calls: u64,
}

impl GeomUnit {
    pub fn new() -> Self {
        Self { eval_calls: 0 }
    }

    /// Compute centroid pull delta
    pub fn eval(&mut self, node_mem: &mut NodeMem, edge_mem: &mut EdgeMem, edge: EdgeIx) -> (NodeIx, (Q16_16, Q16_16, Q16_16)) {
        self.eval_calls += 1;

        let (tails, head, _) = edge_mem.get_edge(edge as usize);

        // Compute centroid
        let mut cx = Q16_16::ZERO;
        let mut cy = Q16_16::ZERO;
        let mut cz = Q16_16::ZERO;

        for &tail in tails {
            let (tx, ty, tz) = node_mem.get_point(tail as usize);
            cx = cx.add(tx);
            cy = cy.add(ty);
            cz = cz.add(tz);
        }

        let k = Q16_16::from_float(tails.len() as f32);
        cx = cx.mul(Q16_16::ONE).div_approx(k);
        cy = cy.mul(Q16_16::ONE).div_approx(k);
        cz = cz.mul(Q16_16::ONE).div_approx(k);

        // Current head position
        let (hx, hy, hz) = node_mem.get_point(head as usize);

        // Direction vector: centroid - head
        let dx = cx.sub(hx);
        let dy = cy.sub(hy);
        let dz = cz.sub(hz);

        // Scale by alpha = 0.5 for half-step
        let alpha = Q16_16::from_float(0.5);
        let delta_x = dx.mul(alpha);
        let delta_y = dy.mul(alpha);
        let delta_z = dz.mul(alpha);

        (head, (delta_x, delta_y, delta_z))
    }
}

/// Point update/apply unit
#[derive(Clone, Debug)]
pub struct ApplyUnit {
    pub apply_calls: u64,
}

impl ApplyUnit {
    pub fn new() -> Self {
        Self { apply_calls: 0 }
    }

    pub fn apply(&mut self, node_mem: &mut NodeMem, node: NodeIx, delta: (Q16_16, Q16_16, Q16_16)) {
        self.apply_calls += 1;

        let (dx, dy, dz) = delta;
        let (mut x, mut y, mut z) = node_mem.get_point(node as usize);

        x = x.add(dx);
        y = y.add(dy);
        z = z.add(dz);

        node_mem.set_point(node as usize, (x, y, z));
    }
}

/// Main toy hardware architecture (GESA-style)
#[derive(Clone, Debug)]
pub struct ToyHw {
    pub node_mem: NodeMem,
    pub edge_mem: EdgeMem,
    pub event_q: EventQueue,
    pub credit_table: CreditTable,
    pub payload_table: PayloadTable,
    pub loci: Vec<Locus>,
    pub pending_topo: PendingTopo,
    pub gate_unit: GateUnit,
    pub geom_unit: GeomUnit,
    pub apply_unit: ApplyUnit,
    pub cycle: u64,
    // Multi-lane tracking
    pub lane_utilization: [u64; LANES],
    pub node_write_conflicts: u64,
    pub edge_read_conflicts: u64,
}

impl ToyHw {
    pub fn new() -> Self {
        Self {
            node_mem: NodeMem::new(),
            edge_mem: EdgeMem::new(),
            event_q: EventQueue::new(),
            credit_table: CreditTable::new(4), // 4 credits per destination
            payload_table: PayloadTable {
                entries: [
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                    PayloadEntry::Float(Q16_16::ZERO),
                ],
            },
            loci: Vec::new(),
            pending_topo: PendingTopo { intents: Vec::new() },
            gate_unit: GateUnit::new(),
            geom_unit: GeomUnit::new(),
            apply_unit: ApplyUnit::new(),
            cycle: 0,
            lane_utilization: [0; LANES],
            node_write_conflicts: 0,
            edge_read_conflicts: 0,
        }
    }

    /// Load a toy instance into hardware memory
    pub fn load_instance(&mut self, inst: &super::toy_instance::ToyInstance) {
        // Load nodes
        for (i, point) in inst.points_xy.iter().enumerate() {
            let (x, y) = (*point).into();
            self.node_mem.set_point(i, (x, y, Q16_16::ZERO));
        }
        self.node_mem.count = inst.points_xy.len();

        // Load edges
        for (i, edge) in inst.edges.iter().enumerate() {
            let radius = Q16_16::from_float(match &edge.footprint {
                super::toy_instance::ToyFootprint::Simplex { radius, .. } => *radius,
                _ => 1.0, // default
            });
            self.edge_mem.set_edge(i, &edge.tails, edge.head, radius);
        }
        self.edge_mem.count = inst.edges.len();

        // Initialize loci (one per node: mix of Field/Topology/Constraint)
        self.loci.clear();
        for i in 0..inst.points_xy.len() {
            let locus_kind = match i % 3 {
                0 => LocusKind::Field,
                1 => LocusKind::Topology,
                2 => LocusKind::Constraint,
                _ => unreachable!(),
            };

            let params = match locus_kind {
                LocusKind::Field => LocusParams::Field {
                    v: Q16_16::ZERO,
                    decay: Q16_16::from_float(0.9), // 0.9 decay
                    bias: Q16_16::from_float(0.01), // small bias
                    threshold: Q16_16::from_float(1.0), // trigger threshold
                },
                LocusKind::Topology => LocusParams::Topology {
                    stress_threshold: Q16_16::from_float(0.5),
                },
                LocusKind::Constraint => LocusParams::Constraint {
                    max_dist: Q16_16::from_float(2.0),
                },
            };

            self.loci.push(Locus {
                kind: locus_kind,
                params,
            });
        }

        // Schedule initial events (GESC style)
        for tick in 0..inst.schedule.ticks {
            for &node in &inst.schedule.dirty_by_tick[tick as usize] {
                // Emit GEOM events for edges involving this node
                for e in 0..self.edge_mem.count {
                    let (tails, _, _) = self.edge_mem.get_edge(e);
                    if tails.contains(&node) {
                        let event = Gescevent::new(
                            e as u32,  // src: edge
                            node as u32, // dst: node
                            GEOM_CLASS,
                            tick as u16,
                            0, // QoS
                            0, // payload_id
                        );

                        if self.credit_table.consume(node as u32) {
                            self.event_q.push(event);
                        }
                    }
                }
            }
        }
    }

    /// Execute one hardware tick (GESC barrier semantics)
    pub fn step_tick(&mut self) {
        self.cycle += 1;

        // Phase 1: Compute (no topology changes)
        self.compute_phase();

        // Phase 2: Structural barrier
        self.apply_structural_barrier();
    }

    /// Compute phase: process GEOM/FIELD events, queue TOPO intents
    fn compute_phase(&mut self) {
        // Process up to some events per tick (toy: process all available)
        while let Some(event) = self.event_q.pop() {
            match event.class() {
                GEOM_CLASS | FIELD_CLASS => {
                    self.process_geom_field_event(event);
                }
                TOPO_CLASS => {
                    self.queue_topo_intent(event);
                }
                _ => {} // Ignore unknown classes
            }

            // Release credit after processing
            self.credit_table.release(event.dst_scope());
        }
    }

    /// Apply structural barrier: resolve and apply TOPO intents deterministically
    fn apply_structural_barrier(&mut self) {
        if self.pending_topo.intents.is_empty() {
            return;
        }

        // Sort intents deterministically (by edge_local_id for toy)
        self.pending_topo.intents.sort_by_key(|intent| intent.edge_local_id);

        // Apply structural changes (toy: simplified - just clear for now)
        // In a real implementation, this would modify the hypergraph
        self.pending_topo.intents.clear();
    }

    /// Process GEOM/FIELD event through loci
    fn process_geom_field_event(&mut self, event: Gescevent) {
        let dst_scope = event.dst_scope() as usize;
        if dst_scope >= self.loci.len() {
            return;
        }

        match self.loci[dst_scope].kind {
            LocusKind::Field => {
                // GLI-style update: v = decay * v + input + bias
                let input = match self.payload_table.entries[event.payload_id() as usize] {
                    PayloadEntry::Float(val) => val,
                    _ => Q16_16::ZERO,
                };

                if let LocusParams::Field { v, decay, bias, threshold } = &mut self.loci[dst_scope].params {
                    *v = decay.mul(*v).add(input).add(*bias);

                    // Threshold trigger (emit event if crosses)
                    if v.0 > threshold.0 {
                        // Emit new event (toy: simplified, just trigger another computation)
                        let current_v = *v;
                        drop(v); // Release the borrow
                        self.trigger_locus_event(dst_scope as u32, current_v);
                    }
                }
            }
            LocusKind::Topology => {
                if let LocusParams::Topology { stress_threshold } = self.loci[dst_scope].params {
                    // Check edge stress and emit TOPO if needed
                    if self.check_edge_stress(event.src_id() as usize, stress_threshold) {
                        self.emit_topo_event(event.src_id(), TopoIntentKind::SplitEdge {
                            edge: event.src_id() as u32
                        });
                    }
                }
            }
            LocusKind::Constraint => {
                if let LocusParams::Constraint { max_dist } = self.loci[dst_scope].params {
                    // Check constraint violation
                    if self.check_constraint_violation(event.src_id() as usize, max_dist) {
                        // Emit corrective event (toy: simplified)
                        self.trigger_locus_event(dst_scope as u32, Q16_16::ZERO);
                    }
                }
            }
        }
    }

    /// Queue TOPO intent from event
    fn queue_topo_intent(&mut self, event: Gescevent) {
        // Decode TOPO intent from payload (toy: simplified mapping)
        let intent = TopoIntent {
            edge_local_id: event.payload_id() as u16,
            kind: TopoIntentKind::SplitEdge { edge: event.src_id() },
        };
        self.pending_topo.intents.push(intent);
    }

    /// Check edge stress for topology locus
    fn check_edge_stress(&mut self, edge: usize, threshold: Q16_16) -> bool {
        // Toy stress metric: edge length > threshold
        let (tails, head, _) = self.edge_mem.get_edge(edge);
        if tails.is_empty() {
            return false;
        }

        // Compute approximate edge "length" as centroid distance
        let mut cx = Q16_16::ZERO;
        let mut cy = Q16_16::ZERO;
        let mut cz = Q16_16::ZERO;

        for &tail in tails {
            let (tx, ty, tz) = self.node_mem.get_point(tail as usize);
            cx = cx.add(tx);
            cy = cy.add(ty);
            cz = cz.add(tz);
        }

        let k = Q16_16::from_float(tails.len() as f32);
        cx = cx.div_approx(k);
        cy = cy.div_approx(k);
        cz = cz.div_approx(k);

        let (hx, hy, hz) = self.node_mem.get_point(head as usize);
        let dx = cx.sub(hx);
        let dy = cy.sub(hy);
        let dz = cz.sub(hz);

        let dist_sq = dx.mul(dx).add(dy.mul(dy)).add(dz.mul(dz));
        dist_sq.0 > threshold.0
    }

    /// Check constraint violation
    fn check_constraint_violation(&mut self, node: usize, max_dist: Q16_16) -> bool {
        // Toy constraint: distance from origin > max_dist
        let (x, y, z) = self.node_mem.get_point(node);
        let dist_sq = x.mul(x).add(y.mul(y)).add(z.mul(z));
        dist_sq.0 > max_dist.0
    }

    /// Trigger locus event emission
    fn trigger_locus_event(&mut self, src_scope: u32, value: Q16_16) {
        // Store value in payload table
        let payload_id = 0; // Fixed for toy
        self.payload_table.entries[payload_id] = PayloadEntry::Float(value);

        // Create and send event (toy: simplified credit check)
        let event = Gescevent::new(
            src_scope,
            src_scope, // Self-trigger for now
            FIELD_CLASS,
            self.cycle as u16,
            0, // QoS
            payload_id as u8,
        );

        if self.credit_table.consume(src_scope) {
            self.event_q.push(event);
        }
    }

    /// Emit TOPO event
    fn emit_topo_event(&mut self, src_scope: u32, kind: TopoIntentKind) {
        let payload_id = 1; // Fixed for TOPO
        let intent = TopoIntent {
            edge_local_id: src_scope as u16,
            kind,
        };
        // Store intent info in payload table (simplified)
        self.payload_table.entries[payload_id] = PayloadEntry::Index(src_scope as u16);

        let event = Gescevent::new(
            src_scope,
            0, // Broadcast TOPO?
            TOPO_CLASS,
            self.cycle as u16,
            0,
            payload_id as u8,
        );

        self.event_q.push(event); // TOPO events don't consume credits
    }

    /// Execute one tick with multi-lane processing and conflict detection
    pub fn step_lanes(&mut self) {
        self.cycle += 1;

        // Collect up to LANES events for parallel processing
        let mut lane_events = Vec::new();
        for _ in 0..LANES {
            if let Some(event) = self.event_q.pop() {
                lane_events.push(event);
            } else {
                break;
            }
        }

        // Track lane utilization
        for i in 0..lane_events.len() {
            self.lane_utilization[i] += 1;
        }

        // Process events in parallel (simulated), but detect and resolve conflicts
        let mut lane_operations = Vec::new();

        for (lane_idx, event) in lane_events.into_iter().enumerate() {
            match event.class() {
                GEOM_CLASS | FIELD_CLASS => {
                    if let Some(op) = self.simulate_lane_operation(lane_idx, event) {
                        lane_operations.push(op);
                    }
                }
                TOPO_CLASS => {
                    self.queue_topo_intent(event);
                }
                _ => {} // Ignore unknown classes
            }

            // Release credit after processing
            self.credit_table.release(event.dst_scope());
        }

        // Detect and resolve memory conflicts between lanes
        self.resolve_lane_conflicts(&mut lane_operations);

        // Apply non-conflicting operations
        for op in lane_operations {
            match op {
                LaneOperation::GeomField { dst_scope, event } => {
                    self.process_geom_field_event(event);
                }
                LaneOperation::Conflicted => {
                    // Already handled in conflict resolution
                }
            }
        }

        // Apply structural barrier
        self.apply_structural_barrier();
    }

    /// Simulate what operation a lane would perform (for conflict detection)
    fn simulate_lane_operation(&mut self, _lane_idx: usize, event: Gescevent) -> Option<LaneOperation> {
        // For now, all GEOM/FIELD events are treated as potentially conflicting
        // In a more sophisticated model, we could analyze which specific memory locations are accessed
        Some(LaneOperation::GeomField {
            dst_scope: event.dst_scope(),
            event,
        })
    }

    /// Detect and resolve conflicts between lane operations
    fn resolve_lane_conflicts(&mut self, operations: &mut Vec<LaneOperation>) {
        use std::collections::HashMap;

        // Group operations by memory access patterns
        let mut node_writes: HashMap<u32, Vec<usize>> = HashMap::new();
        let mut edge_reads: HashMap<u32, Vec<usize>> = HashMap::new();

        for (op_idx, op) in operations.iter().enumerate() {
            match op {
                LaneOperation::GeomField { dst_scope, event } => {
                    // Assume each operation writes to its destination node
                    node_writes.entry(*dst_scope).or_insert(Vec::new()).push(op_idx);

                    // And reads edge information
                    if event.src_id() < self.edge_mem.count as u32 {
                        edge_reads.entry(event.src_id()).or_insert(Vec::new()).push(op_idx);
                    }
                }
                LaneOperation::Conflicted => {
                    // Skip conflicted operations
                }
            }
        }

        // Resolve write conflicts (multiple lanes writing same node)
        for (_node, lane_indices) in node_writes {
            if lane_indices.len() > 1 {
                self.node_write_conflicts += 1;
                // Serialize: keep only first operation, remove others
                for &idx in &lane_indices[1..] {
                    operations[idx] = LaneOperation::Conflicted;
                }
            }
        }

        // Note: Read conflicts are less critical but tracked
        for (_edge, lane_indices) in edge_reads {
            if lane_indices.len() > 1 {
                self.edge_read_conflicts += 1;
                // Reads can proceed in parallel, just count the conflict
            }
        }

        // Remove conflicted operations
        operations.retain(|op| !matches!(op, LaneOperation::Conflicted));
    }

    /// Get hardware counters for RunSummary
    pub fn get_hw_counts(&self) -> HwCounts {
        HwCounts {
            cycles: self.cycle,
            node_mem_reads: self.node_mem.reads,
            node_mem_writes: self.node_mem.writes,
            edge_mem_reads: self.edge_mem.reads,
            event_q_pushes: self.event_q.pushes,
            event_q_pops: self.event_q.pops,
            event_q_overflows: self.event_q.overflows,
            credit_consumes: self.credit_table.credit_consumes,
            credit_releases: self.credit_table.credit_releases,
            credit_denies: self.credit_table.credit_denies,
            pending_topo_events: self.pending_topo.intents.len() as u64,
            lane_utilization: self.lane_utilization,
            node_write_conflicts: self.node_write_conflicts,
            edge_read_conflicts: self.edge_read_conflicts,
        }
    }

    /// Extract final geometry as Point2 array
    pub fn extract_points(&mut self) -> Vec<Point2> {
        (0..self.node_mem.count)
            .map(|i| {
                let (x, y, _) = self.node_mem.get_point(i);
                Point2 {
                    x: x.to_float(),
                    y: y.to_float(),
                }
            })
            .collect()
    }
}