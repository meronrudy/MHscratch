pub mod store;
pub mod metric;
pub mod geodesic;
pub mod spatial;
pub mod footprint;
pub mod backend;

#[cfg(feature = "perf")]
pub mod simd;

#[cfg(test)]
mod tests;
