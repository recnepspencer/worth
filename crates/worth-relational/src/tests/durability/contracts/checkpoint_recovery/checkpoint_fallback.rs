use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_corrupt_latest_checkpoint_falls_back_to_prior_valid_checkpoint() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "first");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let second = create_entity_outcome(execution, &runtime, "second");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let store = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap()
        .store
        .unwrap();
    let latest_checkpoint = store.checkpoints.last().unwrap();
    std::fs::write(&latest_checkpoint.path, b"{not-json").unwrap();

    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = persisted_runtime_with_test_schema();
    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert_eq!(outcome.latest_commit, Some(second.commit.clone()));
    assert!(!outcome
        .integrity_report
        .skipped_corrupt_checkpoints
        .is_empty());
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_string())),
        Some(second.commit.clone())
    );
    assert!(recovered
        .replay()
        .canonical_commit_envelope(first.commit.commit_id)
        .is_some());
}
