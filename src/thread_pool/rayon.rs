use super::ThreadPool;

use crate::Result;

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
