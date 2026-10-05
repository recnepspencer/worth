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
