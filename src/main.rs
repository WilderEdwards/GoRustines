mod gorust;
mod benchmarks;

use std::time::Duration;
use crate::benchmarks::{
    run_channel_benchmark,
    run_goroutine_pool_benchmark, 
    // run_async_goroutine_pool_benchmark,
    run_enhanced_goroutine_pool_benchmark,
    BenchmarkResult
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const TASK_COUNT: usize = 100;
    const THREAD_COUNT: usize = 10;
    
    // Collect benchmark results
    let channel_results = vec![
        run_channel_benchmark(TASK_COUNT, THREAD_COUNT)
    ];

    let pool_results: Vec<BenchmarkResult> = vec![
        run_goroutine_pool_benchmark(TASK_COUNT, THREAD_COUNT),
        //run_enhanced_goroutine_pool_benchmark(TASK_COUNT, THREAD_COUNT),
    ].into_iter().flatten().collect();

    // Display results
    print_channel_results(&channel_results);
    print_pool_results(&pool_results);
    print_system_metrics(&pool_results);
    Ok(())
}

fn print_channel_results(results: &[BenchmarkResult]) {
    println!("\nChannel Benchmark Results:");
    println!("{:-<80}", "");
    println!("{:<20} | {:<15} | {:<15} | {:<15}", 
        "Implementation", 
        "Send Time (ms)",
        "Recv Time (ms)", 
        "Throughput (msg/s)");
    println!("{:-<80}", "");

    for result in results {
        if let BenchmarkResult::Channel(result) = result {
            println!("{:<20} | {:<15.2} | {:<15.2} | {:<15.2}",
                result.name,
                result.send_time.as_millis(),
                result.recv_time.as_millis(),
                result.throughput);
        }
    }
}

fn print_pool_results(results: &[BenchmarkResult]) {
    println!("\nThread Pool Benchmark Results:");
    println!("{:-<80}", "");
    println!("{:<20} | {:<15} | {:<15} | {:<15}", 
        "Implementation", 
        "Total Time (ms)", 
        "Tasks", 
        "Avg Time (micro sec)");
    println!("{:-<80}", "");

    for result in results {
        if let BenchmarkResult::Pool(result) = result {
            println!("{:<20} | {:<15.2} | {:<15} | {:<15.2}",
                result.name,
                result.total_time.as_millis(),
                result.tasks_completed,
                result.avg_task_time.as_micros());
        }
    }
}

fn print_system_metrics(results: &[BenchmarkResult]) {
    println!("\nSystem Metrics:");
    println!("{:-<80}", "");
    println!("{:<20} | {:<15} | {:<15} | {:<15}", 
        "Implementation", 
        "CPU Usage (%)",
        "Memory (MB)", 
        "Duration (ms)");
    println!("{:-<80}", "");

    for result in results {
        if let BenchmarkResult::System(metrics) = result {
            println!("{:<20} | {:<15.2} | {:<15.2} | {:<15.2}",
                "System Metrics",
                metrics.cpu_usage,
                metrics.memory_used as f64 / 1024.0 / 1024.0,
                metrics.duration.as_millis());
        }
    }
}