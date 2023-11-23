use crate::Result;

mod naive;
mod rayon;
mod shared_queue;

pub use naive::NaiveThreadPool;
pub use rayon::RayonThreadPool;
pub use shared_queue::SharedQueueThreadPool;

/// A trait for defining a simple thread pool.
pub trait ThreadPool: Clone + Send + 'static {
    /// Creates a new thread pool with the specified number of threads.
    ///
    /// # Arguments
    ///
    /// * `threads` - The number of threads to create in the pool.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the newly created thread pool if successful.
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;

    /// Spawns a new job in the thread pool.
    ///
    /// # Arguments
    ///
    /// * `job` - A closure representing the job to be executed in the thread pool.
    ///
    /// # Notes
    ///
    /// The closure should take no arguments (`FnOnce()`) and be both `Send` and `'static`.
    fn spawn<T>(&self, job: T)
    where
        T: FnOnce() + Send + 'static;
}
