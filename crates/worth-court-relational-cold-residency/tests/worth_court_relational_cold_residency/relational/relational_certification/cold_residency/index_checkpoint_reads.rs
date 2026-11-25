//! Checkpoint transport is observed through its real restore/read-generation consumer.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::RecoveryVerificationMode, indexes::DerivedIndexGeneration,
    runtime::RelationalRuntime, transactions::RelationalExecutionDenialCause as Execution,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium, durable_publication::durable_runtime,
};
pub(super) fn read(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    expected: &DerivedIndexGeneration,
) -> Result<bool, Execution> {
    let medium = TestDurableMedium::new();
    let mut receiver = durable_runtime(&medium);
    let image = runtime
        .durability_authority()
        .checkpoint(request)
        .map_err(super::checkpoint_read_refusal::memory_stop)?;
    let plan = runtime
        .durability()
        .recovery_plan(
            request,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .map_err(super::checkpoint_read_refusal::memory_stop)?;
    assert_eq!(plan.checkpoint.as_ref(), Some(&image));
    receiver
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    Ok(receiver
        .index_access()
        .read_latest_generation(request, expected.index_id, &expected.source_branch_id)
        .unwrap()
        .as_deref()
        == Some(expected))
}
