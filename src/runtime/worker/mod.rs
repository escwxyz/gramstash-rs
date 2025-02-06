pub mod download;

use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::broadcast;

use super::RuntimeError;

#[async_trait]
pub trait Worker: Send + Sync + 'static {
    fn name(&self) -> &str;
    async fn start(&self) -> Result<(), RuntimeError>;
    #[allow(dead_code)]
    async fn stop(&self) -> Result<(), RuntimeError>;
    #[allow(dead_code)]
    fn is_running(&self) -> bool;
}

pub struct WorkerPool {
    workers: HashMap<String, Box<dyn Worker>>, // TODO: use DashMap
    #[allow(dead_code)]
    shutdown: broadcast::Sender<()>,
}

impl WorkerPool {
    pub fn new() -> Self {
        let (shutdown, _) = broadcast::channel(1);
        Self {
            workers: HashMap::new(),
            shutdown,
        }
    }

    pub fn add_worker<W: Worker + 'static>(&mut self, worker: W) {
        self.workers.insert(worker.name().to_string(), Box::new(worker));
    }

    pub async fn start_all(&self) -> Result<(), RuntimeError> {
        for worker in self.workers.values() {
            worker.start().await?;
        }
        Ok(())
    }
    // TODO: graceful shutdown
    // pub async fn stop_all(&self) -> Result<(), RuntimeError> {
    //     let _ = self.shutdown.send(());
    //     for worker in self.workers.values() {
    //         worker.stop().await?;
    //     }
    //     Ok(())
    // }
}
