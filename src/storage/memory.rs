use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct MemoryCache<T: Clone> {
    cache: Arc<DashMap<String, T>>,
}

impl<T: Clone> MemoryCache<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(DashMap::with_capacity(capacity)),
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

    // pub fn iter(&self) -> impl Iterator<Item = (String, T)> {
    //     self.cache.iter()
    // }
}
