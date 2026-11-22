//! Physical allocation traversal for the persistent aspect entry AVL tree.
use super::{lookup_node, option_arc_ptr_eq, AspectEntryTree, EntryNode, Link};
use crate::aspects::allocation_visitor::{
    arc_allocation_bytes, arc_id, FoundationalAllocationKind, FoundationalAllocationObservation,
    FoundationalAllocationVisitor,
};
use std::sync::Arc;

impl AspectEntryTree {
    pub(in super::super) fn checked_node_allocation_bytes() -> Option<usize> {
        arc_allocation_bytes::<EntryNode>()
    }

    pub(in super::super) fn node_allocation_bytes() -> usize {
        Self::checked_node_allocation_bytes()
            .expect("the fixed aspect entry Arc layout fits the address space")
    }

    pub(in super::super) fn visit_allocations(
        &self,
        ancestor_unique: bool,
        visitor: &mut dyn FoundationalAllocationVisitor,
    ) {
        visit_link(&self.root, ancestor_unique, visitor);
    }

    pub(in super::super) fn visit_new_allocations(
        &self,
        previous: &Self,
        ancestor_unique: bool,
        visitor: &mut dyn FoundationalAllocationVisitor,
    ) {
        if option_arc_ptr_eq(&self.root, &previous.root) {
            return;
        }
        visit_new_link(&self.root, &previous.root, ancestor_unique, visitor);
    }
}

fn visit_link(
    link: &Link,
    ancestor_unique: bool,
    visitor: &mut dyn FoundationalAllocationVisitor,
) {
    let Some(node) = link else { return };
    let strong_count = Arc::strong_count(node);
    let unique = ancestor_unique && strong_count == 1;
    let observation = FoundationalAllocationObservation {
        id: arc_id(node),
        bytes: AspectEntryTree::node_allocation_bytes() as u64,
        unique,
        strong_count,
        kind: FoundationalAllocationKind::ValueBacking,
    };
    if !visitor.visit(observation) {
        return;
    }
    node.value.visit_allocations(unique, visitor);
    visit_link(&node.left, unique, visitor);
    visit_link(&node.right, unique, visitor);
}

fn visit_new_link(
    link: &Link,
    previous_root: &Link,
    ancestor_unique: bool,
    visitor: &mut dyn FoundationalAllocationVisitor,
) {
    let Some(node) = link else { return };
    let previous = lookup_node(previous_root, node.value.key());
    if previous.is_some_and(|prior| Arc::ptr_eq(node, prior)) {
        return;
    }
    let strong_count = Arc::strong_count(node);
    let unique = ancestor_unique && strong_count == 1;
    let observation = FoundationalAllocationObservation {
        id: arc_id(node),
        bytes: AspectEntryTree::node_allocation_bytes() as u64,
        unique,
        strong_count,
        kind: FoundationalAllocationKind::ValueBacking,
    };
    if !visitor.visit(observation) {
        return;
    }
    match previous {
        Some(prior) if node.value.shares_allocation(&prior.value) => {}
        Some(prior) => node
            .value
            .visit_new_allocations(Some(&prior.value), unique, visitor),
        None => node.value.visit_allocations(unique, visitor),
    }
    visit_new_link(&node.left, previous_root, unique, visitor);
    visit_new_link(&node.right, previous_root, unique, visitor);
}
