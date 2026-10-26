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
) -> Result<MutationOutcome, CommitConflict> {
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
        Ok::<_, CommitConflict>((
            kind_id,
            slot_view.extra().authoritative_aspect_state.clone(),
        ))
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
    let new_authoritative_aspect_state =
        record_aspect_patch::apply(authoritative_aspect_state.as_ref(), &patch, target)?;
    workspace.with_context(|context| {
        context
            .state
            .mark_entity_slot_touched(intent.entity_id.partition_id, slot);
        let partition = context
            .state
            .get_partition_mut(intent.entity_id.partition_id);
        let mut updated_extra = partition
            .entity_arena
            .extra_at(slot)
            .expect("entity field update requires a materialized slot")
            .clone();
        updated_extra.authoritative_aspect_state = new_authoritative_aspect_state.clone();
        partition
            .entity_arena
            .apply_extra_update(slot, updated_extra, version_id);
        Ok::<_, CommitConflict>(())
    })?;
    Ok(MutationOutcome::entity_updated_with_authoritative_patch(
        intent.entity_id,
        kind_id,
        authoritative_aspect_state,
        new_authoritative_aspect_state,
        patch,
    ))
}
