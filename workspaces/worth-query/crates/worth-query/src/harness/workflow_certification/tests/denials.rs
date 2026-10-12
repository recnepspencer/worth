use super::super::{MilestoneFivePointFiveWorkflowCertificationAdapter, WorkflowFailureClass};

#[test]
fn workflow_certification_denial_counters_are_exact_and_non_trivial() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneFivePointFiveWorkflowCertificationAdapter::
        workflow_declaration_taxonomy_and_context_binding_test(request_execution);

    let merge_family_denial = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "unsupported-merge-family")
        .expect("merge denial row should exist");
    let writeback_family_denial = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "unsupported-writeback-family")
        .expect("writeback denial row should exist");
    let explicit_rebind = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "explicit-rebind-required")
        .expect("explicit rebind row should exist");
    let stale_denial = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "stale-workflow-denied")
        .expect("stale denial row should exist");

    assert_eq!(
        merge_family_denial.hostile_lane.failure_class,
        WorkflowFailureClass::UnsupportedWorkflowFamily
    );
    assert_eq!(
        writeback_family_denial.hostile_lane.failure_class,
        WorkflowFailureClass::UnsupportedWorkflowFamily
    );
    assert_eq!(
        explicit_rebind.hostile_lane.failure_class,
        WorkflowFailureClass::ExplicitRebindRequired
    );
    assert_eq!(
        stale_denial.hostile_lane.failure_class,
        WorkflowFailureClass::StaleWorkflowDenied
    );
    assert_ne!(
        merge_family_denial.hostile_lane.counter_snapshot_digest,
        explicit_rebind.hostile_lane.counter_snapshot_digest
    );
    assert_ne!(
        writeback_family_denial.hostile_lane.counter_snapshot_digest,
        explicit_rebind.hostile_lane.counter_snapshot_digest
    );
    assert_ne!(
        stale_denial.hostile_lane.counter_snapshot_digest,
        explicit_rebind.hostile_lane.counter_snapshot_digest
    );
}
