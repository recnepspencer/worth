use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_compaction_only_removes_segments_covered_by_checkpoint() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "a");
    create_entity_outcome(execution, &runtime, "b");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    create_entity_outcome(execution, &runtime, "c");
    let before = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap()
        .store
        .unwrap();

    let compaction = runtime.durability_authority().compact_store().unwrap();
    let after = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap()
        .store
        .unwrap();

    assert!(!before.segments.is_empty());
    assert!(after.segments.len() <= before.segments.len());
    assert_eq!(after.segments.len(), compaction.retained_segments.len());
}
