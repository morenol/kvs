use std::thread;

use super::ThreadPool;
use crate::error::Result;

pub struct NaiveThreadPool;

impl ThreadPool for NaiveThreadPool {
    fn new(_threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self)
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(move || {
            job();
        });
    }
}
