mod relation_rematerialization;
use crate::authority::mutation::outcomes::{MutationEvent, MutationOutcome, RecordMutation};
use crate::authority::mutation::MutationWorkspace;
use crate::storage::substrate::{EntityExtra, RelationEndpoints, RelationExtra};
use crate::transactions::data::{
    CommitConflict, ConflictClass, MaterializationMutationIntent, RecordAspectPatchTarget,
};
use relation_rematerialization::rematerialize_relation;
use worth_foundational::facade::PortablePatchReadmissionPurpose;

use super::field_authoring_candidate::{self, FieldAuthoringDomain};
use super::{record_aspect_patch, relation_endpoint_candidate};

pub(super) fn apply(
    intent: &MaterializationMutationIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    match intent {
        MaterializationMutationIntent::SuspendEntity(intent) => {
            suspend_entity(intent.entity_id, workspace)
        }
        MaterializationMutationIntent::SuspendRelation(intent) => {
            suspend_relation(intent.relation_id, workspace)
        }
        MaterializationMutationIntent::RematerializeEntity(intent) => {
            rematerialize_entity(intent, workspace)
        }
        MaterializationMutationIntent::RematerializeRelation(intent) => {
            rematerialize_relation(intent, workspace)
        }
    }
}

fn suspend_entity(
    entity_id: crate::identity::data::EntityId,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let mut unavailable =
        EntityExtra::empty_for_request(&workspace.execution()).map_err(|cause| {
            crate::transactions::data::TransactionCommitError::execution(cause.into())
        })?;
    let (kind_id, old_state) = workspace.with_context(|context| {
        let partition = context
            .state
            .get_partition_mut_for_request(entity_id.partition_id, &context.execution)
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        let slot = partition.entity_arena.get(&entity_id).ok_or_else(|| {
            materialization_conflict(
                crate::transactions::data::RecordRef::Entity(entity_id),
                "entity suspension requires its exact live identity",
            )
        })?;
        let kind_id = slot.kind_id().ok_or_else(|| {
            materialization_conflict(
                crate::transactions::data::RecordRef::Entity(entity_id),
                "entity suspension requires retained kind identity",
            )
        })?;
        unavailable.lineage_id = slot.extra().lineage_id;
        let old_state = Some(slot.retained_extra());
        partition
            .entity_arena
            .suspend_materialization_with_extra_for_request(
                &context.execution,
                entity_id.slot_index(),
                unavailable,
            )
            .map_err(|denial| match denial {
                crate::storage::substrate::MetadataWriteDenial::Request(cause) => {
                    crate::transactions::data::TransactionCommitError::execution(cause.into())
                }
                crate::storage::substrate::MetadataWriteDenial::Invalid(detail) => {
                    materialization_conflict(
                        crate::transactions::data::RecordRef::Entity(entity_id),
                        detail,
                    )
                    .into()
                }
            })?;
        context
            .state
            .mark_entity_slot_touched_for_request(
                entity_id.partition_id,
                entity_id.slot_index(),
                &context.execution,
            )
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        Ok::<_, crate::transactions::data::TransactionCommitError>((kind_id, old_state))
    })?;
    let mut outcome = MutationOutcome::with_capacity(&workspace.execution(), 1, 1)?;
    outcome.record_change(
        &workspace.execution(),
        RecordMutation::EntityMaterializationSuspended {
            entity_id,
            kind_id,
            old_authoritative_aspect_state: old_state,
        },
    )?;
    outcome.record_event(
        &workspace.execution(),
        MutationEvent::EntityUpdated { entity_id },
    )?;
    Ok(outcome)
}

fn suspend_relation(
    relation_id: crate::identity::data::RelationId,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let unavailable =
        RelationExtra::empty_for_request(&workspace.execution()).map_err(|cause| {
            crate::transactions::data::TransactionCommitError::execution(cause.into())
        })?;
    let (kind_id, endpoints, old_state) = workspace.with_context(|context| {
        let partition = context
            .state
            .get_partition_mut_for_request(relation_id.partition_id, &context.execution)
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        let slot = partition.relation_arena.get(&relation_id).ok_or_else(|| {
            materialization_conflict(
                crate::transactions::data::RecordRef::Relation(relation_id),
                "relation suspension requires its exact live identity",
            )
        })?;
        let kind_id = slot.kind_id().ok_or_else(|| {
            materialization_conflict(
                crate::transactions::data::RecordRef::Relation(relation_id),
                "relation suspension requires retained kind identity",
            )
        })?;
        let endpoints = slot.extra().endpoints.clone().ok_or_else(|| {
            materialization_conflict(
                crate::transactions::data::RecordRef::Relation(relation_id),
                "live relation suspension requires exact endpoints",
            )
        })?;
        let old_state = Some(slot.retained_extra());
        partition
            .relation_arena
            .suspend_materialization_with_extra_for_request(
                &context.execution,
                relation_id.slot_index(),
                unavailable,
            )
            .map_err(|denial| match denial {
                crate::storage::substrate::MetadataWriteDenial::Request(cause) => {
                    crate::transactions::data::TransactionCommitError::execution(cause.into())
                }
                crate::storage::substrate::MetadataWriteDenial::Invalid(detail) => {
                    materialization_conflict(
                        crate::transactions::data::RecordRef::Relation(relation_id),
                        detail,
                    )
                    .into()
                }
            })?;
        context
            .state
            .mark_relation_slot_touched_for_request(
                relation_id.partition_id,
                relation_id.slot_index(),
                &context.execution,
            )
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        Ok::<_, crate::transactions::data::TransactionCommitError>((kind_id, endpoints, old_state))
    })?;
    let mut outcome = MutationOutcome::with_capacity(&workspace.execution(), 1, 1)?;
    outcome.record_change(
        &workspace.execution(),
        RecordMutation::RelationMaterializationSuspended {
            relation_id,
            kind_id,
            source: endpoints.source,
            target: endpoints.target,
            old_authoritative_aspect_state: old_state,
        },
    )?;
    outcome.record_event(
        &workspace.execution(),
        MutationEvent::RelationUpdated { relation_id },
    )?;
    Ok(outcome)
}

fn rematerialize_entity(
    intent: &crate::transactions::data::RematerializeEntityIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let target = RecordAspectPatchTarget::EntityCreation {
        kind_id: intent.kind_id,
    };
    let patch = record_aspect_patch::readmit_field_authoring(
        &workspace.execution(),
        &intent.fields,
        PortablePatchReadmissionPurpose::RecordCreation,
        workspace.entity_aspect_plan(intent.kind_id),
        target,
        FieldAuthoringDomain::Entity,
    )?;
    let mut state =
        record_aspect_patch::apply_entity(&workspace.execution(), None, &patch, target)?;
    let version_id = workspace.version_id();
    workspace.with_context(|context| {
        let partition = context
            .state
            .get_partition_mut_for_request(intent.entity_id.partition_id, &context.execution)
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        let lineage_id = partition
            .entity_arena
            .get(&intent.entity_id)
            .filter(|slot| slot.is_materialization_unavailable())
            .and_then(|slot| slot.extra().lineage_id);
        partition
            .entity_arena
            .rematerialize_for_request(
                &context.execution,
                &intent.entity_id,
                intent.kind_id,
                version_id,
                {
                    state.lineage_id = lineage_id;
                    state
                },
            )
            .map_err(|denial| {
                let detail = match denial {
                    crate::storage::substrate::MetadataWriteDenial::Request(cause) => {
                        return crate::transactions::data::TransactionCommitError::execution(
                            cause.into(),
                        )
                    }
                    crate::storage::substrate::MetadataWriteDenial::Invalid(detail) => detail,
                };
                materialization_conflict(
                    crate::transactions::data::RecordRef::Entity(intent.entity_id),
                    detail,
                )
                .into()
            })?;
        context
            .state
            .mark_entity_slot_touched_for_request(
                intent.entity_id.partition_id,
                intent.entity_id.slot_index(),
                &context.execution,
            )
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        Ok::<_, crate::transactions::data::TransactionCommitError>(())
    })?;
    let state = workspace.with_context(|context| {
        context
            .state
            .retained_entity_extra(intent.entity_id.partition_id, intent.entity_id.slot_index())
    });
    let mut outcome = MutationOutcome::with_capacity(&workspace.execution(), 1, 1)?;
    outcome.record_change(
        &workspace.execution(),
        RecordMutation::EntityRematerialized {
            entity_id: intent.entity_id,
            kind_id: intent.kind_id,
            new_authoritative_aspect_state: state,
            authoritative_patch: record_aspect_patch::published_patch(patch),
        },
    )?;
    outcome.record_event(
        &workspace.execution(),
        MutationEvent::EntityUpdated {
            entity_id: intent.entity_id,
        },
    )?;
    Ok(outcome)
}

fn materialization_conflict(
    record: crate::transactions::data::RecordRef,
    detail: impl Into<String>,
) -> CommitConflict {
    CommitConflict::new(ConflictClass::MutationStateInconsistency {
        detail: detail.into(),
        evidence:
            crate::transactions::data::MutationStateInconsistencyEvidence::MaterializationTransition {
                record,
            },
    })
}
