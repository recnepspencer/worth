use worth_foundational::{
    boundary_evidence, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    BoundaryHandle, FoundationalDiagnosticLocator, FoundationalTransitionLocator,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalCommitId,
    FoundationalCommitParentBasis, FoundationalCommitParentageLocator,
};

fn main() {
    let _ = boundary_evidence()
        .attachment()
        .for_diagnostic_bundle(FoundationalDiagnosticLocator::BoundaryArtifact(
            BoundaryArtifactLocator::new(BoundaryArtifactId::new(1), BoundaryArtifactField::Basis),
        ))
        .with_receipt_attachment(
            boundary_evidence()
                .receipt()
                .execution(FoundationalBoundaryEvidenceReceiptBoundary::transition(
                    FoundationalTransitionLocator::CommitParentage(
                        FoundationalCommitParentageLocator::new(
                            FoundationalCommitId::new(BoundaryHandle::new(2)),
                            FoundationalCommitParentBasis::new(
                                worth_foundational::EquivalenceBasisId::new(3),
                            ),
                        ),
                    ),
                ))
                .with_provenance(
                    boundary_evidence()
                        .provenance()
                        .historical(
                            worth_foundational::FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
                                BoundaryArtifactLocator::new(
                                    BoundaryArtifactId::new(4),
                                    BoundaryArtifactField::Basis,
                                ),
                            ),
                        )
                        .with_freshness(
                            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
                        )
                        .unwrap_success(),
                ),
        );
}

trait UnwrapSuccess<T> {
    fn unwrap_success(self) -> T;
}

impl<T, E> UnwrapSuccess<T> for worth_proof::TransitionOutcome<T, E> {
    fn unwrap_success(self) -> T {
        match self {
            worth_proof::TransitionOutcome::Success(value) => value,
            _ => panic!("expected success"),
        }
    }
}
