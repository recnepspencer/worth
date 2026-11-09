use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{
    CommitConflict, ConflictClass, EntityFieldUpdateMissingState, UpdateEntityFieldsIntent,
};

use super::entity_aspect_preparation::PreparedEntityAspectMutation;

pub(super) fn apply(
    intent: &UpdateEntityFieldsIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    let version_id = workspace.version_id();
    let slot = intent.entity_id.slot_index();
    let PreparedEntityAspectMutation {
        kind_id,
        authoritative_aspect_state,
        new_authoritative_aspect_state,
        patch,
    } = workspace.prepare_entity_field_update(intent)?;
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
            .ok_or_else(|| missing(intent, EntityFieldUpdateMissingState::Slot))?
            .clone();
        updated_extra.authoritative_aspect_state = new_authoritative_aspect_state.clone();
        partition
            .entity_arena
            .apply_extra_update(slot, updated_extra, version_id)
            .ok_or_else(|| missing(intent, EntityFieldUpdateMissingState::Slot))?;
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

fn missing(
    intent: &UpdateEntityFieldsIntent,
    missing: EntityFieldUpdateMissingState,
) -> CommitConflict {
    CommitConflict::new(ConflictClass::EntityFieldUpdateStateInconsistency {
        entity_id: intent.entity_id,
        missing,
    })
}
