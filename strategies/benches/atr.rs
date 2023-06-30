use criterion::{criterion_group, criterion_main, Criterion};
use strategies::atr::{calculate_atr_by_fold, calculate_atr_by_for, calculate_atr_by_ndarray};

fn build_prices() -> Vec<(f64, f64, f64)> {
    let mut a = vec![];
    for i in 0..60 * 60 * 24 * 14 {
        a.push((50.0, 47.0, 50.0));
    }
    a
}

fn bench_calculate_atr(c: &mut Criterion) {
    let data = build_prices();
    let mut group = c.benchmark_group("calculate_atr");

    group.bench_function("calculate_atr_by_for 1000", |b| {
        b.iter(|| calculate_atr_by_for(data.as_slice(), 2))
    });
    group.bench_function("calculate_atr_by_fold 1000", |b| {
        b.iter(|| calculate_atr_by_fold(data.as_slice(), 2))
    });
    group.bench_function("calculate_atr_by_ndarray 1000", |b| {
        b.iter(|| calculate_atr_by_ndarray(data.as_slice(), 2))
    });

    group.finish();
}

criterion_group!(benches, bench_calculate_atr);
criterion_main!(benches);
