use super::ThreadPool;
use crate::error::Result;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        let (tx, rx): (
            mpsc::Sender<ThreadPoolMessage>,
            mpsc::Receiver<ThreadPoolMessage>,
        ) = mpsc::channel();

        let rx = Arc::new(Mutex::new(rx));

        for i in 0..threads {
            let rx = WorkerReceiver(rx.clone());
            Worker::new(i, rx);
        }

        Ok(Self {
            tx,
            workers_size: threads as usize,
        })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.tx.send(ThreadPoolMessage::Run(Box::new(job)));
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.workers_size {
            self.tx.send(ThreadPoolMessage::Shutdown);
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum ThreadPoolMessage {
    Run(Job),
    Shutdown,
}

pub struct SharedQueueThreadPool {
    workers_size: usize,
    tx: mpsc::Sender<ThreadPoolMessage>,
}

#[derive(Clone)]
struct WorkerReceiver(Arc<Mutex<mpsc::Receiver<ThreadPoolMessage>>>);

struct Worker {
    _id: u32,
    _handle: thread::JoinHandle<()>,
}

impl Worker {
    fn new(_id: u32, rx: WorkerReceiver) -> Worker {
        let handle = thread::spawn(move || loop {
            let result = rx.0.lock().unwrap().recv();
            match result {
                Ok(message) => match message {
                    ThreadPoolMessage::Run(rx) => rx(),
                    ThreadPoolMessage::Shutdown => break,
                },

                Err(_) => {
                    break;
                }
            }
        });

        Worker {
            _id,
            _handle: handle,
        }
    }
}

impl Drop for WorkerReceiver {
    fn drop(&mut self) {
        if thread::panicking() {
            let rx = self.clone();
            Worker::new(100, rx);
        }
    }
}
