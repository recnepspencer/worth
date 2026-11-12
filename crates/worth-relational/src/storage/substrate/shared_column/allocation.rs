use super::{
    node::{ColumnNode, ColumnStorage},
    SharedColumn,
};
use crate::storage::substrate::{
    StorageAllocation, StorageAllocationVisitor, StoredValueObservation,
};

impl<T: super::StoredValue> SharedColumn<T> {
    pub(crate) fn visit_allocations(
        &self,
        ancestor_unique: bool,
        visitor: &mut dyn StorageAllocationVisitor,
        value: &mut dyn FnMut(&T::Value, StoredValueObservation, &mut dyn StorageAllocationVisitor),
    ) {
        if let Some(default) = &self.default {
            value(
                default.value(),
                StoredValueObservation::new(default, ancestor_unique),
                visitor,
            );
        }
        if let Some(root) = &self.root {
            visit_node(root, ancestor_unique, visitor, value);
        }
    }
}

fn visit_node<T: super::StoredValue>(
    node: &StorageAllocation<ColumnNode<T>>,
    ancestor_unique: bool,
    visitor: &mut dyn StorageAllocationVisitor,
    value: &mut dyn FnMut(&T::Value, StoredValueObservation, &mut dyn StorageAllocationVisitor),
) {
    let mut allocation = node.observation(ancestor_unique);
    if let ColumnStorage::Page(values) = &node.storage {
        allocation.bytes += (values.capacity() * std::mem::size_of::<Option<T>>()) as u64;
    }
    if !visitor.visit(allocation) {
        return;
    }
    match &node.storage {
        ColumnStorage::Page(values) => {
            for entry in values.iter().flatten() {
                value(
                    entry.value(),
                    StoredValueObservation::new(entry, allocation.unique),
                    visitor,
                );
            }
        }
        ColumnStorage::Branch(children) => {
            for child in children.iter().flatten() {
                visit_node(child, allocation.unique, visitor, value);
            }
        }
    }
}
