use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum SendError<T> {
    Full(T),
    Disconnected(T),
}

#[derive(Debug)]
pub enum SendTimeoutError<T> {
    Timeout(T),
    Disconnected(T),
}

#[derive(Debug)]
pub enum RecvError {
    Empty,
    Disconnected,
}

/// A channel-like structure using standard library primitives
pub struct Channelish<T> {
    buffer: Arc<Mutex<VecDeque<T>>>,
    size: Arc<Mutex<usize>>,
    capacity: usize,
    not_full: Arc<Condvar>,
    not_empty: Arc<Condvar>,
    is_closed: Arc<Mutex<bool>>,
}

impl<T> Channelish<T> {
    pub fn new(capacity: usize) -> Self {
        Channelish {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            size: Arc::new(Mutex::new(0)),
            capacity,
            not_full: Arc::new(Condvar::new()),
            not_empty: Arc::new(Condvar::new()),
            is_closed: Arc::new(Mutex::new(false)),
        }
    }

    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        if *self.is_closed.lock().unwrap() {
            return Err(SendError::Disconnected(value));
        }

        let mut size = self.size.lock().unwrap();
        
        while *size >= self.capacity {
            size = match self.not_full.wait(size) {
                Ok(guard) => guard,
                Err(_) => return Err(SendError::Disconnected(value)),
            };
            
            if *self.is_closed.lock().unwrap() {
                return Err(SendError::Disconnected(value));
            }
        }
        
        {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push_back(value);
        }
        
        *size += 1;
        self.not_empty.notify_one();
        Ok(())
    }

    pub fn send_timeout(&self, value: T, timeout: Duration) -> Result<(), SendTimeoutError<T>> {
        if *self.is_closed.lock().unwrap() {
            return Err(SendTimeoutError::Disconnected(value));
        }

        let deadline = Instant::now() + timeout;
        let mut size = self.size.lock().unwrap();
        
        while *size >= self.capacity {
            let now = Instant::now();
            if now >= deadline {
                return Err(SendTimeoutError::Timeout(value));
            }
            
            let wait_time = deadline - now;
            match self.not_full.wait_timeout(size, wait_time) {
                Ok((guard, _)) => size = guard,
                Err(_) => return Err(SendTimeoutError::Disconnected(value)),
            };
            
            if *self.is_closed.lock().unwrap() {
                return Err(SendTimeoutError::Disconnected(value));
            }
        }
        
        {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push_back(value);
        }
        
        *size += 1;
        self.not_empty.notify_one();
        Ok(())
    }

    pub fn recv(&self) -> Result<T, RecvError> {
        let mut size = self.size.lock().unwrap();
        
        while *size == 0 {
            if *self.is_closed.lock().unwrap() {
                return Err(RecvError::Disconnected);
            }
            
            size = match self.not_empty.wait(size) {
                Ok(guard) => guard,
                Err(_) => return Err(RecvError::Disconnected),
            };
        }
        
        let value = {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.pop_front().ok_or(RecvError::Empty)?
        };
        
        *size -= 1;
        self.not_full.notify_one();
        Ok(value)
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvError> {
        let deadline = Instant::now() + timeout;
        let mut size = self.size.lock().unwrap();
        
        while *size == 0 {
            if *self.is_closed.lock().unwrap() {
                return Err(RecvError::Disconnected);
            }
            
            let now = Instant::now();
            if now >= deadline {
                return Err(RecvError::Empty);
            }
            
            let wait_time = deadline - now;
            match self.not_empty.wait_timeout(size, wait_time) {
                Ok((guard, timeout_result)) => {
                    size = guard;
                    if timeout_result.timed_out() {
                        return Err(RecvError::Empty);
                    }
                }
                Err(_) => return Err(RecvError::Disconnected),
            };
        }
        
        let value = {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.pop_front().ok_or(RecvError::Empty)?
        };
        
        *size -= 1;
        self.not_full.notify_one();
        Ok(value)
    }

    pub fn close(&self) {
        let mut closed = self.is_closed.lock().unwrap();
        *closed = true;
        self.not_empty.notify_all();
        self.not_full.notify_all();
    }

    pub fn len(&self) -> usize {
        *self.size.lock().unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<T> Drop for Channelish<T> {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let channel = Channelish::new(2);
        assert!(channel.send(1).is_ok());
        assert!(channel.send(2).is_ok());
        assert_eq!(channel.recv().unwrap(), 1);
        assert_eq!(channel.recv().unwrap(), 2);
    }

    #[test]
    fn test_timeout() {
        let channel = Channelish::new(1);
        assert!(channel.send(1).is_ok());
        assert!(matches!(
            channel.send_timeout(2, Duration::from_millis(100)),
            Err(SendTimeoutError::Timeout(_))
        ));
    }
}


//Note: Both implementations, like the others, do not fully accomplish the task of goroutines, specifically in dynamic scheduling, thread creation in the pool, and channel implementation
// specifically, the tokio runtime allows for a more efficient and dynamic scheduling of tasks, as well as the ability to create a pool of threads that can be used to execute tasks concurrently