use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nalgebra::{Point3, Vector3};
use rand::prelude::*;
use manifold::store::{ManifoldStore, ExplicitEuler, Point, Vecf};
use manifold::spatial::brute_force::BruteForceIndex;
use manifold::geodesic::{exp, log, ManifoldType};

#[cfg(feature = "perf")]
use manifold::simd::{dist_simd, SoAPoints};

fn generate_random_points(n: usize) -> Vec<Point> {
    let mut rng = thread_rng();
    (0..n)
        .map(|_| Point::new(rng.gen::<f32>() * 100.0, rng.gen::<f32>() * 100.0, rng.gen::<f32>() * 100.0))
        .collect()
}

fn bench_distance_computation(c: &mut Criterion) {
    let points = generate_random_points(1000);
    let center = Point::new(50.0, 50.0, 50.0);
    let radius = 25.0;

    let mut group = c.benchmark_group("distance");

    // Benchmark scalar distance
    group.bench_function("scalar_distance", |b| {
        b.iter(|| {
            let mut count = 0;
            for point in &points {
                let dist = nalgebra::distance(point, &center);
                if dist <= radius {
                    count += 1;
                }
            }
            black_box(count)
        })
    });

    // Benchmark SIMD distance (only when perf feature is enabled)
    #[cfg(feature = "perf")]
    group.bench_function("simd_distance", |b| {
        b.iter(|| {
            let mut count = 0;
            for point in &points {
                let dist = dist_simd(point, &center);
                if dist <= radius {
                    count += 1;
                }
            }
            black_box(count)
        })
    });

    group.finish();
}

fn bench_spatial_query(c: &mut Criterion) {
    let points = generate_random_points(10000);
    let mut index = BruteForceIndex::new();
    index.rebuild(&points, points.len());

    let center = Point::new(50.0, 50.0, 50.0);
    let radius = 25.0;

    let mut results = Vec::new();

    c.bench_function("spatial_query", |b| {
        b.iter(|| {
            index.radius_query(&center, radius, &mut results);
            black_box(results.len())
        })
    });
}

fn bench_integration(c: &mut Criterion) {
    let points = generate_random_points(1000);
    let velocities: Vec<Vecf> = (0..1000)
        .map(|_| Vecf::new(1.0, 0.5, -0.5))
        .collect();

    let mut group = c.benchmark_group("integration");

    // Benchmark AoS integration
    #[cfg(not(feature = "perf"))]
    group.bench_function("aos_integration", |b| {
        let mut store = ManifoldStore::new(ExplicitEuler);
        store.points = points.clone();
        store.velocities = Some(velocities.clone());

        b.iter(|| {
            store.integrate(0.01);
            black_box(&store.points)
        })
    });

    // Benchmark SoA integration
    #[cfg(feature = "perf")]
    group.bench_function("soa_integration", |b| {
        let mut store = ManifoldStore::new(ExplicitEuler);
        store.points = SoAPoints::from_aos(&points);
        store.velocities = Some(velocities.clone());

        b.iter(|| {
            store.integrate(0.01);
            black_box(&store.points)
        })
    });

    group.finish();
}

fn bench_geodesic_operations(c: &mut Criterion) {
    let base = Point::new(1.0, 0.0, 0.0);
    let target = Point::new(0.0, 1.0, 0.0);
    let tangent = Vecf::new(0.1, 0.1, 0.0);

    let mut group = c.benchmark_group("geodesic");

    group.bench_function("euclidean_exp", |b| {
        b.iter(|| black_box(exp(ManifoldType::Euclidean, &base, &tangent)))
    });

    group.bench_function("euclidean_log", |b| {
        b.iter(|| black_box(log(ManifoldType::Euclidean, &base, &target)))
    });

    group.bench_function("spherical_exp", |b| {
        b.iter(|| black_box(exp(ManifoldType::Spherical, &base, &tangent)))
    });

    group.bench_function("spherical_log", |b| {
        b.iter(|| black_box(log(ManifoldType::Spherical, &base, &target)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_distance_computation,
    bench_spatial_query,
    bench_integration,
    bench_geodesic_operations
);
criterion_main!(benches);