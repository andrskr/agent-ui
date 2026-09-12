use crate::process::Cancel;
use anyhow::{Context, Result};
use std::thread::JoinHandle;
pub(crate) struct Worker<T> {
    pub id: String,
    pub cancel: Cancel,
    pub worker: Option<JoinHandle<Result<T>>>,
}
impl<T> Worker<T> {
    pub fn finished(&self) -> bool {
        self.worker.as_ref().is_none_or(JoinHandle::is_finished)
    }
    pub fn cancellation(&self) -> Cancel {
        self.cancel.clone()
    }
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
    pub fn join(mut self) -> Result<T> {
        self.worker
            .take()
            .context("Worker was already joined")?
            .join()
            .map_err(|_| anyhow::anyhow!("Worker panicked"))?
    }
}
impl<T> Drop for Worker<T> {
    fn drop(&mut self) {
        self.cancel();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
