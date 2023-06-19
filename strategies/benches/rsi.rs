use criterion::{Criterion, criterion_group, criterion_main};
use ndarray::{Array, array, Array1};
use strategies::rsi::{calculate_rsi, calculate_rsi_by_for, calculate_rsi_by_ndarray, calculate_rsi_by_rayon, calculate_rsi_by_rayon_and_ndarray};


fn build_prices()->Array1<f64>{
    array![46.125, 47.125, 46.4375, 46.9375, 44.9375, 44.25, 44.625, 45.75, 47.8125, 47.5625, 47.0, 44.5625, 46.3125, 47.6875, 46.6875, 45.6875, 43.0625]
}

fn build_prices1()->Vec<f64>{
    let mut a = vec![];
    for i in 0..60 * 60 * 24 * 14 {
        a.push(10.0);
    }
    a
    // vec![46.125, 47.125, 46.4375, 46.9375, 44.9375, 44.25, 44.625, 45.75, 47.8125, 47.5625, 47.0, 44.5625, 46.3125, 47.6875, 46.6875, 45.6875, 43.0625]
}

fn bench_calculate_rsi(c: &mut Criterion) {
    let data = build_prices1();
    let mut group = c.benchmark_group("calculate_rsi");
    group.bench_function("calculate_rsi_by_rayon 1000", |b| {
        b.iter(|| calculate_rsi_by_rayon(&data, 130000))
    });
    group.bench_function("calculate_rsi_by_for 1000", |b| {
        b.iter(|| calculate_rsi_by_for(&data, 130000))
    });
    group.bench_function("calculate_rsi_by_ndarray 1000", |b| {
        b.iter(|| calculate_rsi_by_ndarray(&data, 130000))
    });
    group.bench_function("calculate_rsi_by_rayon_and_ndarray 1000", |b| {
        b.iter(|| calculate_rsi_by_rayon_and_ndarray(&data, 130000))
    });
    group.finish();
}


criterion_group!(benches, bench_calculate_rsi);
criterion_main!(benches);