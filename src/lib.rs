use std::collections::HashMap;

/// This is an example doc test
///
/// Key/value are stores in-memory and not is disk
///
/// Example:
///
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key".to_owned(), "value".to_owned());
/// let val = store.get("key".to_owned());
/// assert_eq!(val, Some("value".to_owned()))
/// ```
#[derive(Default)]
pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    /// new method would create a new instance of `KvStore`
    pub fn new() -> KvStore {
        KvStore {
            map: HashMap::new(),
        }
    }

    /// assing a value to a specific key
    ///
    /// if the key already exists the value is overwritten
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    /// gets the value of a specific key if there is some or none.
    pub fn get(&mut self, key: String) -> Option<String> {
        match self.map.get(&key) {
            Some(a) => Some(a.to_owned()),
            None => None,
        }
    }

    /// removes the the key and the associated value.
    pub fn remove(&mut self, key: String) -> Option<String> {
        self.map.remove(&key)
    }
}
