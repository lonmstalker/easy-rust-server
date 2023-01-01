use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

#[derive(Debug, Clone)]
pub struct PoolError;

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn build(size: usize) -> Result<ThreadPool, PoolError> {
        match size > 0 {
            true => {
                let (sender, receiver) = mpsc::channel::<Job>();
                let receiver = Arc::new(Mutex::new(receiver));
                let mut workers = Vec::with_capacity(size);
                for i in 0..size {
                    workers.push(Worker::new(i, Arc::clone(&receiver)));
                }
                Ok(ThreadPool { workers, sender })
            }
            false => Err(PoolError)
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        Worker {
            id,
            thread: thread::spawn(move || loop {
                receiver
                    .try_lock()
                    .map(|x| x.try_recv())
                    .map(|x| if let Ok(job) = x {
                        println!("Worker-{} got a job", id);
                        job()
                    })
                    .unwrap_or_default();
                thread::sleep(Duration::from_millis(1));
            }),
        }
    }
}

impl ThreadPool {
    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}
