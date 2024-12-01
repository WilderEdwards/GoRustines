use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;

/// A simple channel-like structure using standard library primitives
pub struct Channelish<T> {
    buffer: Arc<Mutex<VecDeque<T>>>,
    capacity: usize,
    not_full: Arc<Condvar>,
    not_empty: Arc<Condvar>,
}

impl<T> Channelish<T> {
    /// Creates a new "channel" with a specified buffer capacity
    pub fn new(capacity: usize) -> Self {
        Channelish {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
            not_full: Arc::new(Condvar::new()),
            not_empty: Arc::new(Condvar::new()),
        }
    }

    //Creats artificial buffer to 'prevent' deadlock. as well as using std sync primitives to create attributes to check if buffer is full or empty


    /// Send a value, blocking if the channel is full
    pub fn send(&self, value: T) {
        let mut buffer = self.buffer.lock().unwrap();
        
        // Wait while the buffer is full. A spurious wakeup is possible, so we need to check the condition again
        while buffer.len() == self.capacity {
            buffer = self.not_full.wait(buffer).unwrap();
        }
        //primitive push_back method to add value to buffer
        buffer.push_back(value);
        drop(buffer);
        self.not_empty.notify_one();
    }

    /// Method to receive a value, blocking if the channel is empty
    pub fn recv(&self) -> T {
        let mut buffer = self.buffer.lock().unwrap();
        
        // Wait while the buffer is empty
        while buffer.is_empty() {
            buffer = self.not_empty.wait(buffer).unwrap();
        }
        
        let value = buffer.pop_front().unwrap();
        drop(buffer);
        self.not_full.notify_one();
        
        value
    }
}

/// A simple goroutine-like pool using standard library threads
pub struct PrimitivePool {
    threads: Vec<thread::JoinHandle<()>>,
    task_queue: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>,
    queue_condvar: Arc<Condvar>,
}

impl PrimitivePool {
    /// Create a new pool with a specified number of worker threads
    pub fn new(num_workers: usize) -> Self {
        let task_queue = Arc::new(Mutex::new(VecDeque::new()));
        let queue_condvar = Arc::new(Condvar::new());
        let mut threads = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            let task_queue_clone = Arc::clone(&task_queue);
            let condvar_clone = Arc::clone(&queue_condvar);

            let handle = thread::spawn(move || {
                loop {
                    let mut queue = task_queue_clone.lock().unwrap();
                    
                    // Wait for tasks
                    while queue.is_empty() {
                        queue = condvar_clone.wait(queue).unwrap();
                    }

                    // Get and execute task
                    if let Some(task) = queue.pop_front() {
                        drop(queue);
                        task();
                    }
                }
            });

            threads.push(handle);
        }

        PrimitivePool {
            threads,
            task_queue,
            queue_condvar,
        }
    }

    /// Spawn a task in the pool
    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut queue = self.task_queue.lock().unwrap();
        queue.push_back(Box::new(task));
        drop(queue);
        self.queue_condvar.notify_one();
    }
}


//Note: Both implementations, like the others, do not fully accomplish the task of goroutines, specifically in dynamic scheduling, thread creation in the pool, and channel implementation
// specifically, the tokio runtime allows for a more efficient and dynamic scheduling of tasks, as well as the ability to create a pool of threads that can be used to execute tasks concurrently