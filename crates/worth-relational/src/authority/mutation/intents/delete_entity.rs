use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::record_changes::delete_entity_with_cascade;
use crate::authority::mutation::stale_targets::ensure_entity_target_is_current;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::DeleteEntityIntent;

pub(super) fn apply(
    intent: &DeleteEntityIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let version_id = workspace.version_id();
    let cascade_delete_policy = workspace.cascade_delete_policy();
    let branch_local_no_op = workspace.branch_local_delete_allows_entity(intent.entity_id);
    let mut outcome = MutationOutcome::entity_deleted(intent.entity_id);
    workspace.with_context(|context| {
        if let Err(error) = ensure_entity_target_is_current(context.state, intent.entity_id) {
            if branch_local_no_op {
                return Ok::<(), crate::transactions::data::TransactionCommitError>(());
            }
            return Err(error.into());
        }
        delete_entity_with_cascade(
            &context.execution,
            context.state,
            version_id,
            intent.entity_id,
            context.schema,
            cascade_delete_policy,
            &mut outcome,
        )?;
        Ok::<(), crate::transactions::data::TransactionCommitError>(())
    })?;
    Ok(outcome)
}
