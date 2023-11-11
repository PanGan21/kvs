use crate::Result;

mod naive;
mod shared_queue;

pub use naive::NaiveThreadPool;
pub use shared_queue::SharedQueueThreadPool;

pub trait ThreadPool {
    fn new(threads: u32) -> Result<Self>
    where
        Self: std::marker::Sized;

    fn spawn<T>(&self, job: T)
    where
        T: FnOnce() + Send + 'static;
}

pub struct RayonThreadPool;

impl ThreadPool for RayonThreadPool {
    fn new(_threads: u32) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        todo!()
    }

    fn spawn<T>(&self, _job: T) {
        todo!()
    }
}
