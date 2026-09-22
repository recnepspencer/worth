//! Compare retained owners by exact allocation identity, pruning reused subtrees.
use super::{
    node::{ColumnNode, ColumnStorage},
    SharedColumn, PAGE_LEN,
};
use crate::storage::substrate::{
    StorageAllocation, StorageAllocationObservation, StorageAllocationVisitor,
};

impl<T: Clone> SharedColumn<T> {
    pub(crate) fn visit_new_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn StorageAllocationVisitor,
        value: &mut dyn FnMut(
            &T,
            Option<&T>,
            StorageAllocationObservation,
            &mut dyn StorageAllocationVisitor,
        ),
    ) {
        if let Some(default) = &self.default {
            if previous
                .default
                .as_ref()
                .is_none_or(|old| old.id() != default.id())
            {
                value(
                    default,
                    previous.default.as_deref(),
                    default.observation(false),
                    visitor,
                );
            }
        }
        if let Some(root) = &self.root {
            visit_node(root, self.height, 0, previous, visitor, value);
        }
    }

    pub(super) fn node_at(
        &self,
        index: usize,
        height: usize,
    ) -> Option<&StorageAllocation<ColumnNode<T>>> {
        if height > self.height || index / PAGE_LEN >= 1usize << self.height {
            return None;
        }
        let mut current = self.root.as_ref()?;
        let mut remaining = self.height;
        while remaining > height {
            let ColumnStorage::Branch(children) = &current.storage else {
                return None;
            };
            current = children[(index / PAGE_LEN >> (remaining - 1)) & 1].as_ref()?;
            remaining -= 1;
        }
        Some(current)
    }
}

fn visit_node<T: Clone>(
    node: &StorageAllocation<ColumnNode<T>>,
    height: usize,
    start: usize,
    previous: &SharedColumn<T>,
    visitor: &mut dyn StorageAllocationVisitor,
    value: &mut dyn FnMut(
        &T,
        Option<&T>,
        StorageAllocationObservation,
        &mut dyn StorageAllocationVisitor,
    ),
) {
    if previous
        .node_at(start, height)
        .is_some_and(|old| old.id() == node.id())
    {
        return;
    }
    let mut allocation = node.observation(false);
    if let ColumnStorage::Page(values) = &node.storage {
        allocation.bytes +=
            (values.capacity() * std::mem::size_of::<Option<StorageAllocation<T>>>()) as u64;
    }
    if !visitor.visit(allocation) {
        return;
    }
    match &node.storage {
        ColumnStorage::Page(values) => {
            for (offset, entry) in values.iter().enumerate() {
                let Some(entry) = entry else {
                    continue;
                };
                let old = previous.get_shared(start + offset);
                if old.is_none_or(|old| old.id() != entry.id()) {
                    value(
                        entry,
                        old.map(StorageAllocation::as_ref),
                        entry.observation(false),
                        visitor,
                    );
                }
            }
        }
        ColumnStorage::Branch(children) => {
            for (direction, child) in children.iter().enumerate() {
                if let Some(child) = child {
                    visit_node(
                        child,
                        height - 1,
                        start + direction * (PAGE_LEN << (height - 1)),
                        previous,
                        visitor,
                        value,
                    );
                }
            }
        }
    }
}
