use super::*;

#[test]
fn causal_inspection_certification_rejects_mislabeled_failure_row() {
    let error = CausalInspectionRepresentativeEvidence::from_failure(
        CausalInspectionRepresentativeKind::ChangedResult,
        "typed-failure-digest",
    )
    .unwrap_err();

    assert!(matches!(
        error.kind(),
        CausalInspectionCertificationErrorKind::RepresentativeMatrixMismatch
    ));
}
