pub mod benchmarks;
pub use benchmarks::{
    run_channel_benchmark,
    run_goroutine_pool_benchmark,
    // run_async_goroutine_pool_benchmark,
    run_enhanced_goroutine_pool_benchmark,
    BenchmarkResult,
    ChannelBenchmarkResult,
    PoolBenchmarkResult
};

