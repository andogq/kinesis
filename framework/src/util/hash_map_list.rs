use std::{
    collections::{hash_map::Keys, HashMap},
    hash::Hash,
};

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

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter::new(&self.0)
    }
}

impl<K, V> IntoIterator for HashMapList<K, V>
where
    K: Eq + PartialEq + Hash + Clone,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0)
    }
}
impl<'a, K, V> IntoIterator for &'a HashMapList<K, V>
where
    K: Eq + PartialEq + Hash + Clone,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

pub struct IntoIter<K, V>(HashMap<K, Vec<V>>);
impl<K, V> Iterator for IntoIter<K, V>
where
    K: Eq + PartialEq + Hash + Clone,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        // Pick a key to work with
        let key = self.0.keys().next()?.clone();

        // Pull the associated list for that key
        let list = self.0.get_mut(&key)?;

        // Remove a value from the list
        let value = list.pop()?;

        // If the list is empty, remove it from the hash map
        if list.is_empty() {
            self.0.remove(&key);
        }

        Some((key, value))
    }
}

pub struct Iter<'a, K, V> {
    collection: &'a HashMap<K, Vec<V>>,
    key_iter: Keys<'a, K, Vec<V>>,
    current: Option<(&'a K, std::slice::Iter<'a, V>)>,
}
impl<'a, K, V> Iter<'a, K, V> {
    pub fn new(collection: &'a HashMap<K, Vec<V>>) -> Self {
        Self {
            collection,
            key_iter: collection.keys(),
            current: None,
        }
    }
}
impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Eq + PartialEq + Hash,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take().or_else(|| {
            let key = self.key_iter.next()?;
            Some((key, self.collection.get(key)?.iter()))
        });

        if let Some((key, mut values)) = current {
            let value = values
                .next()
                .map(|value| (key, value))
                .or_else(|| self.next());

            self.current = Some((key, values));

            value
        } else {
            None
        }
    }
}
