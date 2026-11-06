use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
#[test]
fn candidate_capacity_workflow_settlement_reports_resource_admission() {
    let attempt = WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
        "workflow settlement",
    );
    let denied = WorthQueryApplicationCommitDenial::workflow_settlement_denied(&attempt);
    assert_eq!(
        denied.stage(),
        WorthQueryApplicationCommitDenialStage::ResourceAdmission
    );
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationCommitDenialKind::WorkflowSettlementDenied {
            kind: WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
        }
    );
}
