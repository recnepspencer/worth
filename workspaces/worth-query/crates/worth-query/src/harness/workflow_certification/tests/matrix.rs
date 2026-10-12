use super::super::{
    MilestoneFivePointFiveWorkflowCertificationAdapter, WORKFLOW_REQUIRED_CANONICAL_ROW_NAMES,
    WORKFLOW_REQUIRED_REJECTION_ROW_NAMES,
};
use crate::harness::certification::{milestone_five_point_five_requirements, unmet_required_rows};

#[test]
fn workflow_certification_matrix_covers_required_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneFivePointFiveWorkflowCertificationAdapter::
        workflow_declaration_taxonomy_and_context_binding_test(request_execution);
    let requirements = milestone_five_point_five_requirements();
    assert_eq!(
        requirements.suite_name,
        "Query Workflow Lowering And Writeback Boundary Test"
    );
    assert_eq!(
        unmet_required_rows(
            &matrix,
            WORKFLOW_REQUIRED_CANONICAL_ROW_NAMES,
            WORKFLOW_REQUIRED_REJECTION_ROW_NAMES,
        ),
        Vec::<&'static str>::new()
    );
}

#[test]
fn workflow_certification_lanes_emit_required_verification_outputs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneFivePointFiveWorkflowCertificationAdapter::
        workflow_declaration_taxonomy_and_context_binding_test(request_execution);

    for row in &matrix.rows {
        for lane in [&row.control_lane, &row.hostile_lane, &row.parity_lane] {
            assert!(
                !lane.query_digest.is_empty(),
                "query digest must be present"
            );
            assert!(!lane.plan_digest.is_empty(), "plan digest must be present");
            assert!(
                !lane.result_digest.is_empty(),
                "result digest must be present"
            );
            assert!(
                !lane.delivery_digest.is_empty(),
                "delivery digest must be present"
            );
            assert!(
                !lane.failure_digest.is_empty(),
                "failure digest must be present"
            );
            assert!(
                !lane.counter_snapshot_digest.is_empty(),
                "counter snapshot digest must be present"
            );
        }
    }

    for row in &matrix.rejection_rows {
        assert!(
            !row.hostile_lane.failure_digest.is_empty(),
            "rejection failure digest must be present"
        );
        assert!(
            !row.hostile_lane.counter_snapshot_digest.is_empty(),
            "rejection counter snapshot digest must be present"
        );
    }
}
