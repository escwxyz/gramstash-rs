use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct MemoryCache<T: Clone> {
    cache: Arc<DashMap<String, T>>,
}

impl<T: Clone> MemoryCache<T> {
    pub fn new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            None
        } else {
            Some(Self {
                cache: Arc::new(DashMap::with_capacity(capacity)),
            })
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        if let Some(value) = self.cache.get(key) {
            return Some(value.value().clone());
        } else {
            None
        }
    }

    pub fn set(&self, key: &str, value: &T) {
        self.cache.insert(key.to_string(), value.clone());
    }

    pub fn del(&self, key: &str) {
        self.cache.remove(key);
    }

    pub fn keys(&self, pattern: &str) -> Vec<String> {
        self.cache
            .iter()
            .filter(|entry| entry.key().starts_with(pattern))
            .map(|entry| entry.key().clone())
            .collect()
    }
}
