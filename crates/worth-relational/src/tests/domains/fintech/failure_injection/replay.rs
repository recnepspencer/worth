use crate::facade::history::BranchId;
use crate::facade::replay::{
    RelationalReplayOutcome, RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode,
};
use crate::facade::runtime::RelationalRuntime;

pub(crate) fn drop_latest_parent_envelope_for_replay(runtime: &RelationalRuntime) -> Option<u64> {
    let latest = runtime.history().latest_commit()?.commit_id;
    let chain = runtime
        .history()
        .ancestor_closure_by_commit_id_order(latest);
    let parent = *chain.get(1)?;
    runtime
        .history_authority()
        .remove_commit_envelope_for_test(parent)
        .then_some(parent.0)
}

pub(crate) fn replay_latest_commit_on_wrong_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> Option<RelationalReplayOutcome> {
    let latest = runtime.history().latest_commit()?.commit_id;
    Some(
        runtime
            .replay_authority()
            .replay_commit(
                RelationalReplayRequest {
                    commit_id: latest,
                    branch_id: BranchId("wrong".to_string()),
                    execution_mode: ReplayExecutionMode::SerialDeterministic,
                    verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
                },
                execution,
            )
            .unwrap(),
    )
}
