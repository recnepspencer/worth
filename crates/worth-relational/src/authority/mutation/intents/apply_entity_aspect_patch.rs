use worth_foundational::facade::PortablePatchReadmissionPurpose;

use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::stale_targets::ensure_entity_target_is_current;
use crate::authority::mutation::MutationWorkspace;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::{
    ApplyEntityAspectPatchIntent, CommitConflict, ConflictClass, EntityFieldUpdateMissingState,
    RecordAspectPatchTarget,
};

use super::record_aspect_patch;

pub(super) fn apply(
    intent: &ApplyEntityAspectPatchIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    let slot = intent.entity_id.slot_index();
    let (kind_id, old_state) = workspace.with_context(|context| {
        ensure_entity_target_is_current(context.state, intent.entity_id)?;
        let partition = context
            .state
            .get_current_partition(intent.entity_id.partition_id)
            .ok_or_else(|| missing(intent, EntityFieldUpdateMissingState::Partition))?;
        let slot_view = partition
            .entity_arena()
            .get_slot(slot)
            .ok_or_else(|| missing(intent, EntityFieldUpdateMissingState::Slot))?;
        let kind_id = slot_view
            .kind_id()
            .ok_or_else(|| missing(intent, EntityFieldUpdateMissingState::KindId))?;
        Ok::<_, CommitConflict>((
            kind_id,
            slot_view.extra().authoritative_aspect_state.clone(),
        ))
    })?;
    let target = RecordAspectPatchTarget::Entity {
        entity_id: intent.entity_id,
        kind_id,
    };
    let patch = record_aspect_patch::readmit(
        intent.aspect_patch.clone(),
        PortablePatchReadmissionPurpose::RecordMutation,
        workspace.entity_aspect_plan(kind_id),
        target,
    )?;
    let new_state = record_aspect_patch::apply(old_state.as_ref(), &patch, target)?;
    let version_id = workspace.version_id();
    workspace.with_context(|context| {
        context
            .state
            .mark_entity_slot_touched(intent.entity_id.partition_id, slot);
        let partition = context
            .state
            .get_partition_mut(intent.entity_id.partition_id);
        let mut extra = partition
            .entity_arena
            .extra_at(slot)
            .expect("entity aspect patch requires a materialized slot")
            .clone();
        extra.authoritative_aspect_state = new_state.clone();
        partition
            .entity_arena
            .apply_extra_update(slot, extra, version_id);
    });
    Ok(MutationOutcome::entity_updated_with_authoritative_patch(
        intent.entity_id,
        kind_id,
        old_state,
        new_state,
        patch,
    ))
}

fn missing(
    intent: &ApplyEntityAspectPatchIntent,
    missing: EntityFieldUpdateMissingState,
) -> CommitConflict {
    CommitConflict::new(ConflictClass::EntityFieldUpdateStateInconsistency {
        entity_id: intent.entity_id,
        missing,
    })
}
