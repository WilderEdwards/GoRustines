use std::time::{Duration, Instant};
use std::sync::Arc;
use std::thread;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

use sysinfo::{System, CpuRefreshKind, Cpu};

const TASK_DELAY: u64 = 10; // constant for delay in execution, adjust for different demonstrations

#[derive(Debug)]
pub enum BenchmarkResult {
    Channel(ChannelBenchmarkResult),
    Pool(PoolBenchmarkResult),
    System(SystemMetrics), // New variant
}

#[derive(Debug)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub duration: Duration,
}

// This function is used to capture CPU and memory usage metrics for every operation. It is ran during the program, capturing 
// metrics before and after each operation and storing them to be displayed
fn measure_system_metrics<F>(f: F) -> (Duration, SystemMetrics) 
where 
    F: FnOnce() -> Duration 
{
    let mut sys = System::new();
    let mut total_cpu_usage = 0.0;
    let mut cpu_samples = 0;
    
    // Initial refresh and measurements
    sys.refresh_all();
    let start_memory = sys.used_memory();
    
    // Get baseline CPU usage
    thread::sleep(Duration::from_millis(200)); // Allow system to stabilize
    sys.refresh_all();
    total_cpu_usage += sys.global_cpu_usage();
    cpu_samples += 1;
    
    let duration = f();
    
    // Sample CPU less frequently with longer intervals
    for _ in 0..3 {  // Reduced sample count
        thread::sleep(Duration::from_millis(250)); // Increased interval
        sys.refresh_all();
        total_cpu_usage += sys.global_cpu_usage();
        cpu_samples += 1;
    }
    
    // Final measurements
    sys.refresh_all();
    let end_memory = sys.used_memory();
    
    let avg_cpu_usage = (total_cpu_usage / cpu_samples as f32).min(100.0); // Cap at 100%

    (duration, SystemMetrics {
        cpu_usage: avg_cpu_usage,
        memory_used: end_memory - start_memory,
        duration,
    })
}

use crate::gorust::{
    rust_prim_tools::Channelish,
    thread_pool::GoroutinePool,
    tokio_async_pool::AsyncGoroutinePool,
    adv_go_pool::EnhancedGoroutinePool,
};


#[derive(Debug)]
pub struct ChannelBenchmarkResult {
    pub name: String,
    pub send_time: Duration,
    pub recv_time: Duration,
    pub messages_processed: usize,
    pub throughput: f64,
}

#[derive(Debug)]
pub struct PoolBenchmarkResult {
    pub name: String,
    pub total_time: Duration,
    pub tasks_completed: usize,
    pub avg_task_time: Duration,
}



//a function that runs a channel benchmark with a sender and receiver thread.
//The sender thread sends a message to the receiver thread, which then prints a message to the console.

pub fn run_channel_benchmark(message_count: usize, buffer_size: usize) -> BenchmarkResult {
    let channel: Channelish<usize> = Channelish::new(buffer_size);
    let channel = Arc::new(channel);
    let channel_clone = channel.clone();

    let start_send = Instant::now();
    
    // Sender thread
    let sender = thread::spawn(move || {
        for i in 0..message_count {
            channel_clone.send(i as usize);
        }
    });

    // Receiver thread
    let start_recv = Instant::now();
    let receiver = thread::spawn(move || {
        for _ in 0..message_count {
            let _ = channel.recv();
            println!("Received message");
        }
    });

    sender.join().unwrap();
    receiver.join().unwrap();

    let send_time = start_send.elapsed();
    let recv_time = start_recv.elapsed();
    let total_time = send_time.max(recv_time);
    let throughput = message_count as f64 / total_time.as_secs_f64();

    BenchmarkResult::Channel(ChannelBenchmarkResult {
        name: "Channelish".to_string(),
        send_time,
        recv_time,
        messages_processed: message_count,
        throughput,
    })
}

pub fn run_goroutine_pool_benchmark(task_count: usize, thread_count: usize) -> Vec<BenchmarkResult> {
    let (duration, metrics) = measure_system_metrics(|| {
        let start = Instant::now();
        let pool = GoroutinePool::new(thread_count);
        
        for i in 0..task_count {
            pool.spawn(move || {
                println!(" Basic Pool - Task {} starting...", i);
                thread::sleep(Duration::from_millis(TASK_DELAY));
                let mut sum = 0;
                for j in 0..1000 {
                    sum += j;
                }
                println!(" Basic Pool - Task {} completed (sum: {})", i, sum);
            });
        }
        pool.wait_until_complete();
        start.elapsed()
    });

    vec![
        BenchmarkResult::Pool(PoolBenchmarkResult {
            name: "GoroutinePool".to_string(),
            total_time: duration,
            tasks_completed: task_count,
            avg_task_time: duration / task_count as u32,
        }),
        BenchmarkResult::System(metrics)
    ]
}



pub fn run_enhanced_goroutine_pool_benchmark(task_count: usize, thread_count: usize) -> Vec<BenchmarkResult> {
    let completed_count = Arc::new(AtomicUsize::new(0));
    let (duration, metrics) = measure_system_metrics(|| {
        let start = Instant::now();
        let pool = EnhancedGoroutinePool::new(thread_count);
        
        // Spawn tasks
        for _ in 0..task_count {
            let completed = Arc::clone(&completed_count);
            pool.spawn(move || {
                let mut sum = 0;
                for j in 0..1000 {
                    sum += j;
                }
                completed.fetch_add(1, Ordering::SeqCst);
                sum.to_string()
            });
        }

        // Wait with timeout
        const TIMEOUT_SECS: u64 = 5;
        const CHECK_INTERVAL_MS: u64 = 100;
        let deadline = Instant::now() + Duration::from_secs(TIMEOUT_SECS);
        
        let mut tasks_completed = 0;
        while Instant::now() < deadline {
            tasks_completed = completed_count.load(Ordering::SeqCst);
            if tasks_completed >= task_count {
                break;
            }
            thread::sleep(Duration::from_millis(CHECK_INTERVAL_MS));
        }

        // Get available results
        let results = pool.collect_results();
        let elapsed = start.elapsed();

        // Update actual completion count
        tasks_completed = completed_count.load(Ordering::SeqCst);
        if tasks_completed < task_count {
            println!("Timeout: Only {}/{} tasks completed", tasks_completed, task_count);
        }

        elapsed
    });

    vec![
        BenchmarkResult::Pool(PoolBenchmarkResult {
            name: "EnhancedGoroutinePool".to_string(),
            total_time: duration,
            tasks_completed: completed_count.load(Ordering::SeqCst), // Use actual count
            avg_task_time: duration / completed_count.load(Ordering::SeqCst) as u32,
        }),
        BenchmarkResult::System(metrics)
    ]
}
