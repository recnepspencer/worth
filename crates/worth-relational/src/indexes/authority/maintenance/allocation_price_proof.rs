//! Independent arithmetic for the two-deletion, one-field fixture.
use crate::{
    identity::data::EntityId, indexes::data::*,
    storage::data::AuthoritativeFieldComparisonKey as Key,
};
use std::{mem::size_of, sync::Arc};
fn ordered<K, V>() -> usize {
    11 * size_of::<(K, V)>() + 14 * size_of::<usize>()
}
pub(crate) fn two_row_patch_bytes(
    locator: &worth_foundational::facade::AspectFieldLocator,
    value: &worth_foundational::facade::AspectValue,
) -> u64 {
    let node = 64 * size_of::<(Arc<Key>, DerivedIndexRows<EntityId>)>() + 71 * size_of::<usize>();
    let encoded = Key::required_encoded_capacity_bytes(value).unwrap() as usize;
    // Two height-2 paths; two keys, changed-ID nodes, queued-key nodes and
    // four-slot pending vectors. Fixed: three branch names, input IDs, request
    // set, four pending fields, locator, four prepared entries, publication
    // branch, generation, generation ID and two output branch names.
    (2 * (3 * 2 + 2) * node
        + 2 * (encoded
            + ordered::<EntityId, ()>()
            + ordered::<Key, Vec<super::entry_edits::PendingEdit<EntityId>>>()
            + 4 * size_of::<super::entry_edits::PendingEdit<EntityId>>())
        + 3 * 4
        + size_of::<DerivedIndexId>()
        + ordered::<DerivedIndexId, ()>()
        + 4 * size_of::<super::field::PendingField>()
        + locator.owned_allocation_capacity_bytes()
        + 4 * size_of::<(DerivedIndexId, DerivedIndexEntries)>()
        + 4
        + size_of::<DerivedIndexGeneration>()
        + size_of::<DerivedIndexGenerationId>()
        + 2 * 4) as u64
}
