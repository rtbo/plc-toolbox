
/// A Map optimized for a small number of entries
#[derive(Debug, Clone)]
pub struct SmallMap<K, V> {
    entries: Vec<(K, V)>,
}

impl<K: PartialEq, V> SmallMap<K, V> {
    /// Creates a new `SmallMap` with initial capacity
    pub fn with_capacity(cap: usize) -> Self {
        Self { entries: Vec::with_capacity(cap) }
    }

    /// Inserts a key-value pair into the map. If the key already exists, its value is updated.
    pub fn insert(&mut self, key: K, value: V) {
        for entry in &mut self.entries {
            if entry.0 == key {
                entry.1 = value;
                return;
            }
        }
        self.entries.push((key, value));
    }

    /// Removes the key from the map, returning the value if it existed.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(pos) = self.entries.iter().position(|(k, _)| k == key) {
            Some(self.entries.swap_remove(pos).1)
        } else {
            None
        }
    }
}
