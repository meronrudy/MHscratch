use manifold::store::{ManifoldStore, Integrator};

// A stable-ish hash for tests.
// - If you later need true bit-level determinism, hash f32::to_bits().
// - This is acceptable for the golden harness skeleton.
pub fn state_hash<I: Integrator>(m: &ManifoldStore<I>) -> u64 {
    // FNV-1a 64-bit
    let mut h: u64 = 14695981039346656037;
    h = fnv_u64(h, m.epoch);

    for p in &m.points {
        h = fnv_u32(h, p.x.to_bits());
        h = fnv_u32(h, p.y.to_bits());
        h = fnv_u32(h, p.z.to_bits());
    }
    h
}

#[inline]
fn fnv_u64(mut h: u64, x: u64) -> u64 {
    h ^= x;
    h = h.wrapping_mul(1099511628211);
    h
}
#[inline]
fn fnv_u32(mut h: u64, x: u32) -> u64 {
    h ^= x as u64;
    h = h.wrapping_mul(1099511628211);
    h
}
