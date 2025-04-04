use std::{fmt, sync::{mpsc, Arc, Mutex}, thread};

#[derive(Debug)]
pub enum PoolCreationError {
    TooFewThreads,
    TooManyThreads
}

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PoolCreationError::TooFewThreads => write!(f, "You must use at least one thread on the pool"),
            PoolCreationError::TooManyThreads => write!(f, "You must use less than {} threads on the pool", u32::MAX),
        }
    }
}
impl std::error::Error for PoolCreationError {}




struct Worker {
    id: usize,
    handle: thread::JoinHandle<()>
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });
        Worker {
            id,
            handle: thread
        }
    }
}


type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size < 1 {
            return Err(PoolCreationError::TooFewThreads)
        } else if size > usize::MAX { // I think this is unreachable
            return Err(PoolCreationError::TooManyThreads)
        }

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool { workers, sender: Some(sender) })
    }

    /// Receives a closure compatible with the thread::spawn() function.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static, // to receive a closure compatible with thread.spawn()
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);

            worker.handle.join().unwrap();
        }
    }
}