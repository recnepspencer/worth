//! Physical allocation traversal for persistent AVL struct fields.
use super::{arc_storage, Link, Node, StructFields};
use crate::aspects::allocation_visitor::{
    arc_id, FoundationalAllocationKind, FoundationalAllocationObservation,
    FoundationalAllocationVisitor,
};
use crate::aspects::structs::FieldKey;
use crate::values::AspectValue;
use std::sync::Arc;

impl StructFields {
    pub(in crate::aspects::structs) fn visit_allocations(
        &self,
        ancestor_unique: bool,
        visitor: &mut dyn FoundationalAllocationVisitor,
    ) {
        visit_link(&self.root, ancestor_unique, visitor);
    }

    pub(in crate::aspects::structs) fn visit_new_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn FoundationalAllocationVisitor,
    ) {
        if option_arc_ptr_eq(&self.root, &previous.root) {
            return;
        }
        visit_new_link(&self.root, &previous.root, visitor);
    }
}

fn option_arc_ptr_eq(left: &Link, right: &Link) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => Arc::ptr_eq(left, right),
        _ => false,
    }
}

fn lookup_node<'a>(root: &'a Link, key: &FieldKey) -> Option<&'a Arc<Node>> {
    let mut current = root.as_ref();
    while let Some(node) = current {
        match key.cmp(&node.key) {
            std::cmp::Ordering::Less => current = node.left.as_ref(),
            std::cmp::Ordering::Equal => return Some(node),
            std::cmp::Ordering::Greater => current = node.right.as_ref(),
        }
    }
    None
}

fn node_bytes() -> u64 {
    arc_storage::<Node>().expect("struct field node layout fits") as u64
}

fn key_bytes(key: &FieldKey) -> u64 {
    (arc_storage::<FieldKey>().expect("field key arc layout fits")
        + key.owned_allocation_capacity_bytes()) as u64
}

fn value_bytes(value: &AspectValue) -> u64 {
    (arc_storage::<AspectValue>().expect("field value arc layout fits")
        + value.direct_owned_allocation_capacity_bytes()) as u64
}

fn visit_link(
    link: &Link,
    ancestor_unique: bool,
    visitor: &mut dyn FoundationalAllocationVisitor,
) {
    let Some(node) = link else { return };
    let node_unique = ancestor_unique && Arc::strong_count(node) == 1;
    let node_obs = FoundationalAllocationObservation {
        id: arc_id(node),
        bytes: node_bytes(),
        unique: node_unique,
        kind: FoundationalAllocationKind::ValueBacking,
    };
    if !visitor.visit(node_obs) {
        return;
    }
    visitor.visit(FoundationalAllocationObservation {
        id: arc_id(&node.key),
        bytes: key_bytes(&node.key),
        unique: node_unique && Arc::strong_count(&node.key) == 1,
        kind: FoundationalAllocationKind::ValueBacking,
    });
    let value_unique = node_unique && Arc::strong_count(&node.value) == 1;
    let value_obs = FoundationalAllocationObservation {
        id: arc_id(&node.value),
        bytes: value_bytes(&node.value),
        unique: value_unique,
        kind: FoundationalAllocationKind::ValueBacking,
    };
    if visitor.visit(value_obs) {
        node.value.visit_shared_allocations(value_unique, visitor);
    }
    visit_link(&node.left, node_unique, visitor);
    visit_link(&node.right, node_unique, visitor);
}

fn visit_new_link(
    link: &Link,
    previous_root: &Link,
    visitor: &mut dyn FoundationalAllocationVisitor,
) {
    let Some(node) = link else { return };
    let previous = lookup_node(previous_root, &node.key);
    if previous.is_some_and(|prior| Arc::ptr_eq(node, prior)) {
        return;
    }
    let node_obs = FoundationalAllocationObservation {
        id: arc_id(node),
        bytes: node_bytes(),
        unique: false,
        kind: FoundationalAllocationKind::ValueBacking,
    };
    if !visitor.visit(node_obs) {
        return;
    }
    if !previous.is_some_and(|prior| Arc::ptr_eq(&node.key, &prior.key)) {
        visitor.visit(FoundationalAllocationObservation {
            id: arc_id(&node.key),
            bytes: key_bytes(&node.key),
            unique: false,
            kind: FoundationalAllocationKind::ValueBacking,
        });
    }
    if !previous.is_some_and(|prior| Arc::ptr_eq(&node.value, &prior.value)) {
        let value_obs = FoundationalAllocationObservation {
            id: arc_id(&node.value),
            bytes: value_bytes(&node.value),
            unique: false,
            kind: FoundationalAllocationKind::ValueBacking,
        };
        if visitor.visit(value_obs) {
            node.value.visit_shared_allocations(false, visitor);
        }
    }
    visit_new_link(&node.left, previous_root, visitor);
    visit_new_link(&node.right, previous_root, visitor);
}
