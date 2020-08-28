use crate::Result;

mod naive;
mod shared_queue;

pub struct RayonThreadPool;

pub use self::naive::NaiveThreadPool;
pub use self::shared_queue::SharedQueueThreadPool;

pub trait ThreadPool {
    fn new(threads: usize) -> Result<Self>
    where
        Self: Sized;

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}

impl ThreadPool for RayonThreadPool {
    fn new(_threads: usize) -> Result<Self> {
        unimplemented!()
    }

    fn spawn<F>(&self, _job: F) {
        unimplemented!()
    }
}
