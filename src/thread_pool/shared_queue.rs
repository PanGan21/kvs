use super::ThreadPool;
use crate::Result;

pub struct SharedQueueThreadPool;

impl ThreadPool for SharedQueueThreadPool {
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
