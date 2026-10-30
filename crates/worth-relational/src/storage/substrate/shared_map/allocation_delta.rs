//! Allocation identity, not key/value equality, decides reuse across rotations.
use super::{node::MapNode, SharedMap};
use crate::storage::substrate::{
    StorageAllocation, StorageAllocationObservation, StorageAllocationVisitor,
};

impl<K: Ord + Copy, V: Clone> SharedMap<K, V> {
    pub(crate) fn visit_new_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn StorageAllocationVisitor,
        value: &mut dyn FnMut(
            &V,
            Option<&V>,
            StorageAllocationObservation,
            &mut dyn StorageAllocationVisitor,
        ),
    ) {
        if let Some(root) = &self.root {
            visit_node(root, previous, visitor, value);
        }
    }

    pub(super) fn node_for(&self, key: &K) -> Option<&StorageAllocation<MapNode<K, V>>> {
        let mut current = self.root.as_ref()?;
        loop {
            current = match key.cmp(&current.key) {
                std::cmp::Ordering::Less => current.left.as_ref()?,
                std::cmp::Ordering::Equal => return Some(current),
                std::cmp::Ordering::Greater => current.right.as_ref()?,
            };
        }
    }
}

fn visit_node<K: Ord + Copy, V: Clone>(
    node: &StorageAllocation<MapNode<K, V>>,
    previous: &SharedMap<K, V>,
    visitor: &mut dyn StorageAllocationVisitor,
    value: &mut dyn FnMut(
        &V,
        Option<&V>,
        StorageAllocationObservation,
        &mut dyn StorageAllocationVisitor,
    ),
) {
    let old = previous.node_for(&node.key);
    if old.is_some_and(|old| old.id() == node.id()) {
        return;
    }
    if !visitor.visit(node.observation(false)) {
        return;
    }
    if old.is_none_or(|old| old.value.id() != node.value.id()) {
        value(
            &node.value,
            old.map(|old| old.value.as_ref()),
            node.value.observation(false),
            visitor,
        );
    }
    for child in [&node.left, &node.right].into_iter().flatten() {
        visit_node(child, previous, visitor, value);
    }
}
