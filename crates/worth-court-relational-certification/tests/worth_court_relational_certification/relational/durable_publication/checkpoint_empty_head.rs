//! An actual empty source checkpoint reopens a live admitted branch head.
use super::checkpoint_source_capture::capture_durably;
use worth_relational::facade::durability::RecoveryVerificationMode as RecoveryMode;
use worth_testkit_relational::{
    durable_medium::TestDurableMedium, durable_publication::durable_runtime,
};
#[test]
fn durability_contract_empty_checkpoint_restores_a_live_retained_head() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    capture_durably(execution, &runtime, &medium).unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    let identity = recovered.main_branch_identity();
    let (_, basis) = recovered
        .observe_branch(&identity)
        .expect("an empty recovered branch still has a complete owner root");
    assert_eq!(basis.observation().selected_root_identity(), 0);
}
