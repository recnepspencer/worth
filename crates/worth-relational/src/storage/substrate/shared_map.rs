//! Ordered storage index with path-copying updates and borrowed traversal.
use super::StorageAllocation as Arc;

mod allocation;
mod allocation_delta;
mod iteration;
mod node;
#[cfg(test)]
mod tests;
mod traits;
mod value_delta;

pub(crate) use iteration::SharedMapIter;
use node::MapNode;

#[derive(Debug, Clone)]
pub(crate) struct SharedMap<K: Ord + Copy, V: Clone> {
    root: Option<Arc<MapNode<K, V>>>,
}

impl<K: Ord + Copy, V: Clone> SharedMap<K, V> {
    pub(crate) fn new() -> Self {
        Self { root: None }
    }
    pub(crate) fn len(&self) -> usize {
        self.root.as_ref().map_or(0, |root| root.len)
    }
    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        let mut current = self.root.as_deref();
        while let Some(node) = current {
            match key.cmp(&node.key) {
                std::cmp::Ordering::Less => current = node.left.as_deref(),
                std::cmp::Ordering::Equal => return Some(&node.value),
                std::cmp::Ordering::Greater => current = node.right.as_deref(),
            }
        }
        None
    }

    pub(crate) fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if !self.contains_key(key) {
            return None;
        }
        MapNode::get_mut(&mut self.root, key)
    }

    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub(crate) fn insert(&mut self, key: K, value: V) -> u64 {
        // Replacement never materializes the displaced shared payload.
        let mut copied_bytes = 0;
        MapNode::insert(&mut self.root, key, Arc::new(value), &mut copied_bytes);
        copied_bytes
    }

    /// Adopt only the selected source owner; all other destination keys retain
    /// their exact branch-local values. Missing source keys remove membership.
    pub(crate) fn copy_value_from(&mut self, key: K, source: &Self) -> u64 {
        let Some(source) = source.node_for(&key) else {
            return self.remove(&key).1;
        };
        if self
            .node_for(&key)
            .is_some_and(|current| current.value.id() == source.value.id())
        {
            return 0;
        }
        let mut copied_bytes = 0;
        MapNode::insert(&mut self.root, key, source.value.clone(), &mut copied_bytes);
        copied_bytes
    }

    pub(crate) fn remove(&mut self, key: &K) -> (bool, u64) {
        if !self.contains_key(key) {
            return (false, 0);
        }
        let mut copied_bytes = 0;
        let removed = MapNode::remove(&mut self.root, key, &mut copied_bytes).is_some();
        (removed, copied_bytes)
    }

    pub(crate) fn entry(&mut self, key: K) -> SharedMapEntry<'_, K, V> {
        SharedMapEntry { map: self, key }
    }

    pub(crate) fn iter(&self) -> SharedMapIter<'_, K, V> {
        SharedMapIter::new(self.root.as_deref())
    }
    pub(crate) fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(key, _)| key)
    }
    pub(crate) fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, value)| value)
    }

    /// Explicit full-index mutation used by cold checkpoint reconstruction.
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        let mut values = Vec::with_capacity(self.len());
        MapNode::collect_mut(&mut self.root, &mut values);
        values.into_iter()
    }

    pub(crate) fn last_key_value(&self) -> Option<(&K, &V)> {
        let mut node = self.root.as_deref()?;
        while let Some(right) = node.right.as_deref() {
            node = right;
        }
        Some((&node.key, &node.value))
    }

    pub(crate) fn allocation_bytes(&self) -> u64 {
        let per_entry = Arc::<MapNode<K, V>>::layout_bytes() + Arc::<V>::layout_bytes();
        (self.len() as u64).saturating_mul(per_entry)
    }
}

pub(crate) struct SharedMapEntry<'a, K: Ord + Copy, V: Clone> {
    map: &'a mut SharedMap<K, V>,
    key: K,
}

impl<'a, K: Ord + Copy, V: Clone> SharedMapEntry<'a, K, V> {
    pub(crate) fn or_insert_with(self, value: impl FnOnce() -> V) -> &'a mut V {
        if !self.map.contains_key(&self.key) {
            self.map.insert(self.key, value());
        }
        self.map.get_mut(&self.key).expect("entry was installed")
    }
    pub(crate) fn or_insert(self, value: V) -> &'a mut V {
        self.or_insert_with(|| value)
    }
    pub(crate) fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with(V::default)
    }
}
