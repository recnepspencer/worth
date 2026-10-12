use crate::transactions::data::EntityReference;
use crate::validation::data::TouchedStructuralSet;

use super::super::CustomInvariantWorkMeter;

fn reference_bytes(reference: &EntityReference) -> u64 {
    match reference {
        EntityReference::Existing(_) => 0,
        EntityReference::Created(created) => created.client_key.owned_allocation_capacity_bytes(),
    }
}

pub(super) fn claim_restricted_clones(
    touched: &TouchedStructuralSet,
    work: &CustomInvariantWorkMeter,
) -> bool {
    // Each Vec reserves the input extent; conversion briefly holds the Arc
    // slice too. Bound these real element types,
    // rather than assigning one BTree-node allocation to every copied row.
    let arrays = [
        array_bytes(touched.direct_visible_entity_ids()),
        array_bytes(touched.visible_entity_ids()),
        array_bytes(touched.visible_relation_ids()),
        array_bytes(touched.touched_partitions()),
        array_bytes(touched.planned_entity_deletes()),
        array_bytes(touched.planned_entity_creates()),
        array_bytes(touched.planned_relation_creates()),
        array_bytes(touched.planned_relation_deletes()),
        array_bytes(touched.planned_relation_endpoint_updates()),
    ]
    .into_iter()
    .fold(
        (std::mem::size_of::<TouchedStructuralSet>() + 2 * std::mem::size_of::<usize>()) as u64,
        u64::saturating_add,
    );
    if !work.try_claim_bytes(arrays) {
        return false;
    }
    for create in touched.planned_entity_creates() {
        if !work.try_claim_bytes(create.client_key().owned_allocation_capacity_bytes()) {
            return false;
        }
    }
    for create in touched.planned_relation_creates() {
        let bytes = create
            .client_key()
            .owned_allocation_capacity_bytes()
            .saturating_add(reference_bytes(create.source()))
            .saturating_add(reference_bytes(create.target()));
        if !work.try_claim_bytes(bytes) {
            return false;
        }
    }
    for update in touched.planned_relation_endpoint_updates() {
        let bytes =
            reference_bytes(update.source()).saturating_add(reference_bytes(update.target()));
        if !work.try_claim_bytes(bytes) {
            return false;
        }
    }
    true
}

fn array_bytes<T>(rows: &[T]) -> u64 {
    (rows.len() as u64)
        .saturating_mul(std::mem::size_of::<T>() as u64)
        .saturating_mul(2)
        .saturating_add((2 * std::mem::size_of::<usize>()) as u64)
}

pub(super) fn retain_rows<T>(rows: impl Iterator<Item = T>) -> std::sync::Arc<[T]> {
    let mut retained = Vec::with_capacity(rows.size_hint().1.expect("bounded scope rows"));
    retained.extend(rows);
    retained.into()
}
