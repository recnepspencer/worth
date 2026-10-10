use super::*;

#[test]
fn tail_created_lineage_regression_below_checkpoint_high_water_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "lineage-regression-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity_outcome(&runtime, "lineage-regression-tail");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    plan.tail_log[0]
        .envelope_mut_for_test()
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .targets[0] = crate::identity::data::LineageId(0);
    plan.tail_log[0]
        .envelope_mut_for_test()
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptSegment);
    assert!(error
        .detail
        .contains("create lineage id 0 does not advance"));
}
