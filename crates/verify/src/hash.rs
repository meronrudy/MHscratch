use manifold::store::ManifoldStore;

// A stable-ish hash for tests.
// - If you later need true bit-level determinism, hash f32::to_bits().
// - This is acceptable for the golden harness skeleton.
pub fn state_hash(m: &ManifoldStore) -> u64 {
    // FNV-1a 64-bit
    let mut h: u64 = 14695981039346656037;
    h = fnv_u64(h, m.epoch);

    for x in &m.point_x {
        h = fnv_u32(h, x.to_bits());
    }
    for y in &m.point_y {
        h = fnv_u32(h, y.to_bits());
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
