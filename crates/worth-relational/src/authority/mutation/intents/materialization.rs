use crate::authority::mutation::outcomes::{MutationEvent, MutationOutcome, RecordMutation};
use crate::authority::mutation::MutationWorkspace;
use crate::storage::substrate::{EntityExtra, RelationEndpoints, RelationExtra};
use crate::transactions::data::{
    CommitConflict, ConflictClass, MaterializationMutationIntent, RecordAspectPatchTarget,
};
use worth_foundational::facade::PortablePatchReadmissionPurpose;

use super::field_authoring_candidate::{self, FieldAuthoringDomain};
use super::{record_aspect_patch, relation_endpoint_candidate};

pub(super) fn apply(
    intent: &MaterializationMutationIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
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
) -> Result<MutationOutcome, CommitConflict> {
    let (kind_id, old_state) = workspace.with_context(|context| {
        let partition = context.state.get_partition_mut(entity_id.partition_id);
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
        let old_state = slot.extra().authoritative_aspect_state.clone();
        partition
            .entity_arena
            .suspend_materialization(entity_id.slot_index())
            .map_err(|detail| {
                materialization_conflict(
                    crate::transactions::data::RecordRef::Entity(entity_id),
                    detail,
                )
            })?;
        context
            .state
            .mark_entity_slot_touched(entity_id.partition_id, entity_id.slot_index());
        Ok((kind_id, old_state))
    })?;
    let mut outcome = MutationOutcome::with_capacity(1, 1);
    outcome.record_change(RecordMutation::EntityMaterializationSuspended {
        entity_id,
        kind_id,
        old_authoritative_aspect_state: old_state,
    });
    outcome.record_event(MutationEvent::EntityUpdated { entity_id });
    Ok(outcome)
}

fn suspend_relation(
    relation_id: crate::identity::data::RelationId,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    let (kind_id, endpoints, old_state) = workspace.with_context(|context| {
        let partition = context.state.get_partition_mut(relation_id.partition_id);
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
        let old_state = slot.extra().authoritative_aspect_state.clone();
        partition
            .relation_arena
            .suspend_materialization(relation_id.slot_index())
            .map_err(|detail| {
                materialization_conflict(
                    crate::transactions::data::RecordRef::Relation(relation_id),
                    detail,
                )
            })?;
        context
            .state
            .mark_relation_slot_touched(relation_id.partition_id, relation_id.slot_index());
        Ok((kind_id, endpoints, old_state))
    })?;
    let mut outcome = MutationOutcome::with_capacity(1, 1);
    outcome.record_change(RecordMutation::RelationMaterializationSuspended {
        relation_id,
        kind_id,
        source: endpoints.source,
        target: endpoints.target,
        old_authoritative_aspect_state: old_state,
    });
    outcome.record_event(MutationEvent::RelationUpdated { relation_id });
    Ok(outcome)
}

fn rematerialize_entity(
    intent: &crate::transactions::data::RematerializeEntityIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    let target = RecordAspectPatchTarget::EntityCreation {
        kind_id: intent.kind_id,
    };
    let patch = record_aspect_patch::readmit_field_authoring(
        &intent.fields,
        PortablePatchReadmissionPurpose::RecordCreation,
        workspace.entity_aspect_plan(intent.kind_id),
        target,
        FieldAuthoringDomain::Entity,
    )?;
    let state = record_aspect_patch::apply(None, &patch, target)?;
    let version_id = workspace.version_id();
    workspace.with_context(|context| {
        let partition = context
            .state
            .get_partition_mut(intent.entity_id.partition_id);
        let lineage_id = partition
            .entity_arena
            .get(&intent.entity_id)
            .filter(|slot| slot.is_materialization_unavailable())
            .and_then(|slot| slot.extra().lineage_id);
        partition
            .entity_arena
            .rematerialize(
                &intent.entity_id,
                intent.kind_id,
                version_id,
                EntityExtra {
                    lineage_id,
                    authoritative_aspect_state: state.clone(),
                    ..EntityExtra::default()
                },
            )
            .map_err(|detail| {
                materialization_conflict(
                    crate::transactions::data::RecordRef::Entity(intent.entity_id),
                    detail,
                )
            })?;
        context
            .state
            .mark_entity_slot_touched(intent.entity_id.partition_id, intent.entity_id.slot_index());
        Ok(())
    })?;
    let mut outcome = MutationOutcome::with_capacity(1, 1);
    outcome.record_change(RecordMutation::EntityRematerialized {
        entity_id: intent.entity_id,
        kind_id: intent.kind_id,
        new_authoritative_aspect_state: state,
        authoritative_patch: record_aspect_patch::published_patch(patch),
    });
    outcome.record_event(MutationEvent::EntityUpdated {
        entity_id: intent.entity_id,
    });
    Ok(outcome)
}

fn rematerialize_relation(
    intent: &crate::transactions::data::RematerializeRelationIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    let target_record = RecordAspectPatchTarget::RelationCreation {
        kind_id: intent.kind_id,
    };
    let plan = workspace.relation_aspect_plan(intent.kind_id);
    let candidate = field_authoring_candidate::lower(
        &intent.fields,
        PortablePatchReadmissionPurpose::RecordCreation,
        plan,
        intent.kind_id,
        FieldAuthoringDomain::Relation,
    )
    .map_err(|denial| record_aspect_patch::conflict(target_record, denial))?;
    let candidate = relation_endpoint_candidate::append_authoritative_endpoints(
        candidate,
        plan,
        intent.source,
        intent.target,
    );
    let patch = record_aspect_patch::readmit(
        candidate,
        PortablePatchReadmissionPurpose::RecordCreation,
        plan,
        target_record,
    )?;
    let state = record_aspect_patch::apply(None, &patch, target_record)?;
    let version_id = workspace.version_id();
    workspace.with_context(|context| {
        let partition = context
            .state
            .get_partition_mut(intent.relation_id.partition_id);
        partition
            .relation_arena
            .rematerialize(
                &intent.relation_id,
                intent.kind_id,
                version_id,
                RelationExtra {
                    endpoints: Some(RelationEndpoints {
                        source: intent.source,
                        target: intent.target,
                    }),
                    authoritative_aspect_state: state.clone(),
                },
            )
            .map_err(|detail| {
                materialization_conflict(
                    crate::transactions::data::RecordRef::Relation(intent.relation_id),
                    detail,
                )
            })?;
        context.state.mark_relation_slot_touched(
            intent.relation_id.partition_id,
            intent.relation_id.slot_index(),
        );
        Ok(())
    })?;
    let mut outcome = MutationOutcome::with_capacity(1, 1);
    outcome.record_change(RecordMutation::RelationRematerialized {
        relation_id: intent.relation_id,
        kind_id: intent.kind_id,
        source: intent.source,
        target: intent.target,
        new_authoritative_aspect_state: state,
        authoritative_patch: record_aspect_patch::published_patch(patch),
    });
    outcome.record_event(MutationEvent::RelationUpdated {
        relation_id: intent.relation_id,
    });
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
