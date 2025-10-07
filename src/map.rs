use std::collections::hash_map::{Entry as HashMapEntry};
use std::collections::HashMap;
use std::hash::Hash;

use crate::tree::HashTree;
use crate::{SimHasher};

/// A map that combines exact key lookup with approximate hash matching using SimHash.
///
/// `SimMap` is designed to support workflows where items are indexed by their original
/// string keys while also being searchable by the similarity of their SimHash values.
/// It maintains a regular `HashMap` for fast exact lookups and a [`HashTree`] for
/// approximate matching within a configurable Hamming distance.
pub struct SimMap<K: AsRef<str> + Eq + Hash, T> {
    items: HashMap<K, T>,
    tree: HashTree<T>,
    hasher: SimHasher,
    pub max_dist: u8,
}

impl<K: AsRef<str> + Eq + Hash, T> SimMap<K, T> {
    pub fn new(hasher: SimHasher, max_dist: u8) -> Self {
        Self {
            items: HashMap::new(),
            tree: HashTree::new(),
            hasher,
            max_dist,
        }
    }

    pub fn with_capacity(hasher: SimHasher, max_dist: u8, capacity: usize) -> Self {
        Self {
            items: HashMap::with_capacity(capacity),
            tree: HashTree::new(),
            hasher,
            max_dist,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &T)> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut T)> {
        self.items.iter_mut()
    }

    pub fn hasher(&self) -> &SimHasher {
        &self.hasher
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.items.contains_key(key)
    }

    pub fn get(&self, key: &K) -> Option<&T> {
        self.items.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut T> {
        self.items.get_mut(key)
    }

    pub fn maybe_insert_close_or(&mut self, key: K, f: impl FnOnce() -> T) -> &T
    where
        K: AsRef<[u8]>,
        T: Clone,
    {
        match self.items.entry(key) {
            HashMapEntry::Occupied(entry) => entry.into_mut(),
            HashMapEntry::Vacant(entry) => {
                let hash = self.hasher.hash(entry.key());
                let value = if let Some(value) = self.tree.contains(hash, self.max_dist) {
                    value.clone()
                } else {
                    let value = f();
                    self.tree.add(hash, value.clone());
                    value
                };
                entry.insert(value)
            }
        }
    }
    
}