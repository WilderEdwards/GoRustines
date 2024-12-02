use tokio::task::{self, JoinHandle};
use tokio::sync::Semaphore;
use std::sync::Arc;

/// An async-based GoroutinePool for lightweight concurrent tasks.
pub struct AsyncGoroutinePool {
    semaphore: Arc<Semaphore>,
    handles: Vec<JoinHandle<()>>,
}

impl AsyncGoroutinePool {
    /// Creates a new AsyncGoroutinePool with a maximum number of concurrent tasks.
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
            handles: Vec::new(),
        }
    }

    /// Spawns an async task in the GoroutinePool.
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

    pub async fn wait_until_complete(&mut self) {
        for handle in self.handles.drain(..) {
            let _ = handle.await;
        }
    }
}
