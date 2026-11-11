//! A live relation brought back under judgment without rewriting it.
use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::stale_targets::ensure_relation_target_is_current;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{RevalidateRelationIntent, TransactionCommitError};

pub(super) fn apply(
    intent: &RevalidateRelationIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, TransactionCommitError> {
    workspace.with_context(|context| {
        ensure_relation_target_is_current(context.state, intent.relation_id)?;
        context
            .state
            .mark_relation_slot_touched_for_request(
                intent.relation_id.partition_id,
                intent.relation_id.slot_index(),
                &context.execution,
            )
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        Ok::<(), TransactionCommitError>(())
    })?;
    Ok(MutationOutcome::record_revalidated())
}
