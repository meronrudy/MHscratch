use manifold::store::{ManifoldStore, Integrator};
use core::ids::{NodeIx, EdgeIx};

/// Feature flags for invariant enforcement
#[cfg(feature = "units-dimensions")]
pub const UNITS_DIMENSIONS_ENABLED: bool = true;

#[cfg(not(feature = "units-dimensions"))]
pub const UNITS_DIMENSIONS_ENABLED: bool = false;

/// Edge signature enforcement is always on
pub const EDGE_SIGNATURE_ENFORCEMENT: bool = true;

/// Invariant checker for verification
pub struct InvariantChecker<I: Integrator> {
    manifold: ManifoldStore<I>,
}

impl<I: Integrator> InvariantChecker<I> {
    pub fn new(manifold: ManifoldStore<I>) -> Self {
        Self { manifold }
    }

    /// Check all invariants - panics in debug mode on first breach
    pub fn check_all_invariants(&self) {
        self.check_manifold_bounds();
        self.check_edge_signatures();
        #[cfg(debug_assertions)]
        self.check_csr_bounds_paranoia();
    }

    /// Check manifold bounds and node constraints
    fn check_manifold_bounds(&self) {
        // Check that all points are finite
        for (i, point) in self.manifold.points.iter().enumerate() {
            if !point.x.is_finite() || !point.y.is_finite() || !point.z.is_finite() {
                self.panic_or_log(&format!("Point {} has non-finite coordinates: {:?}", i, point));
            }
        }

        // Check velocities if present
        if let Some(velocities) = &self.manifold.velocities {
            for (i, vel) in velocities.iter().enumerate() {
                if !vel.x.is_finite() || !vel.y.is_finite() || !vel.z.is_finite() {
                    self.panic_or_log(&format!("Velocity {} has non-finite components: {:?}", i, vel));
                }
            }
            // Ensure points and velocities have same length
            if velocities.len() != self.manifold.points.len() {
                self.panic_or_log(&format!("Points ({}) and velocities ({}) length mismatch",
                    self.manifold.points.len(), velocities.len()));
            }
        }
    }

    /// Edge signature enforcement - always enabled
    fn check_edge_signatures(&self) {
        // This would validate that edges have correct input/output signatures
        // For now, this is a placeholder - actual implementation would need graph access
        // In a real system, this would check that edges consume/produce correct types
    }

    /// Debug-only paranoia checks for CSR bounds
    #[cfg(debug_assertions)]
    fn check_csr_bounds_paranoia(&self) {
        // In debug mode, perform expensive checks for CSR matrix bounds
        // This is "paranoia" level checking that's too slow for release
        // For now, this is a placeholder - actual CSR bounds checking would go here
    }

    /// Release mode: no-op for CSR bounds (no runtime tax)
    #[cfg(not(debug_assertions))]
    fn check_csr_bounds_paranoia(&self) {
        // Release mode: no runtime cost
    }

    /// Handle invariant breach: panic in debug, log in release
    #[cfg(debug_assertions)]
    fn panic_or_log(&self, message: &str) {
        panic!("Invariant breach: {}", message);
    }

    /// Handle invariant breach: panic in debug, log in release
    #[cfg(not(debug_assertions))]
    fn panic_or_log(&self, message: &str) {
        // In release mode, could log to a verification log, but for now just ignore
        // The key point is no panic in release mode
        eprintln!("Invariant breach (release mode): {}", message);
    }
}

/// Dimension/unit checking (feature-gated)
#[cfg(feature = "units-dimensions")]
pub mod units_dimensions {
    use super::*;

    /// Check dimensional consistency
    pub fn check_dimensions(node: NodeIx, expected_dims: &[&str], actual_dims: &[&str]) {
        if expected_dims != actual_dims {
            panic!("Dimension mismatch for node {}: expected {:?}, got {:?}", node, expected_dims, actual_dims);
        }
    }

    /// Check unit consistency
    pub fn check_units(edge: EdgeIx, input_units: &[&str], output_units: &[&str]) {
        // Simplified unit checking - real implementation would be more sophisticated
        if input_units.len() != output_units.len() {
            panic!("Unit arity mismatch for edge {}: input {:?}, output {:?}", edge, input_units, output_units);
        }
    }
}

#[cfg(not(feature = "units-dimensions"))]
pub mod units_dimensions {
    use super::*;

    /// No-op in release without units-dimensions feature
    pub fn check_dimensions(_node: NodeIx, _expected_dims: &[&str], _actual_dims: &[&str]) {}

    /// No-op in release without units-dimensions feature
    pub fn check_units(_edge: EdgeIx, _input_units: &[&str], _output_units: &[&str]) {}
}