use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

pub(crate) fn expect_durability_failure(
    stop: crate::durability::data::RelationalRecoveryStop,
) -> crate::durability::data::DurabilityError {
    match stop {
        crate::durability::data::RelationalRecoveryStop::Durability(error) => error,
        crate::durability::data::RelationalRecoveryStop::Allocation(cause) => {
            panic!("expected failed durable evidence, received allocation stop: {cause:?}")
        }
        crate::durability::data::RelationalRecoveryStop::Request(cause) => {
            panic!("expected failed durable evidence, received request stop: {cause:?}")
        }
    }
}

/// Fork `branch_name` from main on a serial request of its own, as a test
/// fixture step whose request is irrelevant to the behavior under test.
pub(crate) fn create_branch_from_main(runtime: &RelationalRuntime, branch_name: &str) -> BranchId {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let branch_id = BranchId(branch_name.to_string());
    runtime
        .history_authority()
        .fork_branch_from(execution, branch_id.clone(), &BranchId("main".to_string()))
        .unwrap();
    branch_id
}

/// Capture `runtime`'s native checkpoint and file it in its test medium under
/// the position it covers, as a home's `checkpoint()` does.
pub(crate) fn checkpoint_into_medium(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) {
    let native = runtime
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .expect("native checkpoint captures");
    attached_test_medium(runtime).file_checkpoint(native);
}

/// What `runtime`'s test medium would hand Relational on reopen.
pub(crate) fn durable_recovery_source(
    runtime: &RelationalRuntime,
) -> crate::durability::medium::RelationalDurableRecoverySource {
    attached_test_medium(runtime)
        .recovery_source()
        .expect("the test medium's artifacts keep intact frames")
}

/// Plan `recovering`'s reconstruction from what `source`'s test medium holds,
/// binding `recovering` to that medium first.
pub(crate) fn medium_recovery_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &RelationalRuntime,
    recovering: &mut RelationalRuntime,
    mode: RecoveryMode,
) -> Result<crate::durability::data::RecoveryPlan, crate::durability::data::RelationalRecoveryStop>
{
    let recovery_source = durable_recovery_source(source);
    bind_durable_medium(recovering, attached_test_medium(source).attachment());
    recovering
        .durability()
        .recovery_plan_from_source(execution, recovery_source, mode)
}

/// Checkpoint `runtime`; a durable runtime also files the native checkpoint
/// in its medium, as a home's `checkpoint()` does.
pub(crate) fn checkpoint_durably(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> Result<
    crate::durability::data::DurableCheckpoint,
    crate::durability::data::RelationalRecoveryStop,
> {
    let checkpoint = runtime.durability_authority().checkpoint(execution)?;
    if attached_durable_medium(runtime).is_some() {
        checkpoint_into_medium(execution, runtime);
    }
    Ok(checkpoint)
}

/// Plan reconstruction from where `runtime`'s history is held: its durable
/// test medium when it has one, otherwise its in-memory canonical log.
pub(crate) fn recovery_plan_for(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    mode: RecoveryMode,
) -> Result<crate::durability::data::RecoveryPlan, crate::durability::data::RelationalRecoveryStop>
{
    if attached_durable_medium(runtime).is_some() {
        runtime.durability().recovery_plan_from_source(
            execution,
            durable_recovery_source(runtime),
            mode,
        )
    } else {
        runtime.durability().recovery_plan(execution, mode)
    }
}

pub(crate) fn checkpoint_and_recover_with<F>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    recovered_factory: F,
) -> (crate::runtime::RecoveryOutcome, RelationalRuntime)
where
    F: FnOnce() -> RelationalRuntime,
{
    checkpoint_durably(execution, runtime).unwrap();
    recover_with(execution, runtime, recovered_factory)
}

/// Recover a fresh runtime from `runtime`'s durable medium when it has one,
/// otherwise from its in-memory canonical log. A recovered durable runtime is
/// bound to the source's medium, as a reopened home is.
pub(crate) fn recover_with<F>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    recovered_factory: F,
) -> (crate::runtime::RecoveryOutcome, RelationalRuntime)
where
    F: FnOnce() -> RelationalRuntime,
{
    let mut recovered = recovered_factory();
    let plan = if attached_durable_medium(runtime).is_some() {
        medium_recovery_plan(
            execution,
            runtime,
            &mut recovered,
            RecoveryMode::NormalRecoveryVerification,
        )
    } else {
        runtime
            .durability()
            .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
    }
    .unwrap();
    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    (outcome, recovered)
}
