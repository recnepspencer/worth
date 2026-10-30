use worth_foundational::{
    boundary_evidence, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceSourceBasis, FoundationalBoundaryEvidenceSupportBasisDisclosure,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalCommitId,
    FoundationalCommitParentBasis, FoundationalCommitParentageLocator, BoundaryHandle,
    FoundationalTransitionLocator,
};

fn main() {
    let support = boundary_evidence()
        .support()
        .published_evidence()
        .with_basis_disclosure(FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedBasis)
        .attested_by(
            boundary_evidence()
                .receipt()
                .support_publication(FoundationalBoundaryEvidenceReceiptBoundary::transition(
                    FoundationalTransitionLocator::CommitParentage(FoundationalCommitParentageLocator::new(
                        FoundationalCommitId::new(BoundaryHandle::new(60)),
                        FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(61)),
                    )),
                ))
                .with_provenance(
                    boundary_evidence()
                        .provenance()
                        .replay_derived(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
                            BoundaryArtifactLocator::new(BoundaryArtifactId::new(6), BoundaryArtifactField::Basis),
                        ))
                        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay)
                        .unwrap_success(),
                ),
        )
        .unwrap_success();

    let _ = support.completed_boundary();
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
