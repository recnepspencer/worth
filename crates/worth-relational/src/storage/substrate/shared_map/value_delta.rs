//! Changed keys are selected by owner identity, independently of AVL shape.
use super::{node::MapNode, SharedMap};
use crate::storage::substrate::StorageAllocation;

impl<K: Ord + Copy, V: Clone> SharedMap<K, V> {
    pub(crate) fn changed_keys_since(&self, previous: &Self) -> std::collections::BTreeSet<K> {
        let mut keys = std::collections::BTreeSet::new();
        walk(self.root.as_ref(), previous, &mut keys);
        walk(previous.root.as_ref(), self, &mut keys);
        keys
    }
}

fn walk<K: Ord + Copy, V: Clone>(
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
    if old.is_none_or(|old| old.value.id() != node.value.id()) {
        keys.insert(node.key);
    }
    walk(node.left.as_ref(), previous, keys);
    walk(node.right.as_ref(), previous, keys);
}
