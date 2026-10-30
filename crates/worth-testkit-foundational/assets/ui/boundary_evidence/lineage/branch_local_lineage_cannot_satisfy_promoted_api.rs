use worth_foundational::{
    boundary_evidence, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    BoundaryHandle, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceLineageSubject, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalCommitId,
    FoundationalCommitParentBasis, FoundationalCommitParentageLocator,
    FoundationalTransitionLocator,
};

fn main() {
    let branch_local = boundary_evidence()
        .lineage()
        .branch_local_replacement(FoundationalBoundaryEvidenceLineageSubject::new(
            BoundaryHandle::new(3),
        ))
        .attested_by(
            boundary_evidence()
                .receipt()
                .execution(FoundationalBoundaryEvidenceReceiptBoundary::transition(
                    FoundationalTransitionLocator::CommitParentage(FoundationalCommitParentageLocator::new(
                        FoundationalCommitId::new(BoundaryHandle::new(30)),
                        FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(31)),
                    )),
                ))
                .with_provenance(
                    boundary_evidence()
                        .provenance()
                        .branch_local(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
                            BoundaryArtifactLocator::new(BoundaryArtifactId::new(3), BoundaryArtifactField::Basis),
                        ))
                        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained)
                        .unwrap_success(),
                ),
        );

    let _ = branch_local.promotion_receipt();
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
