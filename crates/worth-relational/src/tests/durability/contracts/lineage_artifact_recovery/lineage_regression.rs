use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn tail_created_lineage_regression_below_checkpoint_high_water_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "lineage-regression-checkpoint");
    medium.file_embedded_checkpoint(execution, &runtime);
    create_entity_outcome(execution, &runtime, "lineage-regression-tail");
    let mut plan = medium
        .recovery_plan(execution, &runtime, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    plan.tail_log[0]
        .envelope_mut_for_test(execution)
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .targets[0] = crate::identity::data::LineageId(0);
    plan.tail_log[0]
        .envelope_mut_for_test(execution)
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(
        error
            .detail
            .contains("recovered allocated lineage id 0 does not advance"),
        "{error:?}"
    );
}

#[test]
fn tail_lineage_event_regression_below_checkpoint_high_water_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "event-regression-checkpoint");
    medium.file_embedded_checkpoint(execution, &runtime);
    create_entity_outcome(execution, &runtime, "event-regression-tail");
    let mut plan = medium
        .recovery_plan(execution, &runtime, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    plan.tail_log[0]
        .envelope_mut_for_test(execution)
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .event_id = 0;
    plan.tail_log[0]
        .envelope_mut_for_test(execution)
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("lineage event id 0 does not advance"));
}
