//! Shared ordered backing whose actual new cells admit their own allocations.
mod edit;
mod iteration;
pub(crate) mod node;
use crate::{ChargedBytes, ExecutionAllocationDenial, ExecutionAllocationPolicy};
use node::{Entry, Link};

/// A refused edit leaves the original root and every grant intact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionOrderedMapDenial {
    Allocation(ExecutionAllocationDenial),
    CapacityOverflow,
}
impl From<ExecutionAllocationDenial> for ExecutionOrderedMapDenial {
    fn from(value: ExecutionAllocationDenial) -> Self {
        Self::Allocation(value)
    }
}

/// Immutable ordered backing. Cloning shares cells and their original grants;
/// replacement copies only an AVL path. This confers no source, session,
/// currentness or successful-result authority.
///
/// Inserted key/value clones must own no more heap than their `ChargedBytes`
/// bound. The bound admits their copies before cloning; shared child/entry
/// cells keep their own grants rather than funding an entire new root.
pub struct ExecutionOrderedMap<K, V> {
    root: Link<K, V>,
}
impl<K, V> Default for ExecutionOrderedMap<K, V> {
    fn default() -> Self {
        Self { root: None }
    }
}
impl<K, V> Clone for ExecutionOrderedMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
        }
    }
}
impl<K, V> ExecutionOrderedMap<K, V> {
    pub fn len(&self) -> usize {
        self.root.as_ref().map_or(0, |n| n.count)
    }
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
    /// Complete reachable retention descriptor, never a new-allocation quote.
    pub fn retained_bytes(&self) -> u64 {
        self.root.as_ref().map_or(0, |n| n.bytes)
    }
    pub fn iter(&self) -> iteration::Iter<'_, K, V> {
        iteration::Iter::new(self.root.as_deref(), self.len())
    }
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &K> {
        self.iter().map(|(k, _)| k)
    }
    pub fn values(&self) -> impl ExactSizeIterator<Item = &V> {
        self.iter().map(|(_, v)| v)
    }
}
impl<K: Ord, V> ExecutionOrderedMap<K, V> {
    /// Canonical zero-based position, without enumerating preceding entries.
    pub fn rank_of(&self, key: &K) -> Option<usize> {
        let (mut next, mut before) = (self.root.as_deref(), 0_usize);
        while let Some(node) = next {
            let left = node.left.as_ref().map_or(0, |child| child.count);
            match key.cmp(&node.entry.key) {
                std::cmp::Ordering::Less => next = node.left.as_deref(),
                std::cmp::Ordering::Equal => return before.checked_add(left),
                std::cmp::Ordering::Greater => {
                    before = before.checked_add(left)?.checked_add(1)?;
                    next = node.right.as_deref();
                }
            }
        }
        None
    }
    /// Entry at a canonical position. Missing positions allocate nothing.
    pub fn entry_at(&self, mut rank: usize) -> Option<(&K, &V)> {
        let mut next = self.root.as_deref();
        while let Some(node) = next {
            let left = node.left.as_ref().map_or(0, |child| child.count);
            match rank.cmp(&left) {
                std::cmp::Ordering::Less => next = node.left.as_deref(),
                std::cmp::Ordering::Equal => return Some((&node.entry.key, &node.entry.value)),
                std::cmp::Ordering::Greater => {
                    rank -= left + 1;
                    next = node.right.as_deref();
                }
            }
        }
        None
    }
    /// First ordered entry at or above a key, without allocating a range.
    pub fn first_at_or_after(&self, key: &K) -> Option<(&K, &V)> {
        let (mut next, mut found) = (self.root.as_deref(), None);
        while let Some(node) = next {
            if node.entry.key < *key {
                next = node.right.as_deref();
            } else {
                found = Some((&node.entry.key, &node.entry.value));
                next = node.left.as_deref();
            }
        }
        found
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        self.get_key_value(key).map(|(_, v)| v)
    }
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        let mut next = self.root.as_deref();
        while let Some(node) = next {
            match key.cmp(&node.entry.key) {
                std::cmp::Ordering::Less => next = node.left.as_deref(),
                std::cmp::Ordering::Greater => next = node.right.as_deref(),
                std::cmp::Ordering::Equal => return Some((&node.entry.key, &node.entry.value)),
            }
        }
        None
    }
    /// O(log n) copied nodes, each reserved before allocation. Missing removal
    /// allocates nothing. No node retains the prior root or a grant chain.
    pub fn remove_in(
        &mut self,
        key: &K,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<bool, ExecutionOrderedMapDenial> {
        policy.check_live()?;
        if !self.contains_key(key) {
            return Ok(false);
        }
        let root = edit::remove(&self.root, key, policy)?;
        policy.check_live()?;
        self.root = root;
        Ok(true)
    }
}
impl<K: Ord + Clone + ChargedBytes, V: Clone + ChargedBytes> ExecutionOrderedMap<K, V> {
    /// Admits the actual new entry and copied path, not the complete retained
    /// descriptor. Cancellation or refusal publishes no partial root.
    pub fn insert_cloned_in(
        &mut self,
        key: &K,
        value: &V,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), ExecutionOrderedMapDenial> {
        policy.check_live()?;
        let entry = Entry::cloned(key, value, policy)?;
        let root = edit::insert(&self.root, entry, policy)?;
        policy.check_live()?;
        self.root = Some(root);
        Ok(())
    }
}
impl<K: Ord, V> std::ops::Index<&K> for ExecutionOrderedMap<K, V> {
    type Output = V;
    fn index(&self, key: &K) -> &V {
        self.get(key).expect("ordered key is present")
    }
}
impl<K: std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for ExecutionOrderedMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
impl<K: PartialEq, V: PartialEq> PartialEq for ExecutionOrderedMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}
impl<K: Eq, V: Eq> Eq for ExecutionOrderedMap<K, V> {}
impl<'a, K, V> IntoIterator for &'a ExecutionOrderedMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = iteration::Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
#[cfg(test)]
mod tests;
