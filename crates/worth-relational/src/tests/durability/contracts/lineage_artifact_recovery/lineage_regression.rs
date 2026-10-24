use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn tail_created_lineage_regression_below_checkpoint_high_water_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "lineage-regression-checkpoint");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    create_entity_outcome(execution, &runtime, "lineage-regression-tail");
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    plan.tail_log[0]
        .envelope_mut_for_test()
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .targets[0] = crate::identity::data::LineageId(0);
    plan.tail_log[0]
        .envelope_mut_for_test()
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error
        .detail
        .contains("create lineage id 0 does not advance"));
}

#[test]
fn tail_lineage_event_regression_below_checkpoint_high_water_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "event-regression-checkpoint");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    create_entity_outcome(execution, &runtime, "event-regression-tail");
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    plan.tail_log[0]
        .envelope_mut_for_test()
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .event_id = 0;
    plan.tail_log[0]
        .envelope_mut_for_test()
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("lineage event id 0 does not advance"));
}
