use super::entity_aspect_preparation::PreparedEntityAspectMutation;
use crate::authority::mutation::outcomes::MutationOutcome;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::authority::mutation::MutationWorkspace;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::conflict_class::{
    ConflictClass, EntityFieldUpdateMissingState,
};
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;

pub(super) fn apply(
    entity_id: EntityId,
    prepared: PreparedEntityAspectMutation,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, TransactionCommitError> {
    let slot = entity_id.slot_index();
    let PreparedEntityAspectMutation {
        kind_id,
        authoritative_aspect_state: old_state,
        new_authoritative_aspect_state: mut new_state,
        patch,
    } = prepared;
    let version_id = workspace.version_id();
    workspace.with_context(|context| {
        context
            .state
            .mark_entity_slot_touched_for_request(
                entity_id.partition_id,
                slot,
                &context.execution,
            )
            .map_err(|cause| {
                crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(cause.into())
            })?;
        let partition = context
            .state
            .get_partition_mut_for_request(entity_id.partition_id, &context.execution)
            .map_err(|cause| {
                crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(cause.into())
            })?;
        if let Some(prior) = &old_state {
            new_state.lineage_id = prior.lineage_id;
            new_state.structural_fingerprint = prior.structural_fingerprint;
        }
        partition
            .entity_arena
            .apply_extra_update_for_request(&context.execution, slot, new_state, version_id)
            .map_err(|cause| {
                crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(cause.into())
            })?
            .ok_or_else(|| missing_slot(entity_id))?;
        Ok::<_, TransactionCommitError>(())
    })?;
    let new_state = workspace.with_context(|context| {
        context
            .state
            .retained_entity_extra(entity_id.partition_id, slot)
    });
    MutationOutcome::entity_updated_with_authoritative_patch(
        &workspace.execution(),
        entity_id,
        kind_id,
        old_state,
        new_state,
        patch,
    )
}

fn missing_slot(entity_id: EntityId) -> CommitConflict {
    CommitConflict::new(ConflictClass::EntityFieldUpdateStateInconsistency {
        entity_id,
        missing: EntityFieldUpdateMissingState::Slot,
    })
}
