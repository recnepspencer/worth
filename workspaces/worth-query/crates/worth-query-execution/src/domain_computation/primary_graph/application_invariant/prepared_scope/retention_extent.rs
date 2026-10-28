//! Admission-set backing, separate from the native traversal's own allocations.
use super::{ApplicationInvariantAdmission, PlanningAdmission};
use worth_relational::facade::identity::{EntityId, RelationId};

pub(super) fn planning_arc_bytes() -> u64 {
    std::mem::size_of::<(usize, usize, std::sync::Mutex<PlanningAdmission>)>() as u64
}

pub(super) fn frozen_bytes(entities: usize, relations: usize) -> u64 {
    (std::mem::size_of::<(usize, usize, ApplicationInvariantAdmission)>() as u64)
        .saturating_add((entities as u64).saturating_mul(entity_entry_bytes()))
        .saturating_add((relations as u64).saturating_mul(relation_entry_bytes()))
}

pub(super) fn entity_entry_bytes() -> u64 {
    tree_node_bytes::<EntityId>()
}

pub(super) fn relation_entry_bytes() -> u64 {
    tree_node_bytes::<RelationId>()
}

fn tree_node_bytes<Key>() -> u64 {
    // Supported Rust 1.94: eleven keys, twelve child pointers, a parent pointer
    // and one word for length/index/padding. An insertion-only set has at most
    // one node per key. Quote before insertion and again before frozen copies
    // coexist, rather than treating native result backing as free admission.
    (11 * std::mem::size_of::<Key>() + 14 * std::mem::size_of::<usize>()) as u64
}
