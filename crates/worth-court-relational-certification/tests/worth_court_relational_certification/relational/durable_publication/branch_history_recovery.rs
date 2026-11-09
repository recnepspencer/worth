//! Checkpoint and recovery over explicitly retained original medium contents.
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{RecoveryOutcome, RecoveryVerificationMode},
    identity::VersionId,
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{durable_medium::TestDurableMedium, durable_publication::capture};

pub(super) fn checkpoint(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    medium: &Arc<TestDurableMedium>,
) {
    medium.file_checkpoint(capture(request, runtime));
}
pub(super) fn recover(
    request: &ExecutionRequest<'_, '_>,
    medium: &Arc<TestDurableMedium>,
    mut receiver: RelationalRuntime,
) -> (RecoveryOutcome, RelationalRuntime) {
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            request,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let outcome = receiver
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    (outcome, receiver)
}
pub(super) fn main_version(runtime: &RelationalRuntime) -> VersionId {
    runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1
        .observation()
        .version_id()
}

/// Observe an actual fixture branch through the owner's repeatable basis.
pub(super) fn current_head(
    runtime: &RelationalRuntime,
    branch: &worth_relational::facade::history::BranchId,
) -> Option<worth_relational::facade::history::RelationalCommitReceipt> {
    let identity = runtime.branch_identity(branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    runtime
        .history()
        .branch_head_for_observation(&basis.observation())
        .unwrap()
}
