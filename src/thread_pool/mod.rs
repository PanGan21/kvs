use crate::Result;

mod naive;
mod rayon;
mod shared_queue;

pub use naive::NaiveThreadPool;
pub use rayon::RayonThreadPool;
pub use shared_queue::SharedQueueThreadPool;

pub trait ThreadPool {
    fn new(threads: u32) -> Result<Self>
    where
        Self: std::marker::Sized;

    fn spawn<T>(&self, job: T)
    where
        T: FnOnce() + Send + 'static;
}
