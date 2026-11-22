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
        ancestor_unique: bool,
        visitor: &mut dyn FoundationalAllocationVisitor,
    ) {
        if option_arc_ptr_eq(&self.root, &previous.root) {
            return;
        }
        visit_new_link(&self.root, &previous.root, ancestor_unique, visitor);
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
        match key.cmp(&node.entry.key) {
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

fn key_arc_bytes() -> u64 {
    arc_storage::<FieldKey>().expect("field key arc layout fits") as u64
}

fn value_arc_bytes() -> u64 {
    arc_storage::<AspectValue>().expect("field value arc layout fits") as u64
}

fn visit_link(
    link: &Link,
    ancestor_unique: bool,
    visitor: &mut dyn FoundationalAllocationVisitor,
) {
    let Some(node) = link else { return };
    let node_strong_count = Arc::strong_count(node);
    let node_unique = ancestor_unique && node_strong_count == 1;
    let node_obs = FoundationalAllocationObservation {
        id: arc_id(node),
        bytes: node_bytes(),
        unique: node_unique,
        strong_count: node_strong_count,
        birth_seq: node.birth_seq,
        kind: FoundationalAllocationKind::ValueBacking,
    };
    if !visitor.visit(node_obs) {
        return;
    }
    let key_strong_count = Arc::strong_count(&node.entry.key);
    let key_unique = node_unique && key_strong_count == 1;
    if visitor.visit(FoundationalAllocationObservation {
        id: arc_id(&node.entry.key),
        bytes: key_arc_bytes(),
        unique: key_unique,
        strong_count: key_strong_count,
        birth_seq: node.entry.key_birth_seq,
        kind: FoundationalAllocationKind::ValueBacking,
    }) {
        node.entry
            .key
            .visit_allocations(key_unique, node.entry.key_birth_seq, visitor);
    }
    let value_strong_count = Arc::strong_count(&node.entry.value);
    let value_unique = node_unique && value_strong_count == 1;
    if visitor.visit(FoundationalAllocationObservation {
        id: arc_id(&node.entry.value),
        bytes: value_arc_bytes(),
        unique: value_unique,
        strong_count: value_strong_count,
        birth_seq: node.entry.value_birth_seq,
        kind: FoundationalAllocationKind::ValueBacking,
    }) {
        node.entry
            .value
            .visit_allocations(value_unique, node.entry.value_birth_seq, visitor);
    }
    visit_link(&node.left, node_unique, visitor);
    visit_link(&node.right, node_unique, visitor);
}

fn visit_new_link(
    link: &Link,
    previous_root: &Link,
    ancestor_unique: bool,
    visitor: &mut dyn FoundationalAllocationVisitor,
) {
    let Some(node) = link else { return };
    let previous = lookup_node(previous_root, &node.entry.key);
    if previous.is_some_and(|prior| Arc::ptr_eq(node, prior)) {
        return;
    }
    let node_strong_count = Arc::strong_count(node);
    let node_unique = ancestor_unique && node_strong_count == 1;
    let node_obs = FoundationalAllocationObservation {
        id: arc_id(node),
        bytes: node_bytes(),
        unique: node_unique,
        strong_count: node_strong_count,
        birth_seq: node.birth_seq,
        kind: FoundationalAllocationKind::ValueBacking,
    };
    if !visitor.visit(node_obs) {
        return;
    }
    if !previous.is_some_and(|prior| Arc::ptr_eq(&node.entry.key, &prior.entry.key)) {
        let key_strong_count = Arc::strong_count(&node.entry.key);
        let key_unique = node_unique && key_strong_count == 1;
        if visitor.visit(FoundationalAllocationObservation {
            id: arc_id(&node.entry.key),
            bytes: key_arc_bytes(),
            unique: key_unique,
            strong_count: key_strong_count,
            birth_seq: node.entry.key_birth_seq,
            kind: FoundationalAllocationKind::ValueBacking,
        }) {
            node.entry
                .key
                .visit_allocations(key_unique, node.entry.key_birth_seq, visitor);
        }
    }
    if !previous.is_some_and(|prior| Arc::ptr_eq(&node.entry.value, &prior.entry.value)) {
        let value_strong_count = Arc::strong_count(&node.entry.value);
        let value_unique = node_unique && value_strong_count == 1;
        if visitor.visit(FoundationalAllocationObservation {
            id: arc_id(&node.entry.value),
            bytes: value_arc_bytes(),
            unique: value_unique,
            strong_count: value_strong_count,
            birth_seq: node.entry.value_birth_seq,
            kind: FoundationalAllocationKind::ValueBacking,
        }) {
            node.entry.value.visit_new_allocations(
                previous.map(|prior| prior.entry.value.as_ref()),
                value_unique,
                node.entry.value_birth_seq,
                visitor,
            );
        }
    }
    visit_new_link(&node.left, previous_root, node_unique, visitor);
    visit_new_link(&node.right, previous_root, node_unique, visitor);
}
