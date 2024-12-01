use threadpool::ThreadPool;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A thread-based GoroutinePool for managing concurrent tasks.
pub struct GoroutinePool {
    pool: ThreadPool,
    active_tasks: Arc<AtomicUsize>,
}

impl GoroutinePool {
    /// Creates a new thread-based GoroutinePool with a fixed number of threads.
    pub fn new(size: usize) -> Self {
        Self {
            pool: ThreadPool::new(size),
            active_tasks: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Spawns a task to run in the GoroutinePool.
    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let active_tasks = Arc::clone(&self.active_tasks);
        active_tasks.fetch_add(1, Ordering::SeqCst);

        self.pool.execute(move || {
            task();
            active_tasks.fetch_sub(1, Ordering::SeqCst);
        });
    }

    /// Returns the number of active tasks currently running.
    pub fn active_count(&self) -> usize {
        self.active_tasks.load(Ordering::SeqCst)
    }

    /// Waits until all tasks are completed (useful for graceful shutdown).
    pub fn wait_until_complete(&self) {
        while self.active_count() > 0 {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
