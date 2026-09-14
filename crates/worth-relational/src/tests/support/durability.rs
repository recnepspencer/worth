use super::*;

pub(crate) fn create_branch_from_main(runtime: &RelationalRuntime, branch_name: &str) -> BranchId {
    let branch_id = BranchId(branch_name.to_string());
    runtime
        .history_authority()
        .fork_branch_from(branch_id.clone(), &BranchId("main".to_string()))
        .unwrap();
    branch_id
}

pub(crate) fn checkpoint_and_recover_with<F>(
    runtime: &RelationalRuntime,
    recovered_factory: F,
) -> (crate::runtime::RecoveryOutcome, RelationalRuntime)
where
    F: FnOnce() -> RelationalRuntime,
{
    runtime.durability_authority().checkpoint().unwrap();
    recover_with(runtime, recovered_factory)
}

pub(crate) fn recover_with<F>(
    runtime: &RelationalRuntime,
    recovered_factory: F,
) -> (crate::runtime::RecoveryOutcome, RelationalRuntime)
where
    F: FnOnce() -> RelationalRuntime,
{
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = recovered_factory();
    let outcome = recovered.durability_recovery().recover(plan).unwrap();
    (outcome, recovered)
}
