use std::sync::Arc;

use super::ThreadPool;

use crate::{KvsError, Result};

/// A thread pool implementation using the Rayon library.
#[derive(Clone)]
pub struct RayonThreadPool(Arc<rayon::ThreadPool>);

/// Implementation of the `ThreadPool` trait for `RayonThreadPool`.
impl ThreadPool for RayonThreadPool {
    /// Creates a new instance of `RayonThreadPool` with the specified number of threads.
    ///
    /// # Arguments
    ///
    /// * `threads` - The number of threads in the pool.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the newly created `RayonThreadPool`.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an issue creating the Rayon thread pool.
    fn new(threads: u32) -> Result<Self> {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads as usize)
            .build()
            .map_err(|e| KvsError::StringError(format!("{}", e)))?;
        Ok(RayonThreadPool(Arc::new(pool)))
    }

    /// Spawns a new task to be executed in the Rayon thread pool.
    ///
    /// # Arguments
    ///
    /// * `job` - A closure representing the task to be executed in the pool.
    fn spawn<T>(&self, job: T)
    where
        T: FnOnce() + Send + 'static,
    {
        self.0.spawn(job)
    }
}
