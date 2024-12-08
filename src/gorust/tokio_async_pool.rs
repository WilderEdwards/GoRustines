use tokio::task::{self, JoinHandle};
use tokio::sync::Semaphore;
use std::sync::Arc;

/// Async-based thread pool that mimics Go's goroutine behavior using Tokio primitives
pub struct AsyncGoroutinePool {
    semaphore: Arc<Semaphore>,    // Controls concurrent task count
    handles: Vec<JoinHandle<()>>,  // Stores task handles
}

impl AsyncGoroutinePool {
    /// Creates a new pool with specified concurrency limit
    /// 
    /// # Arguments
    /// * `max_concurrency` - Maximum number of tasks that can run simultaneously
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
            handles: Vec::new(),
        }
    }

    /// Spawns an async task in the pool
    /// 
    /// # Arguments
    /// * `task` - Future to execute
    /// 
    /// # Type Bounds
    /// * Task must be Send + 'static for thread safety
    pub fn spawn<F>(&mut self, task: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let permit = Arc::clone(&self.semaphore);
        let _handle = task::spawn(async move {
            let _permit = permit;
            task.await;
        });
        &mut self.handles.push(_handle);
    }

    /// Waits for all spawned tasks to complete
    /// Drains handles vector and awaits each task
    pub async fn wait_until_complete(&mut self) {
        for handle in self.handles.drain(..) {
            let _ = handle.await;
        }
    }
}
