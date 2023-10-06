use std::collections::HashMap;

/// A simple key-value store.
#[derive(Default)]
pub struct KvStore {
    store: HashMap<String, String>,
}

impl KvStore {
    /// Creates a new `KvStore`.
    ///
    /// # Example
    ///
    /// ```
    /// use kvs::KvStore;
    ///
    /// let store = KvStore::new();
    /// ```
    pub fn new() -> Self {
        KvStore {
            store: HashMap::new(),
        }
    }

    /// Sets a key-value pair in the store.
    ///
    /// If the key already exists, its value is updated.
    ///
    /// # Example
    ///
    /// ```
    /// use kvs::KvStore;
    ///
    /// let mut store = KvStore::new();
    /// store.set("name".to_string(), "John".to_string());
    /// ```
    pub fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }

    /// Retrieves the value associated with a key from the store.
    ///
    /// Returns `Some(value)` if the key exists, or `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use kvs::KvStore;
    ///
    /// let mut store = KvStore::new();
    /// store.set("name".to_string(), "John".to_string());
    ///
    /// assert_eq!(store.get("name".to_string()), Some("John".to_string()));
    /// assert_eq!(store.get("age".to_string()), None);
    /// ```
    pub fn get(&self, key: String) -> Option<String> {
        self.store.get(&key).cloned()
    }

    /// Removes a key-value pair from the store.
    ///
    /// If the key does not exist, no action is taken.
    ///
    /// # Example
    ///
    /// ```
    /// use kvs::KvStore;
    ///
    /// let mut store = KvStore::new();
    /// store.set("name".to_string(), "John".to_string());
    /// store.remove("name".to_string());
    ///
    /// assert_eq!(store.get("name".to_string()), None);
    /// ```
    pub fn remove(&mut self, key: String) {
        self.store.remove(&key);
    }
}
