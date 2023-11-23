use std::thread;

use super::ThreadPool;
use crate::Result;

/// A naive implementation of a thread pool that spawns a new thread for each job.
#[derive(Clone)]
pub struct NaiveThreadPool;

/// Implementation of the `ThreadPool` trait for `NaiveThreadPool`.
///
/// It spawns a new thread every time the `spawn` method is called.
impl ThreadPool for NaiveThreadPool {
    /// Creates a new instance of `NaiveThreadPool`.
    ///
    /// # Arguments
    ///
    /// * `_threads` - The number of threads parameter is ignored in this implementation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the newly created `NaiveThreadPool`.
    fn new(_threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(NaiveThreadPool)
    }

    /// Spawns a new thread to execute the provided job.
    ///
    /// # Arguments
    ///
    /// * `job` - A closure representing the job to be executed in a new thread.
    fn spawn<T>(&self, job: T)
    where
        T: FnOnce() + Send + 'static,
    {
        thread::spawn(job);
    }
}
