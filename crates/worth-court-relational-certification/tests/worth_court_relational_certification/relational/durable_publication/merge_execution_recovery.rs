//! Logical checkpoint then recovery, on the storage mode the original court used.
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{RecoveryOutcome, RecoveryVerificationMode},
    runtime::RelationalRuntime,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

/// Checkpoint `runtime`, file its native capture on the explicit original
/// medium, then recover `receiver`, which was built durable over that medium.
pub(super) fn checkpoint_and_recover_durable(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    medium: &Arc<TestDurableMedium>,
    receiver: RelationalRuntime,
) -> (RecoveryOutcome, RelationalRuntime) {
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    super::branch_history_recovery::checkpoint(execution, runtime, medium);
    super::branch_history_recovery::recover(execution, medium, receiver)
}

/// Checkpoint a Memory `runtime` and reconstruct a separate Memory `receiver`
/// from the source's retained canonical log.
pub(super) fn checkpoint_and_recover_memory(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    mut receiver: RelationalRuntime,
) -> (RecoveryOutcome, RelationalRuntime) {
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let outcome = receiver
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    (outcome, receiver)
}
