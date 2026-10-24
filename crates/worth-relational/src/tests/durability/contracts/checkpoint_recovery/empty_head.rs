use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_empty_checkpoint_restores_a_live_retained_head() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    let identity = recovered.main_branch_identity();
    let (_, basis) = recovered
        .observe_branch(&identity)
        .expect("an empty recovered branch still has a complete owner root");
    assert_eq!(basis.observation().selected_root_identity(), 0);
    assert_eq!(recovered.retention_cost_counters().head_installs, 1);
}
