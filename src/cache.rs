use std::{any::Any, collections::HashMap, sync::RwLock};

/// A simple thread-safe in-memory key/value cache.
///
/// Values are stored as `Box<dyn Any + Send + Sync>`; callers must request
/// them back with the exact concrete type used on insertion.
pub struct Cache {
    store: RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            store: RwLock::new(HashMap::new()),
        }
    }

    /// Insert (or overwrite) a value.
    pub fn set<V>(&self, key: &str, value: V)
    where
        V: Any + Send + Sync + Clone,
    {
        if let Ok(mut map) = self.store.write() {
            map.insert(key.to_string(), Box::new(value));
        }
    }

    /// Fetch a cloned value by key. Returns `None` if the key is missing or the
    /// requested type does not match the stored type.
    pub fn get<V>(&self, key: &str) -> Option<V>
    where
        V: Any + Send + Sync + Clone,
    {
        self.store
            .read()
            .ok()
            .and_then(|map| map.get(key).and_then(|v| v.downcast_ref::<V>().cloned()))
    }

    /// Remove a key. Returns `true` if the key was present.
    pub fn remove(&self, key: &str) -> bool {
        self.store
            .write()
            .is_ok_and(|mut map| map.remove(key).is_some())
    }

    /// Returns `true` if the cache contains the key.
    pub fn contains(&self, key: &str) -> bool {
        self.store.read().is_ok_and(|map| map.contains_key(key))
    }

    /// Number of entries currently held.
    pub fn len(&self) -> usize {
        self.store.read().map(|map| map.len()).unwrap_or(0)
    }

    /// Is the cache empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove all entries.
    pub fn clear(&self) {
        if let Ok(mut map) = self.store.write() {
            map.clear();
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_str() {
        let cache = Cache::new();
        cache.set("name", "iclings");
        assert_eq!(cache.get::<&str>("name"), Some("iclings"));
    }

    #[test]
    fn set_and_get_owned() {
        let cache = Cache::new();
        cache.set("n", 42i32);
        assert_eq!(cache.get::<i32>("n"), Some(42));
    }

    #[test]
    fn missing_key_is_none() {
        let cache = Cache::new();
        assert_eq!(cache.get::<i32>("nope"), None);
    }

    #[test]
    fn type_mismatch_returns_none() {
        let cache = Cache::new();
        cache.set("k", 1u32);
        assert_eq!(cache.get::<String>("k"), None);
    }

    #[test]
    fn overwrite_value() {
        let cache = Cache::new();
        cache.set("k", 1);
        cache.set("k", 2);
        assert_eq!(cache.get::<i32>("k"), Some(2));
    }

    #[test]
    fn remove_and_contains() {
        let cache = Cache::new();
        cache.set("k", 1);
        assert!(cache.contains("k"));
        assert!(cache.remove("k"));
        assert!(!cache.contains("k"));
        assert!(!cache.remove("k"));
    }

    #[test]
    fn len_and_clear() {
        let cache = Cache::new();
        assert!(cache.is_empty());
        cache.set("a", 1);
        cache.set("b", 2);
        assert_eq!(cache.len(), 2);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn default_creates_empty() {
        let cache = Cache::default();
        assert!(cache.is_empty());
    }
}
