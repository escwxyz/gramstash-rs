use crate::context::UserTier;
use crate::runtime::task::{Task, TaskWithResult};
use crate::runtime::RuntimeError;
use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use tokio::sync::{oneshot, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    High,
    Normal,
    Low,
}

impl From<UserTier> for Priority {
    fn from(tier: UserTier) -> Self {
        match tier {
            UserTier::Subscriber => Priority::High,
            UserTier::OneTimePaid => Priority::Normal,
            UserTier::Free => Priority::Low,
        }
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Priority::High, Priority::High) => Ordering::Equal,
            (Priority::High, _) => Ordering::Greater,
            (Priority::Normal, Priority::High) => Ordering::Less,
            (Priority::Normal, Priority::Normal) => Ordering::Equal,
            (Priority::Normal, Priority::Low) => Ordering::Greater,
            (Priority::Low, Priority::Low) => Ordering::Equal,
            (Priority::Low, _) => Ordering::Less,
        }
    }
}

#[derive(Debug)]
struct PrioritizedItem<T: Task> {
    priority: Priority,
    timestamp: DateTime<Utc>,
    task: T,
    result_tx: oneshot::Sender<T::Result>,
}

impl<T: Task> PartialEq for PrioritizedItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl<T: Task> Eq for PrioritizedItem<T> {}

impl<T: Task> PartialOrd for PrioritizedItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Task> Ord for PrioritizedItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => self.timestamp.cmp(&other.timestamp).reverse(),
            other => other,
        }
    }
}

pub struct PriorityQueue<T: Task> {
    inner: Mutex<BinaryHeap<PrioritizedItem<T>>>,
    capacity: usize,
}

impl<T: Task> PriorityQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(BinaryHeap::with_capacity(capacity)),
            capacity,
        }
    }

    pub async fn push(&self, task: T, priority: Priority) -> Result<oneshot::Receiver<T::Result>, RuntimeError> {
        let mut queue = self.inner.lock().await;
        if queue.len() >= self.capacity {
            return Err(RuntimeError::QueueError("Queue is full".to_string()));
        }

        let (tx, rx) = oneshot::channel();

        queue.push(PrioritizedItem {
            priority,
            timestamp: Utc::now(),
            task,
            result_tx: tx,
        });

        Ok(rx)
    }

    pub async fn pop(&self) -> Option<TaskWithResult<T>> {
        let mut queue = self.inner.lock().await;
        queue.pop().map(|item| TaskWithResult {
            task: item.task,
            result_tx: item.result_tx,
        })
    }
    // TODO: for admin to check status
    // pub async fn len(&self) -> usize {
    //     self.inner.lock().await.len()
    // }

    // pub async fn is_empty(&self) -> bool {
    //     self.inner.lock().await.is_empty()
    // }

    // pub fn capacity(&self) -> usize {
    //     self.capacity
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[derive(Debug)]
    struct TestTask {
        id: i32,
    }

    impl Task for TestTask {
        type Result = i32;
    }

    #[tokio::test]
    async fn test_priority_queue() {
        let queue = PriorityQueue::<TestTask>::new(5);

        // Push tasks with different priorities
        let rx1 = queue.push(TestTask { id: 1 }, Priority::Low).await.unwrap();
        sleep(Duration::from_millis(10)).await;
        let rx2 = queue.push(TestTask { id: 2 }, Priority::High).await.unwrap();
        sleep(Duration::from_millis(10)).await;
        let rx3 = queue.push(TestTask { id: 3 }, Priority::Normal).await.unwrap();

        // Pop tasks and verify order
        let task1 = queue.pop().await.unwrap();
        assert_eq!(task1.task.id, 2); // High priority
        task1.result_tx.send(task1.task.id).unwrap();

        let task2 = queue.pop().await.unwrap();
        assert_eq!(task2.task.id, 3); // Normal priority
        task2.result_tx.send(task2.task.id).unwrap();

        let task3 = queue.pop().await.unwrap();
        assert_eq!(task3.task.id, 1); // Low priority
        task3.result_tx.send(task3.task.id).unwrap();

        // Verify results
        assert_eq!(rx2.await.unwrap(), 2);
        assert_eq!(rx3.await.unwrap(), 3);
        assert_eq!(rx1.await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_queue_capacity() {
        let queue = PriorityQueue::<TestTask>::new(2);

        // Fill queue to capacity
        queue.push(TestTask { id: 1 }, Priority::Normal).await.unwrap();
        queue.push(TestTask { id: 2 }, Priority::Normal).await.unwrap();

        // Try to push when full
        let result = queue.push(TestTask { id: 3 }, Priority::Normal).await;
        assert!(result.is_err());
    }
}
