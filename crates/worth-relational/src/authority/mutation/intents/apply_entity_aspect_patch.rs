use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{
    ApplyEntityAspectPatchIntent, CommitConflict, ConflictClass, EntityFieldUpdateMissingState,
};

pub(super) fn apply(
    intent: &ApplyEntityAspectPatchIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let slot = intent.entity_id.slot_index();
    let super::entity_aspect_preparation::PreparedEntityAspectMutation {
        kind_id,
        authoritative_aspect_state: old_state,
        new_authoritative_aspect_state: mut new_state,
        patch,
    } = workspace.prepare_entity_aspect_patch(intent)?;
    let version_id = workspace.version_id();
    workspace.with_context(|context| {
        context
            .state
            .mark_entity_slot_touched_for_request(
                intent.entity_id.partition_id,
                slot,
                &context.execution,
            )
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        let partition = context
            .state
            .get_partition_mut_for_request(intent.entity_id.partition_id, &context.execution)
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        if let Some(prior) = &old_state {
            new_state.lineage_id = prior.lineage_id;
            new_state.structural_fingerprint = prior.structural_fingerprint;
        }
        partition
            .entity_arena
            .apply_extra_update_for_request(&context.execution, slot, new_state, version_id)
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?
            .ok_or_else(|| missing(intent, EntityFieldUpdateMissingState::Slot))?;
        Ok::<_, crate::transactions::data::TransactionCommitError>(())
    })?;
    let new_state = workspace.with_context(|context| {
        context
            .state
            .retained_entity_extra(intent.entity_id.partition_id, slot)
    });
    MutationOutcome::entity_updated_with_authoritative_patch(
        &workspace.execution(),
        intent.entity_id,
        kind_id,
        old_state,
        new_state,
        patch,
    )
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
