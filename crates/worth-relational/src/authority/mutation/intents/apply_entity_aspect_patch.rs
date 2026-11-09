use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{
    ApplyEntityAspectPatchIntent, CommitConflict, ConflictClass, EntityFieldUpdateMissingState,
};

pub(super) fn apply(
    intent: &ApplyEntityAspectPatchIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    let slot = intent.entity_id.slot_index();
    let super::entity_aspect_preparation::PreparedEntityAspectMutation {
        kind_id,
        authoritative_aspect_state: old_state,
        new_authoritative_aspect_state: new_state,
        patch,
    } = workspace.prepare_entity_aspect_patch(intent)?;
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
            .ok_or_else(|| missing(intent, EntityFieldUpdateMissingState::Slot))?
            .clone();
        extra.authoritative_aspect_state = new_state.clone();
        partition
            .entity_arena
            .apply_extra_update(slot, extra, version_id)
            .ok_or_else(|| missing(intent, EntityFieldUpdateMissingState::Slot))?;
        Ok::<_, CommitConflict>(())
    })?;
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
