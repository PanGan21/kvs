use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use log::{debug, error};

use super::ThreadPool;
use crate::Result;

pub struct SharedQueueThreadPool {
    tx: Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<Self> {
        let (tx, rx) = channel();
        let rx = Arc::new(Mutex::new(rx));

        for _ in 0..threads {
            let rx = Arc::clone(&rx);
            let rx = JobReceiver(rx);
            thread::Builder::new().spawn(move || execute(rx))?;
        }
        Ok(SharedQueueThreadPool { tx })
    }

    fn spawn<T>(&self, job: T)
    where
        T: FnOnce() + Send + 'static,
    {
        self.tx
            .send(Box::new(job))
            .expect("The thread pool has no thread.");
    }
}

type ConcurrentReceiver = Arc<Mutex<Receiver<Box<dyn FnOnce() + Send + 'static>>>>;
struct JobReceiver(ConcurrentReceiver);

impl Drop for JobReceiver {
    fn drop(&mut self) {
        if thread::panicking() {
            let rx = self.0.clone();
            let rx = JobReceiver(rx);
            if let Err(e) = thread::Builder::new().spawn(move || execute(rx)) {
                error!("Failed to spawn a thread: {}", e);
            }
        }
    }
}

fn execute(rx: JobReceiver) {
    loop {
        let job = rx.0.lock().unwrap().recv();
        match job {
            Ok(job) => {
                job();
            }
            Err(_) => debug!("Thread pool is destroyed, thread exits"),
        }
    }
}
