mod benchmarks;
mod gorust;

use benchmarks::{
    run_primitive_pool_benchmark,
    run_goroutine_pool_benchmark,
    run_async_goroutine_pool_benchmark,
    BenchmarkResult
};

#[tokio::main]
async fn main() {
    const TASK_COUNT: usize = 1000;
    const THREAD_COUNT: usize = 4;
    
    let mut results = Vec::new();

    // Run benchmarks for different implementations
    results.push(benchmarks::run_primitive_pool_benchmark(TASK_COUNT, THREAD_COUNT));
    results.push(benchmarks::run_goroutine_pool_benchmark(TASK_COUNT, THREAD_COUNT));
    results.push(benchmarks::run_async_goroutine_pool_benchmark(TASK_COUNT, THREAD_COUNT));

    // Print results in a formatted table
    println!("\nBenchmark Results:");
    println!("{:-<80}", "");
    println!("{:<20} | {:<15} | {:<15} | {:<15}", 
        "Implementation", 
        "Total Time (ms)", 
        "Tasks", 
        "Avg Time (µs)");
    println!("{:-<80}", "");

    for result in results {
        println!("{:<20} | {:<15.2} | {:<15} | {:<15.2}",
            result.name,
            result.total_time.as_millis(),
            result.tasks_completed,
            result.avg_task_time.as_micros());
    }
}