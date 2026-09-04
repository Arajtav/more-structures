//! Like a map but for checking whether 2 elements are in the same group.

use std::{borrow::Borrow, collections::HashMap, hash::Hash};

#[derive(PartialEq, Eq, Clone, Copy, Default)]
struct Group(usize);

impl Group {
    const fn increment(&mut self) -> Group {
        let previous = *self;
        self.0 = self
            .0
            .checked_add(1)
            .expect("GroupSet cannot have more than usize::max groups");
        previous
    }
}

/// For grouping items.
pub struct GroupSet<T: Eq + Hash> {
    items: HashMap<T, Group>,
    free: Group,
}

impl<T: Eq + Hash> Default for GroupSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash> GroupSet<T> {
    /// Creates a new empty `GroupSet`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            free: Group::default(),
        }
    }

    /// Tries to create a new group with one item.
    /// Fails and returns Some(item) if the item already is in a group.
    pub fn new_group(&mut self, item: T) -> Option<T> {
        if self.contains(&item) {
            Some(item)
        } else {
            let group = self.free.increment();

            self.items.insert(item, group);
            None
        }
    }

    /// Returns `true` if the item already is in the group set.
    pub fn contains<Q>(&self, item: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.items.contains_key(item)
    }

    /// Groups 2 items, the first one has to already be in the group set.
    /// Returns Some(item) item if either the first one is not a part of any group or the new one is.
    pub fn group<Q>(&mut self, existing: &Q, item: T) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let Some(&group) = self.items.get(existing) else {
            return Some(item);
        };

        // Not doable actually.
        #[allow(clippy::map_entry)]
        if self.items.contains_key::<T>(&item) {
            Some(item)
        } else {
            self.items.insert(item, group);
            None
        }
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
        let mut gs = GroupSet::new();
        assert!(!gs.contains(&"a"));
        assert_eq!(gs.new_group("a"), None);
        assert!(gs.contains(&"a"));
        assert_eq!(gs.new_group("a"), Some("a"));
        assert!(!gs.is_same_group(&"a", &"b"));
        assert_eq!(gs.group(&"a", "b"), None);
        assert!(gs.is_same_group(&"a", &"b"));
        assert!(gs.remove(&"a"));
        assert!(!gs.is_same_group(&"a", &"b"));
        assert!(!gs.contains(&"a"));
    }

    #[test]
    fn group_of_3() {
        let mut gs = GroupSet::new();
        assert_eq!(gs.new_group("a"), None);
        assert_eq!(gs.group(&"a", "b"), None);
        assert_eq!(gs.group(&"a", "c"), None);
        assert!(gs.is_same_group(&"a", &"b"));
        assert!(gs.is_same_group(&"b", &"c"));
        assert!(gs.is_same_group(&"c", &"a"));
    }
}
