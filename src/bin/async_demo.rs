use std::time::{Duration, Instant};
use tokio;
use gorustines::benchmarks::BenchmarkResult;
use gorustines::gorust::tokio_async_pool::AsyncGoroutinePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const TASK_COUNT: usize = 10; // Smaller count for demonstration
    const THREAD_COUNT: usize = 4;
    
    println!("\n🚀 Async Goroutine Pool Demonstration");
    println!("====================================");
    println!("Tasks: {}, Threads: {}\n", TASK_COUNT, THREAD_COUNT);

    let mut pool = AsyncGoroutinePool::new(THREAD_COUNT);
    let start = Instant::now();

    // Spawn tasks with visual feedback
    for i in 0..TASK_COUNT {
        pool.spawn(async move {
            println!("🔄 Task {} starting...", i);
            tokio::time::sleep(Duration::from_millis(500)).await; // Visible delay
            let result = i * 2; // Simple computation
            println!("✅ Task {} completed with result: {}", i, result);
        });
    }

    // Wait for completion
    pool.wait_until_complete().await;
    let duration = start.elapsed();

    println!("\n📊 Demo Results");
    println!("----------------");
    println!("Total Time: {:?}", duration);
    println!("Average Time per Task: {:?}", duration / TASK_COUNT as u32);
    
    Ok(())
}