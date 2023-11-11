use std::thread;

use super::ThreadPool;
use crate::Result;

pub struct NaiveThreadPool;

/// It spawns a new thread every time the `spawn` method is called.
impl ThreadPool for NaiveThreadPool {
    fn new(_threads: u32) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        Ok(NaiveThreadPool)
    }

    fn spawn<T>(&self, job: T)
    where
        T: FnOnce() + Send + 'static,
    {
        thread::spawn(job);
    }
}
