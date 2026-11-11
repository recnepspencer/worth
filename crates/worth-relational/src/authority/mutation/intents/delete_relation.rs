use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::record_changes::delete_relation;
use crate::authority::mutation::stale_targets::ensure_relation_target_is_current;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::DeleteRelationIntent;

pub(super) fn apply(
    intent: &DeleteRelationIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let version_id = workspace.version_id();
    let branch_local_no_op = workspace.branch_local_delete_allows_relation(intent.relation_id);
    let mut outcome =
        MutationOutcome::relation_deleted(&workspace.execution(), intent.relation_id)?;
    workspace.with_context(|context| {
        if let Err(error) = ensure_relation_target_is_current(context.state, intent.relation_id) {
            if branch_local_no_op {
                return Ok::<(), crate::transactions::data::TransactionCommitError>(());
            }
            return Err(error.into());
        }
        delete_relation(
            &context.execution,
            context.state,
            version_id,
            intent.relation_id,
            &mut outcome,
        )?;
        Ok::<(), crate::transactions::data::TransactionCommitError>(())
    })?;
    Ok(outcome)
}
