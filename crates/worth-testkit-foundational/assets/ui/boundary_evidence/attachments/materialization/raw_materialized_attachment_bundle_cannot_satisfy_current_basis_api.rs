use worth_foundational::{
    boundary_evidence, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    BoundaryHandle, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceLineageSubject, FoundationalBoundaryEvidenceMaterializationProfile,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalCommitId, FoundationalCommitParentBasis, FoundationalCommitParentageLocator,
    FoundationalTransitionLocator,
};

fn accepts_current_basis_attachment(
    _: &worth_foundational::CurrentBasisBoundaryEvidenceAttachmentBundle,
) {
}

fn main() {
    let materialized = boundary_evidence()
        .attachment()
        .for_boundary_artifact(BoundaryArtifactLocator::new(
            BoundaryArtifactId::new(9),
            BoundaryArtifactField::Basis,
        ))
        .with_attested_continuity(
            boundary_evidence()
                .lineage()
                .continuity(FoundationalBoundaryEvidenceLineageSubject::new(
                    BoundaryHandle::new(9),
                ))
                .attested_by(
                    boundary_evidence()
                        .receipt()
                        .execution(FoundationalBoundaryEvidenceReceiptBoundary::transition(
                            FoundationalTransitionLocator::CommitParentage(
                                FoundationalCommitParentageLocator::new(
                                    FoundationalCommitId::new(BoundaryHandle::new(90)),
                                    FoundationalCommitParentBasis::new(
                                        worth_foundational::EquivalenceBasisId::new(91),
                                    ),
                                ),
                            ),
                        ))
                        .with_provenance(
                            boundary_evidence()
                                .provenance()
                                .historical(
                                    FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
                                        BoundaryArtifactLocator::new(
                                            BoundaryArtifactId::new(90),
                                            BoundaryArtifactField::Basis,
                                        ),
                                    ),
                                )
                                .with_freshness(
                                    FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
                                )
                                .unwrap_success(),
                        ),
                ),
        )
        .materialize_under(FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness);

    accepts_current_basis_attachment(&materialized);
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
