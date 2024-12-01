
use std::time::{Duration, Instant};

use crate::gorust::{
    adv_go_pool::EnhancedGoroutinePool,
    thread_pool::GoroutinePool,
    tokio_async_pool::AsyncGoroutinePool,
    rust_prim_tools::PrimitivePool,
    rust_prim_tools::Channelish,
};


pub struct BenchmarkResult {
    pub name: String,
    pub total_time: Duration,
    pub tasks_completed: usize,
    pub avg_task_time: Duration,
}

pub fn run_benchmark<F>(name: &str, task_count: usize, f: F) -> BenchmarkResult 
where
    F: FnOnce(usize)
{
    let start = Instant::now();
    f(task_count);
    let total_time = start.elapsed();

    BenchmarkResult {
        name: name.to_string(),
        total_time,
        tasks_completed: task_count,
        avg_task_time: total_time.div_f32(task_count as f32),
    }
}

pub fn run_primitive_pool_benchmark(task_count: usize, thread_count: usize) -> BenchmarkResult {
    run_benchmark("PrimitivePool", task_count, |count| {
        let pool = PrimitivePool::new(thread_count);
        for i in 0..count {
            pool.spawn(move || {
                let mut sum = 0;
                for j in 0..1000 {
                    sum += j;
                }
            });
        }
        std::thread::sleep(Duration::from_secs(1)); // Wait for completion
    })
}

pub fn run_goroutine_pool_benchmark(task_count: usize, thread_count: usize) -> BenchmarkResult {
    run_benchmark("GoroutinePool", task_count, |count| {
        let pool = GoroutinePool::new(thread_count);
        for i in 0..count {
            pool.spawn(move || {
                let mut sum = 0;
                for j in 0..1000 {
                    sum += j;
                }
            });
        }
        pool.wait_until_complete();
    })
}

pub fn run_async_goroutine_pool_benchmark(task_count: usize, thread_count: usize) -> BenchmarkResult {
    run_benchmark("AsyncGoroutinePool", task_count, |count| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let pool = AsyncGoroutinePool::new(thread_count);
            for i in 0..count {
                pool.spawn(async move {
                    let mut sum = 0;
                    for j in 0..1000 {
                        sum += j;
                    }
                });
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        });
    })
}