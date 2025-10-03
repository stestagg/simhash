use std::collections::hash_map::{Entry as HashMapEntry, OccupiedEntry, VacantEntry};
use std::collections::HashMap;
use std::ptr::NonNull;

use crate::tree::HashTree;
use crate::SimHasher;

/// A map that combines exact key lookup with approximate hash matching using SimHash.
///
/// `SimMap` is designed to support workflows where items are indexed by their original
/// string keys while also being searchable by the similarity of their SimHash values.
/// It maintains a regular `HashMap` for fast exact lookups and a [`HashTree`] for
/// approximate matching within a configurable Hamming distance.
pub struct SimMap<T> {
    items: HashMap<String, T>,
    tree: HashTree<T>,
    hasher: SimHasher,
    max_dist: u8,
}

impl<T> SimMap<T> {
    /// Creates a new, empty `SimMap` with the provided [`SimHasher`] and maximum
    /// allowed Hamming distance for approximate matches.
    pub fn new(hasher: SimHasher, max_dist: u8) -> Self {
        Self {
            items: HashMap::new(),
            tree: HashTree::new(),
            hasher,
            max_dist,
        }
    }

    /// Creates a new `SimMap` with a preallocated capacity for the underlying
    /// item map.
    pub fn with_capacity(hasher: SimHasher, max_dist: u8, capacity: usize) -> Self {
        Self {
            items: HashMap::with_capacity(capacity),
            tree: HashTree::new(),
            hasher,
            max_dist,
        }
    }

    /// Returns the number of items stored in the exact key map.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if the map contains no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns an iterator over the stored key/value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &T)> {
        self.items.iter().map(|(key, value)| (key.as_str(), value))
    }

    /// Returns a mutable iterator over the stored key/value pairs.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut T)> {
        self.items
            .iter_mut()
            .map(|(key, value)| (key.as_str(), value))
    }

    /// Returns a reference to the underlying [`SimHasher`].
    pub fn hasher(&self) -> &SimHasher {
        &self.hasher
    }

    /// Computes the SimHash for the provided bytes using the configured hasher.
    pub fn hash_bytes(&self, bytes: &[u8]) -> u64 {
        self.hasher.hash_bytes(bytes)
    }

    /// Returns the maximum Hamming distance allowed for approximate matches.
    pub fn max_distance(&self) -> u8 {
        self.max_dist
    }

    /// Updates the maximum Hamming distance allowed for approximate matches.
    pub fn set_max_distance(&mut self, max_dist: u8) {
        self.max_dist = max_dist;
    }

    /// Returns `true` if the exact key map contains `key`.
    pub fn contains_key(&self, key: &str) -> bool {
        self.items.contains_key(key)
    }

    /// Returns the value associated with `key` if it exists in the exact key map.
    pub fn get(&self, key: &str) -> Option<&T> {
        self.items.get(key)
    }

    /// Returns a mutable reference to the value associated with `key` if it exists in the exact key map.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut T> {
        self.items.get_mut(key)
    }

    /// Inserts a key/value pair into the exact key map, returning the previous value if present.
    pub fn insert_key<K>(&mut self, key: K, value: T) -> Option<T>
    where
        K: Into<String>,
    {
        self.items.insert(key.into(), value)
    }

    /// Removes the value associated with `key` from the exact key map.
    pub fn remove_key(&mut self, key: &str) -> Option<T> {
        self.items.remove(key)
    }

    /// Returns `true` if the hash tree contains a value within the configured maximum distance of `hash`.
    pub fn contains_hash(&self, hash: u64) -> bool {
        self.get_hash(hash).is_some()
    }

    /// Searches the hash tree for a value whose hash is within the configured maximum distance of `hash`.
    pub fn get_hash(&self, hash: u64) -> Option<&T> {
        self.tree.contains(hash, self.max_dist as usize)
    }

    /// Searches the hash tree for a value within the provided maximum distance of `hash`.
    pub fn get_hash_within(&self, hash: u64, max_dist: u8) -> Option<&T> {
        self.tree.contains(hash, max_dist as usize)
    }

    /// Inserts a hash/value pair into the approximate match tree.
    pub fn insert_hash(&mut self, hash: u64, value: T) {
        self.tree.add(hash, value);
    }

    /// Returns the number of values stored in the hash tree.
    pub fn tree_len(&self) -> usize {
        self.tree.len()
    }

    /// Creates an entry for `key`, similar to [`HashMap::entry`].
    ///
    /// If the key already exists, an [`SimOccupiedEntry`] is returned. Otherwise a [`SimVacantEntry`]
    /// is returned which can optionally interact with the approximate match tree before inserting.
    pub fn entry<K>(&mut self, key: K) -> SimEntry<'_, T>
    where
        K: Into<String>,
    {
        let max_dist = self.max_dist;
        let tree_ptr = NonNull::from(&mut self.tree);

        match self.items.entry(key.into()) {
            HashMapEntry::Occupied(entry) => SimEntry::Occupied(SimOccupiedEntry { entry }),
            HashMapEntry::Vacant(entry) => SimEntry::Vacant(SimVacantEntry {
                entry,
                tree: tree_ptr,
                max_dist,
            }),
        }
    }
}

/// Entry API for [`SimMap`], mirroring the standard library's [`HashMap::entry`].
pub enum SimEntry<'a, T> {
    Occupied(SimOccupiedEntry<'a, T>),
    Vacant(SimVacantEntry<'a, T>),
}

impl<'a, T> SimEntry<'a, T> {
    /// Returns the key for this entry.
    pub fn key(&self) -> &str {
        match self {
            SimEntry::Occupied(entry) => entry.key(),
            SimEntry::Vacant(entry) => entry.key(),
        }
    }

    /// Inserts a value into the map if the entry is vacant, returning a mutable reference to the value.
    ///
    /// This mirrors [`HashMap::Entry::or_insert`], but it does not interact with the approximate match tree.
    pub fn or_insert(self, default: T) -> &'a mut T {
        match self {
            SimEntry::Occupied(entry) => entry.into_mut(),
            SimEntry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Inserts a value produced by `default` if the entry is vacant, returning a mutable reference to the value.
    pub fn or_insert_with<F>(self, default: F) -> &'a mut T
    where
        F: FnOnce() -> T,
    {
        match self {
            SimEntry::Occupied(entry) => entry.into_mut(),
            SimEntry::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Applies `f` to the contained value if the entry is occupied.
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut T),
    {
        match self {
            SimEntry::Occupied(mut entry) => {
                f(entry.get_mut());
                SimEntry::Occupied(entry)
            }
            other => other,
        }
    }
}

/// An occupied entry in a [`SimMap`].
pub struct SimOccupiedEntry<'a, T> {
    entry: OccupiedEntry<'a, String, T>,
}

impl<'a, T> SimOccupiedEntry<'a, T> {
    /// Returns the key associated with this occupied entry.
    pub fn key(&self) -> &str {
        self.entry.key()
    }

    /// Returns a reference to the stored value.
    pub fn get(&self) -> &T {
        self.entry.get()
    }

    /// Returns a mutable reference to the stored value.
    pub fn get_mut(&mut self) -> &mut T {
        self.entry.get_mut()
    }

    /// Converts the occupied entry into a mutable reference.
    pub fn into_mut(self) -> &'a mut T {
        self.entry.into_mut()
    }
}

/// A vacant entry in a [`SimMap`].
pub struct SimVacantEntry<'a, T> {
    entry: VacantEntry<'a, String, T>,
    tree: NonNull<HashTree<T>>,
    max_dist: u8,
}

impl<'a, T> SimVacantEntry<'a, T> {
    /// Returns the key associated with this vacant entry.
    pub fn key(&self) -> &str {
        self.entry.key()
    }

    /// Checks whether the approximate hash tree contains a value within the default maximum distance of `hash`.
    pub fn tree_contains(&self, hash: u64) -> Option<&T> {
        self.tree_contains_within(hash, self.max_dist)
    }

    /// Checks whether the approximate hash tree contains a value within `max_dist` of `hash`.
    pub fn tree_contains_within(&self, hash: u64, max_dist: u8) -> Option<&T> {
        unsafe { self.tree.as_ref().contains(hash, max_dist as usize) }
    }

    /// Inserts `value` into the approximate hash tree without affecting the key map.
    pub fn insert_into_tree(&mut self, hash: u64, value: T) {
        unsafe { self.tree.as_mut().add(hash, value) }
    }

    /// Inserts `value` into the key map and returns a mutable reference to it.
    pub fn insert(self, value: T) -> &'a mut T {
        self.entry.insert(value)
    }

    /// Inserts a value into the key map and simultaneously adds a value to the hash tree.
    ///
    /// The `tree_value` can be derived from the inserted map value using `to_tree`, allowing callers to
    /// avoid cloning when possible (e.g. by wrapping the stored value in `Arc`).
    pub fn insert_with_tree_from<F>(mut self, hash: u64, value: T, to_tree: F) -> &'a mut T
    where
        F: FnOnce(&mut T) -> T,
    {
        let reference = self.entry.insert(value);
        let tree_value = to_tree(reference);
        unsafe { self.tree.as_mut().add(hash, tree_value) };
        reference
    }

    /// Inserts `map_value` into the key map and `tree_value` into the hash tree.
    pub fn insert_with_tree(mut self, hash: u64, map_value: T, tree_value: T) -> &'a mut T {
        unsafe { self.tree.as_mut().add(hash, tree_value) };
        self.entry.insert(map_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_hasher() -> SimHasher {
        SimHasher::new(crate::hash::sip_::hash_fn, |bytes: &[u8]| {
            Box::new(bytes.chunks(4).map(|chunk| chunk.into()))
        })
    }

    #[test]
    fn map_supports_basic_operations() {
        let hasher = test_hasher();
        let mut map = SimMap::new(hasher, 3);

        assert!(map.is_empty());
        assert!(!map.contains_key("hello"));

        let hash = map.hash_bytes(b"hello");

        match map.entry("hello") {
            SimEntry::Occupied(_) => panic!("entry should be vacant"),
            SimEntry::Vacant(entry) => {
                assert!(entry.tree_contains(hash).is_none());
                entry.insert_with_tree(hash, 1u32, 1u32);
            }
        }

        assert_eq!(map.len(), 1);
        assert_eq!(map.tree_len(), 1);
        assert!(map.contains_key("hello"));
        assert!(map.contains_hash(hash));
        assert_eq!(map.get("hello"), Some(&1));
        assert_eq!(map.get_hash(hash), Some(&1));
    }

    #[test]
    fn insert_with_tree_from_allows_reusing_value() {
        use std::sync::Arc;

        let hasher = test_hasher();
        let mut map = SimMap::new(hasher, 3);
        let hash = map.hash_bytes(b"world");

        match map.entry("world") {
            SimEntry::Occupied(_) => panic!("entry should be vacant"),
            SimEntry::Vacant(entry) => {
                let value = Arc::new(String::from("value"));
                entry.insert_with_tree_from(hash, value, |stored| stored.clone());
            }
        }

        let tree_value = map.get_hash(hash).expect("value should be stored in tree");
        assert_eq!(tree_value.as_str(), "value");

        let key_value = map.get("world").expect("value should be stored in map");
        assert_eq!(key_value.as_str(), "value");
    }
}
