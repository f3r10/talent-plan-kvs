use super::Result;
use super::ThreadPool;

pub struct RayonThreadPool {
    pool: rayon::ThreadPool,
}

impl ThreadPool for RayonThreadPool {
    fn new(threads: usize) -> Result<Self> {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("RayonThreadPool was not created");
        Ok(RayonThreadPool { pool })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pool.spawn(job)
    }
}
