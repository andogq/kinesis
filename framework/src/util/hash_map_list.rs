use std::{collections::HashMap, hash::Hash};

/// Simple wrapper to make functionality easier for interfacing with `HashMap<K, Vec<T>>`. Handles
/// the logic surrounding initialising a new [Vec] when a new value is inserted with a unique key.
/// Allows for key collisions to be handled by appending the value to a [Vec], rather than over
/// writing it.
pub struct HashMapList<K, V>(HashMap<K, Vec<V>>);

impl<K, V> HashMapList<K, V>
where
    K: Eq + PartialEq + Hash,
{
    /// Creates an empty [HashMapList].
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Retrieves a list of values by key. Will return [None] if there are no items that match the
    /// provided key.
    pub fn get(&self, k: &K) -> Option<&[V]> {
        self.0.get(k).map(|v| v.as_slice())
    }

    /// Inserts a value with a given key into the collection. If there is no existing [Vec] for the
    /// key, an empty one will be initialised before the value is inserted.
    pub fn insert(&mut self, k: K, v: V) {
        self.0.entry(k).or_insert(Vec::new()).push(v);
    }
}
