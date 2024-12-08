use std::time::{Duration, Instant};
use tokio;
use gorustines::gorust::tokio_async_pool::AsyncGoroutinePool;


/// Async demo showing concurrent task execution with AsyncGoroutinePool
/// 
/// # Program Flow:
/// 1. Creates thread pool with THREAD_COUNT worker threads
/// 2. Spawns TASK_COUNT async tasks that:
///    - Print start message
///    - Sleep for 500ms to simulate work 
///    - Compute(i * 2)
///    - Print completion message
/// 3. Waits for all tasks to finish
/// 4. Displays timing metrics

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const TASK_COUNT: usize = 10000; // Smaller count for demonstration
    const THREAD_COUNT: usize = 16;
    
    println!("\n Async Goroutine Pool Demonstration");
    println!("====================================");
    println!("Tasks: {}, Threads: {}\n", TASK_COUNT, THREAD_COUNT);

    let mut pool = AsyncGoroutinePool::new(THREAD_COUNT);
    let start = Instant::now();

    // Spawn tasks with visual feedback
    for i in 0..TASK_COUNT {
        pool.spawn(async move {
            println!(" Task {} starting...", i);
            tokio::time::sleep(Duration::from_millis(500)).await; // Visible delay
            let result = i * 2; // Simple computation
            println!(" Task {} completed with result: {}", i, result);
        });
    }

    // Wait for completion
    pool.wait_until_complete().await;
    let duration = start.elapsed();

    println!("\n Demo Results");
    println!("----------------");
    println!("Total Time: {:?}", duration);
    println!("Average Time per Task: {:?}", duration / TASK_COUNT as u32);
    
    Ok(())
}