use crate::runtime::RuntimeError;
use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    High,
    Normal,
    Low,
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

#[derive(Debug, Clone, Eq, PartialEq)]
struct PrioritizedItem<T> {
    priority: Priority,
    timestamp: DateTime<Utc>,
    item: T,
}

impl<T: Eq> PartialOrd for PrioritizedItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Eq> Ord for PrioritizedItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => self.timestamp.cmp(&other.timestamp).reverse(),
            other => other,
        }
    }
}

pub struct PriorityQueue<T> {
    inner: Mutex<BinaryHeap<PrioritizedItem<T>>>,
    capacity: usize,
}
impl<T: Ord> PriorityQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(BinaryHeap::with_capacity(capacity)),
            capacity,
        }
    }

    pub async fn push(&self, item: T, priority: Priority) -> Result<(), RuntimeError> {
        let mut queue = self.inner.lock().await;
        if queue.len() >= self.capacity {
            return Err(RuntimeError::QueueError("Queue is full".to_string()));
        }
        queue.push(PrioritizedItem {
            priority,
            timestamp: Utc::now(),
            item,
        });
        Ok(())
    }

    pub async fn pop(&self) -> Option<T> {
        let mut queue = self.inner.lock().await;
        queue.pop().map(|item| item.item)
    }
}
