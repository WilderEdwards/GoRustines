use crossbeam::channel::{bounded, Sender, Receiver};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::sync::atomic::AtomicBool;

// Enhanced Goroutine Pool with advanced features like parallel execution and result collection
// uses crossbeam channels and rayon parallel iterators

// Example usage:
// 
// let pool = EnhancedGoroutinePool::new(4); // Create 4-thread pool
// pool.spawn(|| "task complete".to_string()); // Submit task
// let results = pool.collect_results(); // Get results

pub struct EnhancedGoroutinePool {
    // Channel for task distribution
    task_sender: Sender<Box<dyn FnOnce() + Send>>,
    task_receiver: Receiver<Box<dyn FnOnce() + Send>>,
    
    // Results collection mechanism
    results: Arc<Mutex<Vec<String>>>,
    
    // Configurable pool size
    pool_size: usize,
}

impl EnhancedGoroutinePool {
    /// Create a new enhanced goroutine pool
    /// NOTE: BOUND channels for safety, expereiment with unbound channels for performance
    pub fn new(pool_size: usize) -> Self {
        let (task_sender, task_receiver) = bounded(pool_size);
        
        Self {
            task_sender,
            task_receiver,
            results: Arc::new(Mutex::new(Vec::new())),
            pool_size,
        }
    }

     /// Submits task to pool for async execution
    /// Task must be Send + 'static for thread safety
    /// Results stored in thread-safe collection
    pub fn spawn<F, R>(&self, task: F)
    where 
        F: FnOnce() -> R + Send + 'static,
        R: ToString + Send + 'static,
    {
        let results = Arc::clone(&self.results);
        
        let wrapped_task = move || {
            let result = task();
            let mut results_guard = results.lock().unwrap();
            results_guard.push(result.to_string());
        };
        
        // Non-blocking send
        self.task_sender.send(Box::new(wrapped_task)).unwrap();
    }

    /// Parallel execution of multiple tasks
    /// Optimal for CPU-intensive parallel workloads (thanks rayon!)
    /// Go-like goroutine parallelism
    pub fn parallel_execute<F, R>(&self, tasks: Vec<F>) -> Vec<R>
    where 
        F: Fn() -> R + Send + Sync,
        R: Send,
    {
        tasks.into_par_iter()
            .map(|task| task())
            .collect()
    }

    /// Retrieves results from completed tasks
    /// Returns cloned result vector to prevent blocking on thread
    /// Use this method to collect results after submitting tasks
    pub fn collect_results(&self) -> Vec<String> {
        // Wait for tasks to complete
        std::thread::sleep(Duration::from_millis(100));
        
        let results = self.results.lock().unwrap();
        results.clone()
    }

    /// Start worker threads
    /// Each thread listens for tasks on channel, similair to goroutines. 
    /// data races aren't possible but deadlock can occur if the pool is full
    pub fn start(&self) {
        let receiver = self.task_receiver.clone();
        
        for _ in 0..self.pool_size {
            let rx = receiver.clone();
            
            std::thread::spawn(move || {
                while let Ok(task) = rx.recv() {
                    task();
                }
            });
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

#[test]
fn test_pool_creation() {
    let pool = EnhancedGoroutinePool::new(4);
    assert!(pool.task_sender.is_empty());
    assert!(pool.results.lock().unwrap().is_empty());
    println!("Pool created successfully");
    }


#[test]
fn test_spawn_single_task() {
    // Create pool and initialize workers
    let pool = EnhancedGoroutinePool::new(2);
    
    // Setup worker status tracking
    let worker_started = Arc::new(AtomicBool::new(false));
    let worker_started_clone = Arc::clone(&worker_started);
    
    // Setup task completion tracking
    let completed = Arc::new(AtomicBool::new(false));
    let completed_clone = Arc::clone(&completed);
    
    // Wait for worker initialization
    std::thread::sleep(Duration::from_millis(100));
    println!("Workers initialized");
    
    // Spawn task with worker status check
    println!("Spawning task...");
    pool.spawn(move || {
        worker_started_clone.store(true, Ordering::SeqCst);
        println!("Task executing in worker thread");

        let result = "test".to_string();
        completed_clone.store(true, Ordering::SeqCst);
        println!("Task completed");
        result
    });

    // Wait for worker to start
    let mut worker_timeout = std::time::Instant::now();
    while !worker_started.load(Ordering::SeqCst) {

        if worker_timeout.elapsed() > Duration::from_secs(10) {
            panic!("Worker thread failed to start");

        }
        std::thread::sleep(Duration::from_millis(10));
    }
    println!("Worker thread started");

    // Wait for task completion
    let task_timeout = std::time::Instant::now();
    while !completed.load(Ordering::SeqCst) {
        if task_timeout.elapsed() > Duration::from_secs(2) {
            panic!("Task execution timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    println!("Task completion confirmed");

    // Verify results
    let results = pool.collect_results();
    assert_eq!(results.len(), 1, "Expected 1 result, got {}", results.len());
    assert_eq!(results[0], "test", "Expected 'test', got '{}'", results[0]);
}

#[test]
fn test_spawn_multiple_tasks() {
        let pool = EnhancedGoroutinePool::new(4);
        let counter = Arc::new(AtomicUsize::new(0));
        
        for _ in 0..5 {
            let counter = Arc::clone(&counter);
            pool.spawn(move || {
                counter.fetch_add(1, Ordering::SeqCst);
                println!("Task completed");
                "done".to_string()
            });
        }
        
        std::thread::sleep(Duration::from_millis(100));
        let results = pool.collect_results();
        
        assert_eq!(results.len(), 5);
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

#[test]
fn test_parallel_execute() {
        let pool = EnhancedGoroutinePool::new(4);
        let tasks: Vec<Box<dyn Fn() -> i32 + Send + Sync>> = vec![
            Box::new(|| 1),
            Box::new(|| 2),
            Box::new(|| 3),
        ];
        
        let results = pool.parallel_execute(tasks);
        assert_eq!(results.len(), 3);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        assert!(results.contains(&3));
    }

#[test]
fn test_collect_results_ordering() {
        let pool = EnhancedGoroutinePool::new(2);
        
        for i in 0..3 {
            pool.spawn(move || i.to_string());
        }
        
        std::thread::sleep(Duration::from_millis(50));
        let results = pool.collect_results();
        
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.parse::<i32>().is_ok()));
    }

    #[test]
    fn test_high_work() {
        let pool = EnhancedGoroutinePool::new(8);
        let task_count = 100;
        let counter = Arc::new(AtomicUsize::new(0));
        
        for _ in 0..task_count {
            let counter = Arc::clone(&counter);
            pool.spawn(move || {
                std::thread::sleep(Duration::from_millis(1));
                counter.fetch_add(1, Ordering::SeqCst);
                "done".to_string()
            });
        }
        
        std::thread::sleep(Duration::from_millis(200));
        let results = pool.collect_results();
        println!("Results: {:?}", results);
        
        assert_eq!(results.len(), task_count);
        assert_eq!(counter.load(Ordering::SeqCst), task_count);
    }
}