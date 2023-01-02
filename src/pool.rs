use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

#[derive(Debug, Clone)]
pub struct PoolError;

struct Worker {
    id: usize,
    is_end: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
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
                Ok(ThreadPool { workers, sender: Some(sender) })
            }
            false => Err(PoolError)
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let is_end = Arc::new(AtomicBool::new(false));
        Worker {
            id,
            is_end: is_end.clone(),
            thread: Some(create_thread(id, receiver, is_end.clone())),
        }
    }
}

impl ThreadPool {
    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        if let Some(s) = self.sender.as_ref() {
            s.send(job).unwrap();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.sender.take();
        for worker in &mut self.workers {
            if let Some(t) = worker.thread.take() {
                worker.is_end.swap(true, Ordering::Relaxed);
                t.join().unwrap();
            }
        }
    }
}

fn create_thread(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>, is_end: Arc<AtomicBool>) -> JoinHandle<()> {
    thread::spawn(move || loop {
        if is_end.load(Ordering::Relaxed) {
            println!("Worker-{} shutdown", id);
            return;
        }
        receiver
            .try_lock()
            .map(|x| x.try_recv())
            .map(|x| if let Ok(job) = x {
                println!("Worker-{} got a job", id);
                job()
            })
            .unwrap_or_default();
        sleep(Duration::from_millis(1));
    })
}
