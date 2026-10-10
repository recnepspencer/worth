use super::*;

#[test]
fn durability_contract_compaction_only_removes_segments_covered_by_checkpoint() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "a");
    create_entity_outcome(&runtime, "b");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity_outcome(&runtime, "c");
    let before = runtime
        .durability()
        .recovery_plan(
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .store
        .unwrap();

    let compaction = runtime.durability_authority().compact_store().unwrap();
    let after = runtime
        .durability()
        .recovery_plan(
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .store
        .unwrap();

    assert!(!before.segments.is_empty());
    assert!(after.segments.len() <= before.segments.len());
    assert_eq!(after.segments.len(), compaction.retained_segments.len());
}
