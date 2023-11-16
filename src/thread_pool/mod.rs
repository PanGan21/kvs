use crate::Result;

mod naive;
mod rayon;
mod shared_queue;

pub use naive::NaiveThreadPool;
pub use rayon::RayonThreadPool;
pub use shared_queue::SharedQueueThreadPool;

/// A trait for defining a simple thread pool.
pub trait ThreadPool {
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
    ///
    /// # Example
    ///
    /// ```
    /// use your_thread_pool_crate::ThreadPool;
    ///
    /// let pool = ThreadPool::new(4).expect("Failed to create thread pool");
    /// pool.spawn(|| {
    ///     // Your job implementation
    /// });
    /// ```
    fn spawn<T>(&self, job: T)
    where
        T: FnOnce() + Send + 'static;
}
