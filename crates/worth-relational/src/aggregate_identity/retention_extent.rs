//! Conservative coexistence bounds for the closed aggregate record grammar.
use super::{AggregateAssignment, AggregateLineageEvent};
use crate::identity::data::{EntityId, PartitionId, RelationId};
use crate::transactions::data::{EntityReference, MutationIntent};
use std::collections::{BTreeMap, BTreeSet};
use std::mem::size_of;
use worth_foundational::facade::{AggregateIdentity, AspectFieldLocator, AspectValue, FieldKey};

// Rust 1.94 uses eleven entries, twelve child links, parent and length/index.
// One whole node per retained entry covers both split nodes and unused slots.
pub(super) fn tree_entry<K, V>() -> u64 {
    (11 * size_of::<(K, V)>() + 14 * size_of::<usize>()) as u64
}
pub(super) fn vector_entry<T>() -> u64 {
    (4 * size_of::<T>()) as u64
}
pub(super) fn arc_backing<T>() -> u64 {
    let header = 2 * size_of::<usize>();
    let alignment = std::mem::align_of::<T>();
    let padding = (alignment - header % alignment) % alignment;
    (header + padding + size_of::<T>()) as u64
}
pub(super) fn seed() -> u64 {
    tree_entry::<EntityId, ()>()
}
pub(super) fn affected_delta() -> u64 {
    2 * seed() + tree_entry::<EntityId, BTreeSet<RelationId>>() + tree_entry::<RelationId, ()>()
}
pub(super) fn frontier_vertex() -> u64 {
    seed() + 2 * vector_entry::<(usize, EntityId)>()
}
pub(super) fn relation_candidate() -> u64 {
    tree_entry::<RelationId, ()>() + 2 * vector_entry::<RelationId>()
}
pub(super) fn successor_edge() -> u64 {
    tree_entry::<(RelationId, EntityId), ()>()
}
pub(super) fn membership() -> u64 {
    tree_entry::<EntityId, (RelationId, AggregateIdentity)>()
}
pub(super) fn identity_target() -> u64 {
    tree_entry::<AggregateIdentity, EntityReference>() + 64
}
pub(super) fn assignment_member() -> u64 {
    tree_entry::<EntityId, AggregateIdentity>()
}
pub(super) fn demand() -> u64 {
    tree_entry::<EntityId, ()>() + tree_entry::<RelationId, ()>() + tree_entry::<PartitionId, ()>()
}
pub(super) fn demands(entities: usize, relations: usize, partitions: usize) -> u64 {
    entities as u64 * tree_entry::<EntityId, ()>()
        + relations as u64 * tree_entry::<RelationId, ()>()
        + partitions as u64 * tree_entry::<PartitionId, ()>()
}
pub(super) fn transition_member() -> u64 {
    4 * tree_entry::<EntityId, usize>()
        + 2 * tree_entry::<AggregateIdentity, ()>()
        + 2 * tree_entry::<usize, EntityId>()
        + 2 * tree_entry::<AggregateLineageEvent, ()>()
        + seed()
        + vector_entry::<AggregateAssignment>()
}

// These names are fixed by record.rs; no application string or aspect payload
// enters the metadata grammar. Temporary formatted field keys coexist with
// their locator strings, field-path backing, B-tree and boxed patch vector.
const FIXED_KEY_CAPACITY: u64 = 64;
pub(super) fn record_fields() -> u64 {
    let fields = super::record::AGGREGATE_FIELDS.len() as u64;
    fields
        * (tree_entry::<AspectFieldLocator, AspectValue>()
            + vector_entry::<(AspectFieldLocator, AspectValue)>()
            + 4 * FIXED_KEY_CAPACITY
            + size_of::<FieldKey>() as u64)
}
pub(super) fn stored_row() -> u64 {
    record_fields()
        + tree_entry::<AggregateIdentity, super::cohorts::StoredAggregate>()
        + vector_entry::<super::PriorAggregate>()
}
pub(super) fn row_mutation() -> u64 {
    record_fields() + demand() + 3 * FIXED_KEY_CAPACITY
}
pub(super) fn membership_mutation() -> u64 {
    demand() + 4 * FIXED_KEY_CAPACITY + size_of::<MutationIntent>() as u64
}
pub(super) fn lineage_mutation() -> u64 {
    2 * demand() + record_fields() + 8 * FIXED_KEY_CAPACITY
}
pub(super) fn group() -> u64 {
    vector_entry::<BTreeSet<EntityId>>()
}
pub(super) fn empty_candidate() -> u64 {
    size_of::<BTreeMap<EntityId, usize>>() as u64
}
