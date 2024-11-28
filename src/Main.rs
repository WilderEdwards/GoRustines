mod goroutine_pool;

use goroutine_pool::thread_pool::GoroutinePool;
use goroutine_pool::async_pool::AsyncGoroutinePool;

#[tokio::main]
async fn main() {
    // Thread-based GoroutinePool
    let thread_pool = GoroutinePool::new(4);
    for i in 0..10 {
        thread_pool.spawn(move || {
            println!("Thread-based task {} is running!", i);
            std::thread::sleep(std::time::Duration::from_millis(100));
            println!("Thread-based task {} is done!", i);
        });
    }
    thread_pool.wait_until_complete();
    println!("All thread-based tasks completed.");

    // Async-based GoroutinePool
    let async_pool = AsyncGoroutinePool::new(4);
    for i in 0..10 {
        async_pool.spawn(async move {
            println!("Async task {} is running!", i);
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            println!("Async task {} is done!", i);
        });
    }
    tokio::time::sleep(std::time::Duration::from_secs(1)).await; // Wait for async tasks
    println!("All async-based tasks completed.");
}