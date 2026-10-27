//! Before-growth bounds for this traversal's actual standard collections.
use super::super::super::StructuralRelationRecord;
use super::super::complete_result::StructuralWalkStep;
use crate::identity::data::EntityId;

pub(super) fn vertex_bytes() -> usize {
    // The supported Rust BTree has eleven key slots, twelve child pointers,
    // one parent pointer and one word for length/index/padding. An insertion-
    // only set has at most one node per stored key, including its root.
    let visited_node = 11 * std::mem::size_of::<EntityId>() + 14 * std::mem::size_of::<usize>();
    // These element types exceed eight bytes. Their Vec/VecDeque starts at
    // four slots and doubles: four slots per retained item bounds both the
    // first allocation and every subsequent capacity, before it grows.
    let frontier = 4 * std::mem::size_of::<(usize, EntityId)>();
    let result = 4 * std::mem::size_of::<StructuralWalkStep>();
    visited_node + frontier + result
}

pub(super) fn relation_bytes() -> usize {
    4 * std::mem::size_of::<StructuralRelationRecord>()
}
