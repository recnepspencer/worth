use crate::transactions::data::{CreatedEntityRef, EntityReference};

use super::super::super::context::InvariantExecutionContext;
use super::super::common::contract_candidate_kind_matches;

pub(super) fn visible_entities_of_kinds(
    context: &InvariantExecutionContext<'_, '_>,
    kind_ids: &[crate::identity::data::KindId],
) -> Vec<EntityReference> {
    let mut entities = Vec::new();
    collect_state_view_entities(context, kind_ids, &mut entities);
    if !context.checkpoint(0) {
        return Vec::new();
    }
    if context.applied_created_entities().is_none() {
        collect_planned_entities(context, kind_ids, &mut entities);
    }
    entities
}

fn collect_state_view_entities(
    context: &InvariantExecutionContext<'_, '_>,
    kind_ids: &[crate::identity::data::KindId],
    entities: &mut Vec<EntityReference>,
) {
    let state_view = context.state_view();
    for partition_id in state_view.state().partition_ids_iter() {
        if !context.checkpoint(1) {
            return;
        }
        let slots = state_view.try_entity_slots(partition_id, |work, bytes| {
            if context.checkpoint(work) && context.claim_scratch(bytes) {
                Ok(())
            } else {
                Err(())
            }
        });
        let Ok(slots) = slots else {
            return;
        };
        for slot in slots {
            if !context.checkpoint(1) {
                return;
            }
            context.metrics().count_entity_slot_scans(1);
            let Some(metadata) = state_view.entity_metadata_for_slot(partition_id, slot) else {
                continue;
            };
            if contract_candidate_kind_matches(metadata.kind_id, kind_ids) {
                if !context.claim_scratch(std::mem::size_of::<EntityReference>() as u64) {
                    return;
                }
                entities.push(EntityReference::Existing(metadata.entity_id));
            }
        }
    }
}

fn collect_planned_entities(
    context: &InvariantExecutionContext<'_, '_>,
    kind_ids: &[crate::identity::data::KindId],
    entities: &mut Vec<EntityReference>,
) {
    let Some(merged_plan) = context.merged_plan() else {
        return;
    };
    for intent in &merged_plan.merged_intents {
        if !context.checkpoint(1) {
            return;
        }
        match intent {
            crate::transactions::data::MutationIntent::Create(
                crate::transactions::data::CreateIntent::Entity(spec),
            ) => {
                if contract_candidate_kind_matches(spec.kind_id, kind_ids) {
                    if !context.claim_scratch(
                        (std::mem::size_of::<EntityReference>() as u64)
                            .saturating_add(spec.client_key.owned_allocation_capacity_bytes()),
                    ) {
                        return;
                    }
                    entities.push(EntityReference::Created(CreatedEntityRef {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: spec.client_key.clone(),
                    }));
                }
            }
            crate::transactions::data::MutationIntent::Create(
                crate::transactions::data::CreateIntent::BulkEntities(spec),
            ) if contract_candidate_kind_matches(spec.kind_id, kind_ids) => {
                for client_key in &spec.client_keys {
                    if !context.checkpoint(1) {
                        return;
                    }
                    if !context.claim_scratch(
                        (std::mem::size_of::<EntityReference>() as u64)
                            .saturating_add(client_key.owned_allocation_capacity_bytes()),
                    ) {
                        return;
                    }
                    entities.push(EntityReference::Created(CreatedEntityRef {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: client_key.clone(),
                    }));
                }
            }
            _ => {}
        }
    }
}
