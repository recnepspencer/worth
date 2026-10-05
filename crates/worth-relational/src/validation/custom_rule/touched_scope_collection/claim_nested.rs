use crate::symbols::data::ClientKey;
use crate::transactions::data::{
    CreateIntent, EntityReference, MutationIntent, RelationMutationIntent,
};

use super::super::CustomInvariantWorkMeter;

fn reference_bytes(reference: &EntityReference) -> u64 {
    match reference {
        EntityReference::Existing(_) => 0,
        EntityReference::Created(created) => created.client_key.owned_allocation_capacity_bytes(),
    }
}

fn claim_key(key: &ClientKey, work: &CustomInvariantWorkMeter) -> bool {
    work.try_charge(0) && work.try_claim_bytes(key.owned_allocation_capacity_bytes())
}

fn claim_references(
    source: &EntityReference,
    target: &EntityReference,
    work: &CustomInvariantWorkMeter,
) -> bool {
    work.try_charge(0)
        && work.try_claim_bytes(reference_bytes(source).saturating_add(reference_bytes(target)))
}

/// Preclaim variable-size keys and created references before touched-scope
/// construction clones them. Fixed-size collection retention is charged by
/// the per-candidate work meter.
pub(super) fn claim_intent_nested_allocations(
    intent: &MutationIntent,
    access: &crate::validation::data::CustomInvariantAccessContract,
    work: &CustomInvariantWorkMeter,
) -> bool {
    match intent {
        MutationIntent::Create(CreateIntent::Entity(spec)) => {
            !access.affects_entity(spec.kind_id) || claim_key(&spec.client_key, work)
        }
        MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
            !access.affects_entity(spec.kind_id) || claim_key(&spec.client_key, work)
        }
        MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
            !access.affects_entity(spec.kind_id)
                || spec.client_keys.iter().all(|key| claim_key(key, work))
        }
        MutationIntent::Create(CreateIntent::Relation(spec)) => {
            !access.affects_relation(spec.kind_id)
                || (claim_key(&spec.client_key, work)
                    && claim_references(&spec.source, &spec.target, work))
        }
        MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
            !access.affects_relation(spec.kind_id)
                || (claim_key(&spec.client_key, work)
                    && claim_references(&spec.source, &spec.target, work))
        }
        MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
            !access.affects_relation(spec.kind_id)
                || (spec.client_keys.iter().all(|key| claim_key(key, work))
                    && spec
                        .endpoints
                        .iter()
                        .all(|(source, target)| claim_references(source, target, work)))
        }
        MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
            !access.affects_relation(spec.kind_id)
                || claim_references(&spec.source, &spec.target, work)
        }
        _ => true,
    }
}
