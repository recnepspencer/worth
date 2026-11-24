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
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    plan.tail_log_mut_for_test()[0]
        .readmitted_envelope_mut_for_test(execution)
        .published_lineage_mut_for_test()
        .lineage_events_mut_for_test()[0]
        .owner_targets_mut()[0] = crate::identity::data::LineageId(0);
    plan.tail_log_mut_for_test()[0]
        .readmitted_envelope_mut_for_test(execution)
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
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    *plan.tail_log_mut_for_test()[0]
        .readmitted_envelope_mut_for_test(execution)
        .published_lineage_mut_for_test()
        .lineage_events_mut_for_test()[0]
        .event_id_mut_for_test() = 0;
    plan.tail_log_mut_for_test()[0]
        .readmitted_envelope_mut_for_test(execution)
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("lineage event id 0 does not advance"));
}

#[test]
fn tail_lineage_commit_and_branch_cross_splices_deny() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "cross-splice-checkpoint");
    medium.file_embedded_checkpoint(execution, &runtime);
    create_entity_outcome(execution, &runtime, "cross-splice-tail");
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let event = plan.tail_log_mut_for_test()[0]
        .readmitted_envelope_mut_for_test(execution)
        .published_lineage_mut_for_test()
        .lineage_events_mut_for_test()
        .first_mut()
        .expect("tail event");
    event.owner_commit_mut().branch_id = BranchId("spliced-sibling".to_owned());
    *event.lineage_event_branch_mut_for_test() = BranchId("spliced-sibling".to_owned());
    plan.tail_log_mut_for_test()[0]
        .readmitted_envelope_mut_for_test(execution)
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("cross-spliced"));
}
