//! Cache interface and implementations

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, RwLock};

/// Cache interface
pub trait Cache: Send + Sync + std::fmt::Debug {
    /// Insert a key-value pair
    fn insert(&self, key: &[u8], value: Vec<u8>, charge: usize) -> Option<Vec<u8>>;

    /// Lookup a value by key
    fn lookup(&self, key: &[u8]) -> Option<Vec<u8>>;

    /// Remove a key from cache
    fn erase(&self, key: &[u8]) -> bool;

    /// Get cache usage
    fn get_usage(&self) -> usize;

    /// Get cache capacity
    fn get_capacity(&self) -> usize;

    /// Set cache capacity
    fn set_capacity(&self, capacity: usize);

    /// Get number of entries
    fn get_entry_count(&self) -> usize;

    /// Clear all entries
    fn clear(&self);
}

/// LRU Cache implementation
#[derive(Debug)]
pub struct LRUCache {
    capacity: RwLock<usize>,
    usage: RwLock<usize>,
    data: Mutex<HashMap<Vec<u8>, CacheEntry>>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: Vec<u8>,
    charge: usize,
    access_time: std::time::Instant,
}

impl LRUCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: RwLock::new(capacity),
            usage: RwLock::new(0),
            data: Mutex::new(HashMap::new()),
        }
    }

    fn evict_if_needed(&self) {
        let capacity = *self.capacity.read().unwrap();
        let mut data = self.data.lock().unwrap();
        let mut usage = self.usage.write().unwrap();

        while *usage > capacity && !data.is_empty() {
            // Find least recently used entry
            let mut oldest_key = None;
            let mut oldest_time = std::time::Instant::now();

            for (key, entry) in data.iter() {
                if entry.access_time < oldest_time {
                    oldest_time = entry.access_time;
                    oldest_key = Some(key.clone());
                }
            }

            if let Some(key) = oldest_key {
                if let Some(entry) = data.remove(&key) {
                    *usage -= entry.charge;
                }
            }
        }
    }
}

impl Cache for LRUCache {
    fn insert(&self, key: &[u8], value: Vec<u8>, charge: usize) -> Option<Vec<u8>> {
        let entry = CacheEntry {
            value: value.clone(),
            charge,
            access_time: std::time::Instant::now(),
        };

        let mut data = self.data.lock().unwrap();
        let mut usage = self.usage.write().unwrap();

        let old_value = if let Some(old_entry) = data.insert(key.to_vec(), entry) {
            *usage = usage.saturating_sub(old_entry.charge).saturating_add(charge);
            Some(old_entry.value)
        } else {
            *usage += charge;
            None
        };

        drop(data);
        drop(usage);

        self.evict_if_needed();
        old_value
    }

    fn lookup(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut data = self.data.lock().unwrap();
        if let Some(entry) = data.get_mut(key) {
            entry.access_time = std::time::Instant::now();
            Some(entry.value.clone())
        } else {
            None
        }
    }

    fn erase(&self, key: &[u8]) -> bool {
        let mut data = self.data.lock().unwrap();
        if let Some(entry) = data.remove(key) {
            let mut usage = self.usage.write().unwrap();
            *usage -= entry.charge;
            true
        } else {
            false
        }
    }

    fn get_usage(&self) -> usize {
        *self.usage.read().unwrap()
    }

    fn get_capacity(&self) -> usize {
        *self.capacity.read().unwrap()
    }

    fn set_capacity(&self, capacity: usize) {
        *self.capacity.write().unwrap() = capacity;
        self.evict_if_needed();
    }

    fn get_entry_count(&self) -> usize {
        self.data.lock().unwrap().len()
    }

    fn clear(&self) {
        let mut data = self.data.lock().unwrap();
        let mut usage = self.usage.write().unwrap();
        data.clear();
        *usage = 0;
    }
}

/// No-op cache for testing
#[derive(Debug)]
pub struct NoCache;

impl Cache for NoCache {
    fn insert(&self, _key: &[u8], value: Vec<u8>, _charge: usize) -> Option<Vec<u8>> {
        Some(value) // Return the value as if it was replaced
    }

    fn lookup(&self, _key: &[u8]) -> Option<Vec<u8>> {
        None
    }

    fn erase(&self, _key: &[u8]) -> bool {
        false
    }

    fn get_usage(&self) -> usize {
        0
    }

    fn get_capacity(&self) -> usize {
        0
    }

    fn set_capacity(&self, _capacity: usize) {}

    fn get_entry_count(&self) -> usize {
        0
    }

    fn clear(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic() {
        let cache = LRUCache::new(100);

        // Test insert and lookup
        cache.insert(b"key1", b"value1".to_vec(), 10);
        assert_eq!(cache.lookup(b"key1"), Some(b"value1".to_vec()));
        assert_eq!(cache.lookup(b"nonexistent"), None);

        assert_eq!(cache.get_usage(), 10);
        assert_eq!(cache.get_entry_count(), 1);
    }

    #[test]
    fn test_lru_cache_eviction() {
        let cache = LRUCache::new(20);

        // Fill cache to capacity
        cache.insert(b"key1", b"value1".to_vec(), 10);
        cache.insert(b"key2", b"value2".to_vec(), 10);

        // This should trigger eviction
        cache.insert(b"key3", b"value3".to_vec(), 10);

        // One of the earlier entries should have been evicted
        assert!(cache.get_usage() <= 20);
        assert_eq!(cache.lookup(b"key3"), Some(b"value3".to_vec()));
    }

    #[test]
    fn test_lru_cache_erase() {
        let cache = LRUCache::new(100);

        cache.insert(b"key1", b"value1".to_vec(), 10);
        assert_eq!(cache.get_usage(), 10);

        assert!(cache.erase(b"key1"));
        assert_eq!(cache.get_usage(), 0);
        assert_eq!(cache.lookup(b"key1"), None);

        assert!(!cache.erase(b"nonexistent"));
    }

    #[test]
    fn test_lru_cache_clear() {
        let cache = LRUCache::new(100);

        cache.insert(b"key1", b"value1".to_vec(), 10);
        cache.insert(b"key2", b"value2".to_vec(), 10);

        cache.clear();
        assert_eq!(cache.get_usage(), 0);
        assert_eq!(cache.get_entry_count(), 0);
    }

    #[test]
    fn test_no_cache() {
        let cache = NoCache;

        assert_eq!(cache.insert(b"key", b"value".to_vec(), 10), Some(b"value".to_vec()));
        assert_eq!(cache.lookup(b"key"), None);
        assert!(!cache.erase(b"key"));
        assert_eq!(cache.get_usage(), 0);
        assert_eq!(cache.get_capacity(), 0);
        assert_eq!(cache.get_entry_count(), 0);
    }
}