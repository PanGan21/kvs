use super::ThreadPool;

use crate::{KvsError, Result};

pub struct RayonThreadPool(rayon::ThreadPool);

impl ThreadPool for RayonThreadPool {
    fn new(threads: u32) -> Result<Self> {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads as usize)
            .build()
            .map_err(|e| KvsError::StringError(format!("{}", e)))?;
        Ok(RayonThreadPool(pool))
    }

    fn spawn<T>(&self, job: T)
    where
        T: FnOnce() + Send + 'static,
    {
        self.0.spawn(job)
    }
}
