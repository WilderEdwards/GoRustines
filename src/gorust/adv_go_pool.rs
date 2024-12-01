use crossbeam::channel::{bounded, unbounded, Sender, Receiver};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Enhanced Goroutine Pool with advanced features like parallel execution and result collection
// uses crossbeam channels and rayon parallel iterators
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
    pub fn new(pool_size: usize) -> Self {
        let (task_sender, task_receiver) = bounded(pool_size);
        
        Self {
            task_sender,
            task_receiver,
            results: Arc::new(Mutex::new(Vec::new())),
            pool_size,
        }
    }

    /// Spawn a task with result collection
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

    /// Execute tasks in parallel using rayon
    pub fn parallel_execute<F, R>(&self, tasks: Vec<F>) -> Vec<R>
    where 
        F: Fn() -> R + Send + Sync,
        R: Send,
    {
        tasks.into_par_iter()
            .map(|task| task())
            .collect()
    }

    /// Collect and process results
    pub fn collect_results(&self) -> Vec<String> {
        // Wait for tasks to complete
        std::thread::sleep(Duration::from_millis(100));
        
        let results = self.results.lock().unwrap();
        results.clone()
    }

    /// Start worker threads
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

    #[test]
    fn test_enhanced_goroutine_pool() {
        let pool = EnhancedGoroutinePool::new(4);
        pool.start();

        // Spawn individual tasks
        pool.spawn(|| {
            println!("Task 1 completed");
            42
        });

        pool.spawn(|| {
            println!("Task 2 completed");
            "Hello".to_string()
        });

        // Parallel execution example
        let parallel_results: Vec<i32> = pool.parallel_execute(vec![
            || { std::thread::sleep(Duration::from_millis(10)); 1 },
            || { std::thread::sleep(Duration::from_millis(20)); 2 },
            || { std::thread::sleep(Duration::from_millis(30)); 3 },
        ]);

        println!("Parallel results: {:?}", parallel_results);

        // Collect results
        let results = pool.collect_results();
        println!("Collected results: {:?}", results);
    }
}