use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::stale_targets::ensure_entity_target_is_current;
use crate::authority::mutation::MutationWorkspace;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::{
    CommitConflict, ConflictClass, EntityFieldUpdateMissingState, RecordAspectPatchTarget,
    UpdateEntityFieldsIntent,
};
use worth_foundational::facade::PortablePatchReadmissionPurpose;

use super::field_authoring_candidate::FieldAuthoringDomain;
use super::record_aspect_patch;

pub(super) fn apply(
    intent: &UpdateEntityFieldsIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let version_id = workspace.version_id();
    let slot = intent.entity_id.slot_index();
    let (kind_id, authoritative_aspect_state) = workspace.with_context(|context| {
        ensure_entity_target_is_current(context.state, intent.entity_id)?;
        let partition = context
            .state
            .get_current_partition(intent.entity_id.partition_id)
            .ok_or_else(|| {
                CommitConflict::new(ConflictClass::EntityFieldUpdateStateInconsistency {
                    entity_id: intent.entity_id,
                    missing: EntityFieldUpdateMissingState::Partition,
                })
            })?;
        let slot_view = partition.entity_arena().get_slot(slot).ok_or_else(|| {
            CommitConflict::new(ConflictClass::EntityFieldUpdateStateInconsistency {
                entity_id: intent.entity_id,
                missing: EntityFieldUpdateMissingState::Slot,
            })
        })?;
        let kind_id = slot_view.kind_id().ok_or_else(|| {
            CommitConflict::new(ConflictClass::EntityFieldUpdateStateInconsistency {
                entity_id: intent.entity_id,
                missing: EntityFieldUpdateMissingState::KindId,
            })
        })?;
        Ok::<_, CommitConflict>((kind_id, Some(slot_view.retained_extra())))
    })?;
    let target = RecordAspectPatchTarget::Entity {
        entity_id: intent.entity_id,
        kind_id,
    };
    let patch = record_aspect_patch::readmit_field_authoring(
        &intent.fields,
        PortablePatchReadmissionPurpose::RecordMutation,
        workspace.entity_aspect_plan(kind_id),
        target,
        FieldAuthoringDomain::Entity,
    )?;
    let mut new_authoritative_aspect_state = record_aspect_patch::apply_entity(
        &workspace.execution(),
        authoritative_aspect_state
            .as_ref()
            .and_then(|extra| extra.authoritative_aspect_state()),
        &patch,
        target,
    )?;
    workspace.with_context(|context| {
        context
            .state
            .mark_entity_slot_touched(intent.entity_id.partition_id, slot);
        let partition = context
            .state
            .get_partition_mut(intent.entity_id.partition_id);
        if let Some(prior) = &authoritative_aspect_state {
            new_authoritative_aspect_state.lineage_id = prior.lineage_id;
            new_authoritative_aspect_state.structural_fingerprint = prior.structural_fingerprint;
        }
        partition
            .entity_arena
            .apply_extra_update(slot, new_authoritative_aspect_state, version_id)
            .ok_or_else(|| missing(intent, EntityFieldUpdateMissingState::Slot))?;
        Ok::<_, CommitConflict>(())
    })?;
    let new_authoritative_aspect_state = workspace.with_context(|context| {
        context
            .state
            .get_partition_mut(intent.entity_id.partition_id)
            .entity_arena
            .retained_extra(slot)
    });
    Ok(MutationOutcome::entity_updated_with_authoritative_patch(
        intent.entity_id,
        kind_id,
        authoritative_aspect_state,
        new_authoritative_aspect_state,
        patch,
    ))
}

fn missing(
    intent: &UpdateEntityFieldsIntent,
    missing: EntityFieldUpdateMissingState,
) -> CommitConflict {
    CommitConflict::new(ConflictClass::EntityFieldUpdateStateInconsistency {
        entity_id: intent.entity_id,
        missing,
    })
}
