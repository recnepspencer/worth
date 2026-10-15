use super::*;

#[test]
fn support_matrix_matches_inventory_for_implemented_rows() {
    let inventory = worth_query_intent_admission_coverage_inventory();
    let support_matrix = worth_query_intent_admission_support_matrix();

    assert_eq!(inventory.rows().len(), support_matrix.rows().len());

    for (coverage, support) in inventory.rows().iter().zip(support_matrix.rows().iter()) {
        assert_eq!(coverage.family(), support.family());
        assert_eq!(coverage.entrypoint(), support.entrypoint());
        assert_eq!(coverage.execution_boundary(), support.execution_boundary());

        match coverage.status() {
            WorthQueryIntentAdmissionCoverageStatus::Implemented => {
                assert_eq!(
                    support.posture(),
                    WorthQueryIntentAdmissionSupportPosture::Admitted
                );
            }
            WorthQueryIntentAdmissionCoverageStatus::PlannedNeighbor => {
                assert_eq!(
                    support.posture(),
                    WorthQueryIntentAdmissionSupportPosture::Deferred
                );
            }
        }
    }
}
