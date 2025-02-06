use std::sync::Arc;
use teloxide::{adaptors::Throttle, Bot};
use tokio::sync::broadcast;

mod cache;
mod error;
mod queue;
mod task;
mod worker;

pub use cache::*;
pub use error::*;
pub use queue::TaskQueueManager;
pub use task::{DownloadTask, TaskContext};
pub use worker::WorkerPool;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct RuntimeManager {
    pub queue_manager: TaskQueueManager,
    pub worker_pool: Arc<WorkerPool>,
    shutdown: broadcast::Sender<()>,
}

impl RuntimeManager {
    pub fn new(queue_capacity: usize, bot: Throttle<Bot>) -> Result<Self, RuntimeError> {
        info!("Initializing RuntimeManager...");
        let (shutdown_tx, _) = broadcast::channel(1);
        let queue_manager = TaskQueueManager::new(queue_capacity);
        let mut worker_pool = WorkerPool::new();

        info!("Adding download worker...");

        let concurrency = AppConfig::get()?.runtime.queue.worker_count;

        worker_pool.add_worker(worker::download::DownloadWorker::new(
            "download_worker",
            concurrency,
            queue_manager.clone(),
            bot.clone(),
            // shutdown_tx.clone(),
        ));

        info!("Adding post download worker...");

        worker_pool.add_worker(worker::download::PostDownloadWorker::new(
            "post_download_worker",
            concurrency,
            queue_manager.clone(),
            bot.clone(),
            // shutdown_tx.clone(),
        ));

        info!("RuntimeManager initialized");

        Ok(Self {
            queue_manager,
            worker_pool: Arc::new(worker_pool),
            shutdown: shutdown_tx,
        })
    }

    pub async fn start(&self) -> Result<(), RuntimeError> {
        self.worker_pool.start_all().await?;
        Ok(())
    }

    // pub async fn shutdown(&self) -> Result<(), RuntimeError> {
    //     self.shutdown
    //         .send(())
    //         .map_err(|e| RuntimeError::QueueError(e.to_string()))?;

    //     self.worker_pool.stop_all().await?;
    //     Ok(())
    // }

    // pub fn get_queue_manager(&self) -> Arc<TaskQueueManager> {
    //     Arc::clone(&self.queue_manager)
    // }

    // pub fn get_worker_pool(&self) -> Arc<WorkerPool> {
    //     Arc::clone(&self.worker_pool)
    // }
}

impl Drop for RuntimeManager {
    fn drop(&mut self) {
        let _ = self.shutdown.send(());
    }
}
