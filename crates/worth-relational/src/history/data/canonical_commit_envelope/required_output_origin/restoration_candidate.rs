//! Compare the actual inverse candidate against Native's complete recorded payload plan.
use super::*;
use crate::{storage::overlay::*, transactions::data::*};
use worth_execution::MapKernelContext;
use worth_foundational::facade::*;

pub(crate) fn validate(
    basis: &AdmittedRelationalBranchBasis,
    state: &WorkingState,
    created: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
    restoration: &crate::mvcc::RelationalAuthoredRestoration,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    let after = OverlayStateView::new(basis.inner.root.as_ref(), state);
    for intent in &restoration.inverse {
        context
            .checkpoint(
                (restoration.entities.len() + restoration.relations.len()) as u64
                    * intent.owned_allocation_capacity_bytes().saturating_add(1),
            )
            .map_err(origin_request)?;
        let subject = match intent {
            MutationIntent::Create(CreateIntent::EntityAspects(spec)) => restoration
                .entities
                .iter()
                .find(|(_, target)| {
                    target.partition_id == spec.partition_id
                        && target.kind_id == spec.kind_id
                        && target.client_key == spec.client_key
                })
                .map(|(id, _)| RecordRef::Entity(*id)),
            MutationIntent::Create(CreateIntent::RelationAspects(spec)) => restoration
                .relations
                .iter()
                .find(|(_, target)| {
                    target.partition_id == spec.partition_id
                        && target.kind_id == spec.kind_id
                        && target.client_key == spec.client_key
                })
                .map(|(id, _)| RecordRef::Relation(*id)),
            _ => None,
        }
        .unwrap_or(RecordRef::Entity(
            restoration
                .retirement
                .authored_cascade_retirement
                .as_ref()
                .expect("Native restoration has its retirement")
                .anchor,
        ));
        let refused = || incomplete(subject.clone());
        context
            .checkpoint(intent.owned_allocation_capacity_bytes().saturating_add(1))
            .map_err(origin_request)?;
        match intent {
            MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
                let target = CreatedEntityRef {
                    partition_id: spec.partition_id,
                    kind_id: spec.kind_id,
                    client_key: spec.client_key.clone(),
                };
                context
                    .checkpoint(created.resolve_work_bound(&target).ok_or_else(refused)?)
                    .map_err(origin_request)?;
                let id = created.resolve(&target).ok_or_else(refused)?;
                let (_, partition) = current_entity_partition_source_for_slot(
                    &after,
                    id.partition_id,
                    id.slot_index(),
                )
                .ok_or_else(refused)?;
                let row = partition
                    .entity_arena()
                    .get(&id)
                    .filter(|row| row.is_live() && row.kind_id() == Some(spec.kind_id))
                    .ok_or_else(refused)?;
                validate_payload(
                    basis
                        .inner
                        .root
                        .schema_authority()
                        .entity_aspect_plan(spec.kind_id)
                        .ok_or_else(refused)?,
                    &spec.aspect_patch,
                    row.extra().authoritative_aspect_state.as_ref(),
                    subject.clone(),
                )?;
            }
            MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
                let target = CreatedRelationRef {
                    partition_id: spec.partition_id,
                    kind_id: spec.kind_id,
                    client_key: spec.client_key.clone(),
                    source: spec.source.clone(),
                    target: spec.target.clone(),
                };
                context
                    .checkpoint(
                        created_relations
                            .resolve_work_bound(&target)
                            .ok_or_else(refused)?,
                    )
                    .map_err(origin_request)?;
                let id = created_relations.resolve(&target).ok_or_else(refused)?;
                let (_, partition) = current_relation_partition_source_for_slot(
                    &after,
                    id.partition_id,
                    id.slot_index(),
                )
                .ok_or_else(refused)?;
                let row = partition
                    .relation_arena()
                    .get(&id)
                    .filter(|row| row.is_live() && row.kind_id() == Some(spec.kind_id))
                    .ok_or_else(refused)?;
                let mut resolve = |target: &EntityReference| match target {
                    EntityReference::Existing(id) => Ok(*id),
                    EntityReference::Created(target) => {
                        context
                            .checkpoint(created.resolve_work_bound(target).ok_or_else(refused)?)
                            .map_err(origin_request)?;
                        created.resolve(target).ok_or_else(refused)
                    }
                };
                let (source, target) = (resolve(&spec.source)?, resolve(&spec.target)?);
                if !row
                    .extra()
                    .endpoints
                    .as_ref()
                    .is_some_and(|ends| ends.source == source && ends.target == target)
                {
                    return Err(refused());
                }
                let plan = basis
                    .inner
                    .root
                    .schema_authority()
                    .relation_aspect_plan(spec.kind_id)
                    .ok_or_else(refused)?;
                let patch = crate::authority::mutation::intents::relation_endpoint_candidate::append_authoritative_endpoints(
                    spec.aspect_patch.clone(), Some(plan), source, target);
                validate_payload(
                    plan,
                    &patch,
                    row.extra().authoritative_aspect_state.as_ref(),
                    subject.clone(),
                )?;
            }
            _ => return Err(refused()),
        };
    }
    Ok(())
}

fn validate_payload(
    plan: &crate::schema::data::LoweredAspectContractPlan,
    patch: &PortableRecordAspectPatch,
    payload: Option<&AuthoritativeRecordAspectState>,
    subject: RecordRef,
) -> Result<(), TransactionCommitError> {
    let refused = || incomplete(subject.clone());
    let expected = match readmit_portable_record_aspect_patch(
        patch.clone(),
        PortablePatchReadmissionPurpose::RecordCreation,
        &|key: &AspectKey| plan.contract_for(key),
    ) {
        worth_proof::TransitionOutcome::Success(patch) => patch,
        worth_proof::TransitionOutcome::Denied(_) => return Err(refused()),
    };
    let expected = match expected.apply_to_optional(None) {
        worth_proof::TransitionOutcome::Success(state) => {
            state.map(|a| a.into_parts().into_parts().0)
        }
        worth_proof::TransitionOutcome::Denied(_) => return Err(refused()),
    };
    if expected.as_ref() != payload {
        return Err(refused());
    }
    Ok(())
}

fn incomplete(subject: RecordRef) -> TransactionCommitError {
    TransactionCommitError::publication(
        crate::publication::data::PublicationError::authored_retirement(
            crate::publication::data::AuthoredRetirementRefusal::IncompleteRestoration,
            subject,
        ),
    )
}
