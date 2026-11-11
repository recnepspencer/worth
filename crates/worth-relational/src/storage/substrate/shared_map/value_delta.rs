//! Changed keys are selected by owner identity, independently of AVL shape.
use super::{node::MapNode, SharedMap};
use crate::storage::substrate::StorageAllocation;

impl<K: Ord + Copy, V: super::StoredValue> SharedMap<K, V> {
    pub(crate) fn changed_keys_since(&self, previous: &Self) -> std::collections::BTreeSet<K> {
        let mut keys = std::collections::BTreeSet::new();
        walk(self.root.as_ref(), previous, &mut keys);
        walk(previous.root.as_ref(), self, &mut keys);
        keys
    }
}

fn walk<K: Ord + Copy, V: super::StoredValue>(
    node: Option<&StorageAllocation<MapNode<K, V>>>,
    previous: &SharedMap<K, V>,
    keys: &mut std::collections::BTreeSet<K>,
) {
    let Some(node) = node else {
        return;
    };
    let old = previous.node_for(&node.key);
    if old.is_some_and(|old| old.id() == node.id()) {
        return;
    }
    if old.is_none_or(|old| !old.value.same_storage(&node.value)) {
        keys.insert(node.key);
    }
    walk(node.left.as_ref(), previous, keys);
    walk(node.right.as_ref(), previous, keys);
}

impl<K: Ord + Copy, V: super::StoredValue> SharedMap<K, V> {
    /// Cost: changed persistent paths; identical subtrees are pruned by owner.
    pub(crate) fn try_changed_keys_since<E>(
        &self,
        previous: &Self,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<std::collections::BTreeSet<K>, E> {
        let mut keys = std::collections::BTreeSet::new();
        paid_walk(self.root.as_ref(), previous, &mut keys, prepare)?;
        paid_walk(previous.root.as_ref(), self, &mut keys, prepare)?;
        Ok(keys)
    }
}
fn paid_walk<K: Ord + Copy, V: super::StoredValue, E>(
    node: Option<&StorageAllocation<MapNode<K, V>>>,
    previous: &SharedMap<K, V>,
    keys: &mut std::collections::BTreeSet<K>,
    prepare: &mut dyn FnMut(u64, u64) -> Result<(), E>,
) -> Result<(), E> {
    let Some(node) = node else { return Ok(()) };
    prepare(previous.navigation_work_bound() as u64 + 1, 0)?;
    let old = previous.node_for(&node.key);
    if old.is_some_and(|old| old.id() == node.id()) {
        return Ok(());
    }
    if old.is_none_or(|old| !old.value.same_storage(&node.value)) {
        prepare(32, 1024)?;
        keys.insert(node.key);
    }
    paid_walk(node.left.as_ref(), previous, keys, prepare)?;
    paid_walk(node.right.as_ref(), previous, keys, prepare)
}
