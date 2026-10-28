use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::record_changes::delete_relation;
use crate::authority::mutation::stale_targets::ensure_relation_target_is_current;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{CommitConflict, DeleteRelationIntent};

pub(super) fn apply(
    intent: &DeleteRelationIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    let version_id = workspace.version_id();
    let branch_local_no_op = workspace.branch_local_delete_allows_relation(intent.relation_id);
    let mut outcome = MutationOutcome::relation_deleted(intent.relation_id);
    workspace.with_context(|context| {
        if let Err(error) = ensure_relation_target_is_current(context.state, intent.relation_id) {
            if branch_local_no_op {
                return Ok::<(), CommitConflict>(());
            }
            return Err(error);
        }
        delete_relation(context.state, version_id, intent.relation_id, &mut outcome)?;
        Ok::<(), CommitConflict>(())
    })?;
    Ok(outcome)
}
