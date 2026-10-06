use crate::storage::data::RecordLifecycleState;
use crate::transactions::data::{CreatedEntityRef, EntityReference};
use crate::validation::data::{InvariantClass, InvariantViolation};

use super::super::super::context::InvariantExecutionContext;
use super::{storage_inconsistency_violation, StorageInconsistencyContext};

pub(crate) fn contract_candidate_kind_matches(
    kind_id: crate::identity::data::KindId,
    candidate_kinds: &[crate::identity::data::KindId],
) -> bool {
    candidate_kinds.binary_search(&kind_id).is_ok()
}

pub(crate) fn entity_kind_in_state(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    entity_id: crate::identity::data::EntityId,
) -> Result<Option<crate::identity::data::KindId>, InvariantViolation> {
    let Some(partition) = context
        .partition_access()
        .get_partition(entity_id.partition_id)
    else {
        return Err(storage_inconsistency_violation(
            class,
            format!(
                "entity endpoint {:?} references missing partition {:?}",
                entity_id, entity_id.partition_id
            ),
            StorageInconsistencyContext::default()
                .with_entity_id(entity_id)
                .with_partition_id(entity_id.partition_id)
                .with_lookup(
                    crate::validation::data::StorageInconsistencyLookup::EntityKindInState,
                ),
        ));
    };
    let Some(slot) = partition.entity_arena.get(&entity_id) else {
        return Err(storage_inconsistency_violation(
            class,
            format!(
                "entity endpoint {:?} references missing entity slot",
                entity_id
            ),
            StorageInconsistencyContext::default()
                .with_entity_id(entity_id)
                .with_partition_id(entity_id.partition_id)
                .with_lookup(crate::validation::data::StorageInconsistencyLookup::EntityKindInState)
                .with_failure(crate::validation::data::StorageInconsistencyFailure::MissingSlot),
        ));
    };
    if slot.lifecycle() != RecordLifecycleState::Live {
        return Ok(None);
    }
    slot.kind_id()
        .ok_or_else(|| {
            storage_inconsistency_violation(
                class,
                format!(
                    "entity endpoint {:?} is live but missing kind id",
                    entity_id
                ),
                StorageInconsistencyContext::default()
                    .with_entity_id(entity_id)
                    .with_partition_id(entity_id.partition_id)
                    .with_lookup(
                        crate::validation::data::StorageInconsistencyLookup::EntityKindInState,
                    )
                    .with_failure(
                        crate::validation::data::StorageInconsistencyFailure::MissingKindId,
                    ),
            )
        })
        .map(Some)
}

pub(crate) fn entity_reference_kind(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    entity_reference: &EntityReference,
) -> Result<Option<crate::identity::data::KindId>, InvariantViolation> {
    match entity_reference {
        EntityReference::Existing(entity_id) => entity_kind_in_state(context, class, *entity_id),
        EntityReference::Created(created) => Ok(created_entity_kind_in_plan(context, created)),
    }
}

fn created_entity_kind_in_plan(
    context: &InvariantExecutionContext<'_, '_>,
    created: &CreatedEntityRef,
) -> Option<crate::identity::data::KindId> {
    let merged_plan = context.merged_plan()?;
    for intent in &merged_plan.merged_intents {
        if !context.checkpoint(1) {
            return None;
        }
        match intent {
            crate::transactions::data::MutationIntent::Create(
                crate::transactions::data::CreateIntent::Entity(spec),
            ) if spec.partition_id == created.partition_id
                && spec.kind_id == created.kind_id
                && spec.client_key == created.client_key =>
            {
                return Some(spec.kind_id);
            }
            crate::transactions::data::MutationIntent::Create(
                crate::transactions::data::CreateIntent::BulkEntities(spec),
            ) if spec.partition_id == created.partition_id && spec.kind_id == created.kind_id => {
                for key in &spec.client_keys {
                    if !context.checkpoint(1) {
                        return None;
                    }
                    if key == &created.client_key {
                        return Some(spec.kind_id);
                    }
                }
            }
            _ => {}
        }
    }
    None
}
