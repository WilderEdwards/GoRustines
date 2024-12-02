pub mod gorust {
    pub mod rust_prim_tools;
    pub mod thread_pool;
    pub mod tokio_async_pool;
    pub mod adv_go_pool;
}

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

