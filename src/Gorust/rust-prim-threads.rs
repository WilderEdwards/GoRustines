use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;

/// A simple channel-like structure using standard library primitives
pub struct StandardChannel<T> {
    buffer: Arc<Mutex<VecDeque<T>>>,
    capacity: usize,
    not_full: Arc<Condvar>,
    not_empty: Arc<Condvar>,
}

impl<T> StandardChannel<T> {
    /// Create a new channel with a specified buffer capacity
    pub fn new(capacity: usize) -> Self {
        StandardChannel {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
            not_full: Arc::new(Condvar::new()),
            not_empty: Arc::new(Condvar::new()),
        }
    }

    /// Send a value, blocking if the channel is full
    pub fn send(&self, value: T) {
        let mut buffer = self.buffer.lock().unwrap();
        
        // Wait while the buffer is full
        while buffer.len() == self.capacity {
            buffer = self.not_full.wait(buffer).unwrap();
        }
        
        buffer.push_back(value);
        drop(buffer);
        self.not_empty.notify_one();
    }

    /// Receive a value, blocking if the channel is empty
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
pub struct StandardGoroutinePool {
    threads: Vec<thread::JoinHandle<()>>,
    task_queue: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>,
    queue_condvar: Arc<Condvar>,
}

impl StandardGoroutinePool {
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

        StandardGoroutinePool {
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

fn main() {
    // Demonstrate channel usage
    let channel = StandardChannel::new(10);
    
    // Producer thread
    let channel_send = channel.clone();
    let producer = thread::spawn(move || {
        for i in 0..20 {
            channel_send.send(i);
            println!("Sent: {}", i);
            thread::sleep(Duration::from_millis(50));
        }
    });

    // Create a consumer thread
    let channel_recv = channel.clone();
    let consumer = thread::spawn(move || {
        for _ in 0..20 {
            let value = channel_recv.recv();
            println!("Received: {}", value);
        }
    });

    // Demonstratation of the 'goroutine' pool.
    let pool = StandardGoroutinePool::new(4);
    
    for i in 0..10 {
        pool.spawn(move || {
            println!("Task {} started", i);
            thread::sleep(Duration::from_millis(100));
            println!("Task {} completed", i);
        });
    }

    // Wait for threads to complete
    producer.join().unwrap();
    consumer.join().unwrap();

    // Note: The pool threads will continue running
    thread::sleep(Duration::from_millis(500));
}