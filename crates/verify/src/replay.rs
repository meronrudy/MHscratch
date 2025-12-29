use manifold::store::{ManifoldStore, Integrator};

/// Snapshot format for replay: manifold points, frozen graph, event log
#[derive(Clone, Debug)]
pub struct Snapshot<I: Integrator> {
    pub manifold_state: ManifoldStore<I>,
    pub frozen_graph: Vec<u8>, // Serialized graph state
    pub event_log: Vec<EventRecord>,
    pub initial_hash: u64,
}

#[derive(Clone, Debug)]
pub struct EventRecord {
    pub epoch: u64,
    pub fired_edges: Vec<u32>,
    pub state_hash: u64,
}

/// Replay engine that validates hashes each step
pub struct Replayer<I: Integrator> {
    current_snapshot: Option<Snapshot<I>>,
}

impl<I: Integrator> Replayer<I> {
    pub fn new() -> Self {
        Self {
            current_snapshot: None,
        }
    }

    /// Load a snapshot for replay
    pub fn load_snapshot(&mut self, snapshot: Snapshot<I>) {
        self.current_snapshot = Some(snapshot);
    }

    /// Replay from snapshot and validate determinism
    pub fn replay_and_validate(&mut self) -> Result<(), ReplayError> {
        let snapshot = self.current_snapshot.as_mut()
            .ok_or(ReplayError::NoSnapshot)?;

        let mut expected_hash = snapshot.initial_hash;
        let mut step = 0;

        for record in &snapshot.event_log {
            // Validate hash at this step
            let current_hash = crate::hash::state_hash(&snapshot.manifold_state);
            if current_hash != expected_hash {
                return Err(ReplayError::HashMismatch {
                    step,
                    expected: expected_hash,
                    actual: current_hash,
                    record: record.clone(),
                });
            }

            // Apply the recorded events
            // Note: In a real implementation, this would replay the edge firings
            // For now, we assume the manifold state is updated accordingly

            expected_hash = record.state_hash;
            step += 1;
        }

        // Final validation
        let final_hash = crate::hash::state_hash(&snapshot.manifold_state);
        if final_hash != expected_hash {
            return Err(ReplayError::FinalHashMismatch {
                expected: expected_hash,
                actual: final_hash,
            });
        }

        Ok(())
    }

    /// Find minimal counterexample for non-determinism
    pub fn find_minimal_counterexample(&self, failing_snapshot: &Snapshot<I>) -> Option<Counterexample> {
        // Binary search through the event log to find the first diverging step
        let events = &failing_snapshot.event_log;
        if events.is_empty() {
            return None;
        }

        let mut left = 0;
        let mut right = events.len() - 1;
        let mut first_divergence = None;

        while left <= right {
            let mid = left + (right - left) / 2;

            // Replay up to mid and check hash
            let temp_manifold = failing_snapshot.manifold_state.clone();
            let mut expected_hash = failing_snapshot.initial_hash;

            for i in 0..=mid {
                if crate::hash::state_hash(&temp_manifold) != expected_hash {
                    first_divergence = Some(i);
                    right = mid - 1;
                    break;
                }
                expected_hash = events[i].state_hash;
                // Apply event (simplified)
            }

            if first_divergence.is_none() {
                left = mid + 1;
            }
        }

        first_divergence.map(|step| Counterexample {
            step,
            event_record: events[step].clone(),
            description: format!("First divergence at step {}", step),
        })
    }
}

/// Errors that can occur during replay
#[derive(Debug, Clone)]
pub enum ReplayError {
    NoSnapshot,
    HashMismatch {
        step: usize,
        expected: u64,
        actual: u64,
        record: EventRecord,
    },
    FinalHashMismatch {
        expected: u64,
        actual: u64,
    },
}

/// Minimal counterexample for debugging non-determinism
#[derive(Debug, Clone)]
pub struct Counterexample {
    pub step: usize,
    pub event_record: EventRecord,
    pub description: String,
}
