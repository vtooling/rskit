//! Thread-safe LRU cache (wraps the `lru` crate in a `Mutex`).

use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Mutex;

/// A thread-safe LRU cache with a fixed capacity.
pub struct LruCache<K: Hash + Eq, V> {
    inner: Mutex<lru::LruCache<K, V>>,
}

impl<K: Hash + Eq, V> LruCache<K, V> {
    /// Create a cache holding at most `capacity` entries. A `capacity` of 0 is
    /// treated as 1 (the underlying crate requires a non-zero size).
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::MIN);
        LruCache {
            inner: Mutex::new(lru::LruCache::new(cap)),
        }
    }

    /// Insert or overwrite a key. If at capacity, evicts the least-recently-used.
    pub fn put(&self, key: K, value: V) {
        self.inner.lock().expect("lru lock").put(key, value);
    }

    /// Fetch a value, marking it most-recently-used. Returns `None` if absent.
    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        self.inner.lock().expect("lru lock").get(key).cloned()
    }

    /// Peek without updating recency. Returns `None` if absent.
    pub fn peek(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        self.inner.lock().expect("lru lock").peek(key).cloned()
    }

    /// Remove a key, returning its value.
    pub fn pop(&self, key: &K) -> Option<V> {
        self.inner.lock().expect("lru lock").pop(key)
    }

    /// Current number of entries.
    pub fn len(&self) -> usize {
        self.inner.lock().expect("lru lock").len()
    }

    /// Is the cache empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Change the capacity, evicting LRU entries if shrinking.
    pub fn resize(&self, capacity: usize) {
        let cap = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::MIN);
        self.inner.lock().expect("lru lock").resize(cap);
    }

    /// Remove all entries.
    pub fn clear(&self) {
        self.inner.lock().expect("lru lock").clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_basic() {
        let c: LruCache<&str, i32> = LruCache::new(3);
        c.put("a", 1);
        c.put("b", 2);
        assert_eq!(c.get(&"a"), Some(1));
        assert_eq!(c.get(&"b"), Some(2));
        assert_eq!(c.get(&"c"), None);
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn capacity_evicts_lru() {
        let c: LruCache<&str, i32> = LruCache::new(2);
        c.put("a", 1);
        c.put("b", 2);
        // access "a" so "b" becomes LRU
        assert_eq!(c.get(&"a"), Some(1));
        c.put("c", 3); // evicts "b"
        assert_eq!(c.get(&"a"), Some(1));
        assert_eq!(c.get(&"b"), None);
        assert_eq!(c.get(&"c"), Some(3));
    }

    #[test]
    fn peek_does_not_update_recency() {
        let c: LruCache<&str, i32> = LruCache::new(2);
        c.put("a", 1);
        c.put("b", 2);
        // peek "a" without touching recency => "a" is still LRU
        assert_eq!(c.peek(&"a"), Some(1));
        c.put("c", 3); // evicts "a"
        assert_eq!(c.get(&"a"), None);
        assert_eq!(c.get(&"b"), Some(2));
    }

    #[test]
    fn pop_and_clear() {
        let c: LruCache<&str, i32> = LruCache::new(3);
        c.put("a", 1);
        assert_eq!(c.pop(&"a"), Some(1));
        assert!(c.is_empty());
        c.put("b", 2);
        c.clear();
        assert!(c.is_empty());
    }

    #[test]
    fn resize_shrinks_evicting_lru() {
        let c: LruCache<i32, i32> = LruCache::new(3);
        c.put(1, 10);
        c.put(2, 20);
        c.put(3, 30);
        c.resize(1);
        assert_eq!(c.len(), 1);
        // only the most recently used survives
        assert_eq!(c.get(&3), Some(30));
        assert_eq!(c.get(&1), None);
    }

    #[test]
    fn capacity_zero_treated_as_one() {
        let c: LruCache<i32, i32> = LruCache::new(0);
        c.put(1, 10);
        c.put(2, 20);
        assert_eq!(c.get(&1), None);
        assert_eq!(c.get(&2), Some(20));
    }
}
