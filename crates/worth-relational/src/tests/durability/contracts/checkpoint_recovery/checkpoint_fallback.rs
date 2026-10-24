use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_damaged_checkpoint_refuses_rather_than_shortening_history() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "first");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    create_entity_outcome(execution, &runtime, "second");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    attached_test_medium(&runtime).damage_checkpoint_payload();

    let mut recovered = durable_runtime_with_test_schema();
    let refused = medium_recovery_plan(
        execution,
        &runtime,
        &mut recovered,
        RecoveryMode::NormalRecoveryVerification,
    )
    .expect_err("a damaged checkpoint is refused, never skipped");

    assert_eq!(
        expect_durability_failure(refused).class,
        RecoveryFailureClass::CorruptCheckpoint
    );
    assert!(recovered
        .history()
        .branch_head(&BranchId("main".to_string()))
        .is_none());
}
