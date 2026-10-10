use super::super::MilestoneFivePointFiveWorkflowCertificationAdapter;

#[test]
fn workflow_certification_hostile_rows_are_distinct_when_spec_says_they_must_be() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneFivePointFiveWorkflowCertificationAdapter::
        workflow_declaration_taxonomy_and_context_binding_test(request_execution);

    for row in &matrix.rows {
        match row.row_name {
            "query-authored-mutation-lowering-parity"
            | "query-authored-merge-lowering-parity"
            | "query-triggered-writeback-lowering-parity"
            | "workflow-preview-foundation-no-rediscovery"
            | "workflow-rediscovery-zero-parity" => {
                assert_eq!(
                    row.hostile_lane.result_digest, row.control_lane.result_digest,
                    "row {} should preserve control result digest",
                    row.row_name
                );
            }
            _ => {
                assert_ne!(
                    (
                        row.hostile_lane.result_digest.clone(),
                        row.hostile_lane.delivery_digest.clone(),
                        row.hostile_lane.inspection_family.clone(),
                        row.hostile_lane.authority_outcome_family.clone(),
                    ),
                    (
                        row.control_lane.result_digest.clone(),
                        row.control_lane.delivery_digest.clone(),
                        row.control_lane.inspection_family.clone(),
                        row.control_lane.authority_outcome_family.clone(),
                    ),
                    "row {} should stay distinct from control in at least one verification surface",
                    row.row_name
                );
            }
        }
    }
}
