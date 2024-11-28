use tokio::task;
use tokio::sync::Semaphore;
use std::sync::Arc;

/// An async-based GoroutinePool for lightweight concurrent tasks.
pub struct AsyncGoroutinePool {
    semaphore: Arc<Semaphore>,
}

impl AsyncGoroutinePool {
    /// Creates a new AsyncGoroutinePool with a maximum number of concurrent tasks.
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
        }
    }

    /// Spawns an async task in the GoroutinePool.
    pub fn spawn<F>(&self, task: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let permit = Arc::clone(&self.semaphore);
        task::spawn(async move {
            let _permit = permit.acquire().await.unwrap();
            task.await;
        });
    }
}
