pub mod priority;

use dashmap::DashMap;
use priority::PriorityQueue;
use std::sync::Arc;

use crate::platform::{DownloadState, MediaFile, MediaInfo, PostDownloadState};

use super::{
    task::{DownloadTask, PostDownloadTask, TaskWithResult},
    RuntimeError, TaskContext,
};

#[derive(Clone)]
pub struct TaskQueueManager {
    download_queue: Arc<PriorityQueue<DownloadTask>>,
    post_download_queue: Arc<PriorityQueue<PostDownloadTask>>,
    pending_confirmations: Arc<DashMap<String, PostDownloadTask>>, // identifier -> task
}

impl TaskQueueManager {
    pub fn new(capacity: usize) -> Self {
        Self {
            download_queue: Arc::new(PriorityQueue::new(capacity)),
            post_download_queue: Arc::new(PriorityQueue::new(capacity)),
            pending_confirmations: Arc::new(DashMap::new()), // TODO add pending capacity
        }
    }

    pub async fn get_task_by_identifier(&self, identifier: &str) -> Option<PostDownloadTask> {
        self.pending_confirmations.remove(identifier).map(|(_, task)| task)
    }

    pub async fn push_download_task(&self, task: DownloadTask) -> Result<DownloadState, RuntimeError> {
        let priority = task.context.user_tier.into();
        let rx = self.download_queue.push(task, priority).await?;
        rx.await.map_err(|e| RuntimeError::RecvError(e.to_string()))
    }

    async fn push_post_download_task(&self, task: PostDownloadTask) -> Result<PostDownloadState, RuntimeError> {
        let priority = task.context.user_tier.into();
        let rx = self.post_download_queue.push(task, priority).await?;
        rx.await.map_err(|e| RuntimeError::RecvError(e.to_string()))
    }

    pub async fn handle_download_confirmation(&self, identifier: &str) -> Result<PostDownloadState, RuntimeError> {
        if let Some(task) = self.get_task_by_identifier(identifier).await {
            let post_task = PostDownloadTask::new(task.media_file, task.context);

            let state = self.push_post_download_task(post_task).await?;
            Ok(state)
        } else {
            Err(RuntimeError::QueueError("task not found".to_string()))
        }
    }

    pub async fn pop_download_task(&self) -> Option<TaskWithResult<DownloadTask>> {
        self.download_queue.pop().await
    }

    pub async fn pop_post_download_task(&self) -> Option<TaskWithResult<PostDownloadTask>> {
        self.post_download_queue.pop().await
    }

    pub fn add_pending_confirmation(&self, media_info: MediaInfo, context: TaskContext) {
        let media_file: MediaFile = media_info.into();

        let post_task = PostDownloadTask::new(media_file.clone(), context);

        self.pending_confirmations.insert(media_file.identifier, post_task);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::runtime::task::{CacheTask, DownloadTask, TaskContext, UserTier};
//     use std::time::Duration;
//     use tokio::time::sleep;
//     use url::Url;

//     fn create_test_context(user_tier: UserTier) -> TaskContext {
//         TaskContext {
//             user_id: 1,
//             chat_id: 1,
//             message_id: 1,
//             user_tier,
//         }
//     }

//     fn create_download_task(urls: &[Url], user_tier: UserTier) -> DownloadTask {
//         DownloadTask::new(urls.to_vec(), create_test_context(user_tier))
//     }

//     fn create_cache_task(download_task_id: &str, file_id: &str, user_tier: UserTier) -> CacheTask {
//         CacheTask::new(
//             download_task_id.to_string(),
//             file_id.to_string(),
//             create_test_context(user_tier),
//         )
//     }

//     //     #[tokio::test]
//     //     async fn test_priority_ordering() {
//     //         let manager = TaskQueueManager::new(10);

//     //         let low_task = create_download_task("low.com", UserTier::Free);
//     //         let normal_task = create_download_task("normal.com", UserTier::OneTimePaid);
//     //         let high_task = create_download_task("high.com", UserTier::Subscriber);

//     //         manager.push_download_task(low_task).await.unwrap();
//     //         manager.push_download_task(normal_task).await.unwrap();
//     //         manager.push_download_task(high_task).await.unwrap();

//     //         let task1 = manager.pop_download_task().await.unwrap();
//     //         let task2 = manager.pop_download_task().await.unwrap();
//     //         let task3 = manager.pop_download_task().await.unwrap();

//     //         assert_eq!(task1.url, "high.com");
//     //         assert_eq!(task2.url, "normal.com");
//     //         assert_eq!(task3.url, "low.com");
//     //     }

//     //     #[tokio::test]
//     //     async fn test_fifo_within_same_priority() {
//     //         let manager = TaskQueueManager::new(10);

//     //         let task1 = create_download_task("first.com", UserTier::Subscriber);
//     //         let task2 = create_download_task("second.com", UserTier::Subscriber);

//     //         manager.push_download_task(task1).await.unwrap();
//     //         sleep(Duration::from_micros(10)).await;
//     //         manager.push_download_task(task2).await.unwrap();

//     //         let popped1 = manager.pop_download_task().await.unwrap();
//     //         let popped2 = manager.pop_download_task().await.unwrap();

//     //         assert_eq!(popped1.url, "first.com");
//     //         assert_eq!(popped2.url, "second.com");
//     //     }

//     //     #[tokio::test]
//     //     async fn test_queue_capacity() {
//     //         let manager = TaskQueueManager::new(2);

//     //         let task1 = create_download_task("url1.com", UserTier::Free);
//     //         let task2 = create_download_task("url2.com", UserTier::Free);
//     //         let task3 = create_download_task("url3.com", UserTier::Free);

//     //         assert!(manager.push_download_task(task1).await.is_ok());
//     //         assert!(manager.push_download_task(task2).await.is_ok());

//     //         assert!(manager.push_download_task(task3).await.is_err());
//     //     }

//     //     #[tokio::test]
//     //     async fn test_pending_confirmation() {
//     //         let manager = TaskQueueManager::new(10);
//     //         let task = create_download_task("test.com", UserTier::Free);
//     //         let task_id = task.id.clone();

//     //         manager.add_pending_confirmation(task.clone()).await;

//     //         let confirmed = manager.confirm_download(&task_id).unwrap();
//     //         assert_eq!(confirmed.url, "test.com");

//     //         assert!(manager.confirm_download(&task_id).is_none());
//     //     }

//     //     #[tokio::test]
//     //     async fn test_concurrent_operations() {
//     //         let manager = Arc::new(TaskQueueManager::new(100));
//     //         let mut handles = vec![];

//     //         for i in 0..10 {
//     //             let manager = manager.clone();
//     //             let handle = tokio::spawn(async move {
//     //                 let task = create_download_task(&format!("url{}.com", i), UserTier::Free);
//     //                 manager.push_download_task(task).await.unwrap();
//     //             });
//     //             handles.push(handle);
//     //         }

//     //         for handle in handles {
//     //             handle.await.unwrap();
//     //         }

//     //         let mut count = 0;
//     //         while let Some(_) = manager.pop_download_task().await {
//     //             count += 1;
//     //         }
//     //         assert_eq!(count, 10);
//     //     }

//     //     #[tokio::test]
//     //     async fn test_cache_task_priority() {
//     //         let manager = TaskQueueManager::new(10);

//     //         let download_task = create_download_task("test.com", UserTier::Subscriber);
//     //         let download_id = download_task.id.clone();

//     //         manager.add_pending_confirmation(download_task).await;

//     //         let low_task = create_cache_task(&download_id, "file1", UserTier::Free);
//     //         let normal_task = create_cache_task(&download_id, "file2", UserTier::OneTimePaid);
//     //         let high_task = create_cache_task(&download_id, "file3", UserTier::Subscriber);

//     //         manager.push_cache_task(low_task).await.unwrap();
//     //         manager.push_cache_task(normal_task).await.unwrap();
//     //         manager.push_cache_task(high_task).await.unwrap();

//     //         let task1 = manager.pop_cache_task().await.unwrap();
//     //         let task2 = manager.pop_cache_task().await.unwrap();
//     //         let task3 = manager.pop_cache_task().await.unwrap();

//     //         assert_eq!(task1.file_id, "file3"); // High priority
//     //         assert_eq!(task2.file_id, "file2"); // Normal priority
//     //         assert_eq!(task3.file_id, "file1"); // Low priority
//     //     }

//     //     #[tokio::test]
//     //     async fn test_download_cache_relationship() {
//     //         let manager = TaskQueueManager::new(10);

//     //         let download_task = create_download_task("test.com", UserTier::Subscriber);
//     //         let download_id = download_task.id.clone();

//     //         manager.push_download_task(download_task.clone()).await.unwrap();
//     //         let popped_download = manager.pop_download_task().await.unwrap();
//     //         manager.add_pending_confirmation(popped_download).await;

//     //         let confirmed = manager.confirm_download(&download_id).unwrap();
//     //         assert_eq!(confirmed.url, "test.com");

//     //         let cache_task = create_cache_task(&download_id, "file1", UserTier::Subscriber);
//     //         manager.push_cache_task(cache_task).await.unwrap();

//     //         let popped_cache = manager.pop_cache_task().await.unwrap();
//     //         assert_eq!(popped_cache.download_task_id, download_id);
//     //     }

//     //     #[tokio::test]
//     //     async fn test_cache_task_without_download() {
//     //         let manager = TaskQueueManager::new(10);

//     //         let cache_task = create_cache_task("non_existent_id", "file1", UserTier::Free);

//     //         manager.push_cache_task(cache_task).await.unwrap();

//     //         // TODO
//     //         let popped = manager.pop_cache_task().await.unwrap();
//     //         assert!(manager.confirm_download(&popped.download_task_id).is_none());
//     //     }

//     //     #[tokio::test]
//     //     async fn test_concurrent_download_and_cache() {
//     //         let manager = Arc::new(TaskQueueManager::new(100));
//     //         let mut handles = vec![];

//     //         for i in 0..5 {
//     //             let manager = manager.clone();
//     //             let handle = tokio::spawn(async move {
//     //                 let download_task = create_download_task(&format!("url{}.com", i), UserTier::Free);
//     //                 let download_id = download_task.id.clone();

//     //                 manager.push_download_task(download_task.clone()).await.unwrap();

//     //                 sleep(Duration::from_millis(10)).await;

//     //                 let cache_task = create_cache_task(&download_id, &format!("file{}", i), UserTier::Free);
//     //                 manager.push_cache_task(cache_task).await.unwrap();
//     //             });
//     //             handles.push(handle);
//     //         }

//     //         for handle in handles {
//     //             handle.await.unwrap();
//     //         }

//     //         let mut download_count = 0;
//     //         let mut cache_count = 0;

//     //         while let Some(_) = manager.pop_download_task().await {
//     //             download_count += 1;
//     //         }
//     //         while let Some(_) = manager.pop_cache_task().await {
//     //             cache_count += 1;
//     //         }

//     //         assert_eq!(download_count, 5);
//     //         assert_eq!(cache_count, 5);
//     //     }
// }
