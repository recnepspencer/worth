//! A minimum's candidate sets retain only the endpoints that it checks.
use super::{
    InvariantExecutionContext, LoweredCardinalityMinimumContract, VisibleRelationCountSnapshot,
};
use crate::identity::data::KindId;
use crate::transactions::data::EntityReference;
use crate::validation::engine::evaluator::common::contract_candidate_kind_matches;

pub(super) fn record_candidate_entity(
    context: &InvariantExecutionContext<'_, '_>,
    contract: &LoweredCardinalityMinimumContract,
    snapshot: &mut VisibleRelationCountSnapshot,
    kind_id: KindId,
    entity: EntityReference,
) {
    let nested = match &entity {
        EntityReference::Existing(_) => 0,
        EntityReference::Created(created) => created.client_key.owned_allocation_capacity_bytes(),
    };
    let bytes = ((std::mem::size_of::<EntityReference>() + 3 * std::mem::size_of::<usize>())
        as u64)
        .saturating_add(nested);
    for (minimum, kinds, candidates) in [
        (
            contract.source_min,
            &contract.candidate_source_kinds,
            &mut snapshot.candidate_source_entities,
        ),
        (
            contract.target_min,
            &contract.candidate_target_kinds,
            &mut snapshot.candidate_target_entities,
        ),
    ] {
        if minimum.is_some()
            && contract_candidate_kind_matches(kind_id, kinds)
            && !candidates.contains(&entity)
        {
            if !context.claim_scratch(bytes) {
                return;
            }
            candidates.insert(entity.clone());
        }
    }
}
