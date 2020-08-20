use crate::Result;
use std::thread;

pub struct NaiveThreadPool;
pub struct SharedQueueThreadPool;
pub struct RayonThreadPool;

pub trait ThreadPool  {
    fn new (threads: u32) -> Result<Self>
    where Self:Sized;

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}

impl ThreadPool for NaiveThreadPool {
    fn new (threads: u32) -> Result<Self> {
        Ok(NaiveThreadPool)
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(move ||{
            job();
        });
    }
}

impl ThreadPool for SharedQueueThreadPool {
    fn new (threads: u32) -> Result<Self> {
        unimplemented!()
    }

    fn spawn<F>(&self, job: F) {
        unimplemented!()
    }
}

impl ThreadPool for RayonThreadPool {
    fn new (threads: u32) -> Result<Self> {
        unimplemented!()
    }

    fn spawn<F>(&self, job: F) {
        unimplemented!()
    }
}

