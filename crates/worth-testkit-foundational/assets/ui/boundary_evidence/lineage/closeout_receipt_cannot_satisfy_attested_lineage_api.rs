use worth_foundational::{
    boundary_evidence, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    BoundaryHandle, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceLineageSubject, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalCommitId,
    FoundationalCommitParentBasis, FoundationalCommitParentageLocator,
    FoundationalTransitionLocator,
};

fn main() {
    let _ = boundary_evidence()
        .lineage()
        .continuity(FoundationalBoundaryEvidenceLineageSubject::new(
            BoundaryHandle::new(4),
        ))
        .attested_by(
            boundary_evidence()
                .receipt()
                .blocked_closeout(FoundationalBoundaryEvidenceReceiptBoundary::transition(
                    FoundationalTransitionLocator::CommitParentage(FoundationalCommitParentageLocator::new(
                        FoundationalCommitId::new(BoundaryHandle::new(40)),
                        FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(41)),
                    )),
                ))
                .with_provenance(
                    boundary_evidence()
                        .provenance()
                        .replay_derived(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
                            BoundaryArtifactLocator::new(BoundaryArtifactId::new(4), BoundaryArtifactField::Basis),
                        ))
                        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay)
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
