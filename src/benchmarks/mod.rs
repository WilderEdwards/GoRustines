pub mod benchmarks;
pub use benchmarks::{
    run_primitive_pool_benchmark,
    run_goroutine_pool_benchmark,
    run_async_goroutine_pool_benchmark,
    BenchmarkResult
};

