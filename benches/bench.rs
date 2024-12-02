use criterion::{criterion_group, criterion_main, Criterion};
use gorustines::benchmarks::{run_channel_benchmark, run_goroutine_pool_benchmark,  run_enhanced_goroutine_pool_benchmark};

const TASK_COUNT: usize = 10000;
const THREAD_COUNT: usize = 10;

fn channel_benchmark(c: &mut Criterion) {
    c.bench_function("Channel Benchmark", |b| {
        b.iter(|| run_channel_benchmark(TASK_COUNT, THREAD_COUNT))
    });
}

fn goroutine_pool_benchmark(c: &mut Criterion) {
    c.bench_function("Goroutine Pool Benchmark", |b| {
        b.iter(|| run_goroutine_pool_benchmark(TASK_COUNT, THREAD_COUNT))
    });
}


fn enhanced_goroutine_pool_benchmark(c: &mut Criterion) {
    c.bench_function("Enhanced Goroutine Pool Benchmark", |b| {
        b.iter(|| run_enhanced_goroutine_pool_benchmark(TASK_COUNT, THREAD_COUNT))
    });
}

criterion_group!(
    benches,
    channel_benchmark,
    goroutine_pool_benchmark,
    enhanced_goroutine_pool_benchmark
);
criterion_main!(benches);