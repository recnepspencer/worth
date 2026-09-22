use super::{node::MapNode, SharedMap};
use crate::storage::substrate::{
    StorageAllocation, StorageAllocationObservation, StorageAllocationVisitor,
};

impl<K: Ord + Copy, V: Clone> SharedMap<K, V> {
    pub(crate) fn visit_allocations(
        &self,
        ancestor_unique: bool,
        visitor: &mut dyn StorageAllocationVisitor,
        value: &mut dyn FnMut(&V, StorageAllocationObservation, &mut dyn StorageAllocationVisitor),
    ) {
        if let Some(root) = &self.root {
            visit_node(root, ancestor_unique, visitor, value);
        }
    }
}

fn visit_node<K: Ord + Copy, V: Clone>(
    node: &StorageAllocation<MapNode<K, V>>,
    ancestor_unique: bool,
    visitor: &mut dyn StorageAllocationVisitor,
    value: &mut dyn FnMut(&V, StorageAllocationObservation, &mut dyn StorageAllocationVisitor),
) {
    let allocation = node.observation(ancestor_unique);
    if !visitor.visit(allocation) {
        return;
    }
    value(
        &node.value,
        node.value.observation(allocation.unique),
        visitor,
    );
    for child in [&node.left, &node.right].into_iter().flatten() {
        visit_node(child, allocation.unique, visitor, value);
    }
}
