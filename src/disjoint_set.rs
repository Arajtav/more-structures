//! Disjoint set implementation with a `HashMap`.

use std::{borrow::Borrow, collections::HashMap, hash::Hash};

#[derive(PartialEq, Eq, Clone, Copy)]
struct Group(usize);

impl Default for Group {
    fn default() -> Self {
        Self(usize::MAX)
    }
}

impl Group {
    const fn increment(&mut self) -> Group {
        self.0 = self.0.wrapping_add(1);
        assert!(
            self.0 != usize::MAX,
            "GroupSet cannot have more than usize::max groups"
        );
        *self
    }
}

/// Disjoint set.
pub struct DisjointSet<T: Eq + Hash> {
    items: HashMap<T, Group>,
    free: Group,
}

impl<T: Eq + Hash> Default for DisjointSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash> DisjointSet<T> {
    /// Creates a new empty `DisjoinSet`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            free: Group::default(),
        }
    }

    /// Inserts an item. Does nothing if the item already exists.
    pub fn insert(&mut self, item: T) {
        self.items.entry(item).or_insert(self.free.increment());
    }

    /// Returns `true` if the item already is in the group set.
    pub fn contains<Q>(&self, item: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.items.contains_key(item)
    }

    /// Merges 2 groups. Fails and returns `false` if either item was not inserted before.
    pub fn merge<Q>(&mut self, first: &Q, second: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let (Some(&group), Some(&old_group)) = (self.items.get(first), self.items.get(second))
        else {
            return false;
        };

        for item in self.items.values_mut() {
            if *item == old_group {
                *item = group;
            }
        }

        true
    }

    /// Tries to remove an item and returns `true` if it succeeds.
    pub fn remove<Q>(&mut self, key: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.items.remove(key).is_some()
    }

    /// Check whether 2 items are in the same group.
    pub fn is_same_group<Q>(&self, a: &Q, b: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let a = self.items.get(a);
        let b = self.items.get(b);

        a.is_some() && a == b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn big_test() {
        let mut gs = DisjointSet::new();
        assert!(!gs.contains(&"a"));
        gs.insert("a");
        assert!(gs.contains(&"a"));
        assert!(!gs.is_same_group(&"a", &"b"));
        assert!(!gs.merge(&"a", &"b"));
        gs.insert("b");
        assert!(!gs.is_same_group(&"a", &"b"));
        assert!(gs.merge(&"a", &"b"));
        assert!(gs.is_same_group(&"a", &"b"));
        gs.insert("a");
        assert!(gs.is_same_group(&"a", &"b"));
        assert!(gs.remove(&"a"));
        assert!(!gs.is_same_group(&"a", &"b"));
        assert!(!gs.contains(&"a"));
    }

    #[test]
    fn merge() {
        let mut gs = DisjointSet::new();
        gs.insert("a");
        gs.insert("b");
        gs.insert("c");
        assert!(!gs.is_same_group(&"a", &"b"));
        assert!(!gs.is_same_group(&"b", &"c"));
        assert!(!gs.is_same_group(&"c", &"a"));
        assert!(gs.merge("a", "b"));
        assert!(gs.is_same_group(&"a", &"b"));
        assert!(!gs.is_same_group(&"b", &"c"));
        assert!(!gs.is_same_group(&"c", &"a"));
        assert!(gs.merge("b", "c"));
        assert!(gs.is_same_group(&"a", &"b"));
        assert!(gs.is_same_group(&"b", &"c"));
        assert!(gs.is_same_group(&"c", &"a"));
    }
}
