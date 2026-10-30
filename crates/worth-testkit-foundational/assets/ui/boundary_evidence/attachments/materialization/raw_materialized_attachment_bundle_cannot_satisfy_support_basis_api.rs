use worth_foundational::{
    boundary_evidence, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    BoundaryHandle, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceLineageSubject, FoundationalBoundaryEvidenceMaterializationProfile,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalCommitId, FoundationalCommitParentBasis, FoundationalCommitParentageLocator,
    FoundationalTransitionLocator,
};

fn accepts_support_basis_attachment(
    _: &worth_foundational::SupportBasisBoundaryEvidenceAttachmentBundle,
) {
}

fn main() {
    let materialized = boundary_evidence()
        .attachment()
        .for_boundary_artifact(BoundaryArtifactLocator::new(
            BoundaryArtifactId::new(19),
            BoundaryArtifactField::Basis,
        ))
        .with_attested_continuity(
            boundary_evidence()
                .lineage()
                .continuity(FoundationalBoundaryEvidenceLineageSubject::new(
                    BoundaryHandle::new(19),
                ))
                .attested_by(
                    boundary_evidence()
                        .receipt()
                        .execution(FoundationalBoundaryEvidenceReceiptBoundary::transition(
                            FoundationalTransitionLocator::CommitParentage(
                                FoundationalCommitParentageLocator::new(
                                    FoundationalCommitId::new(BoundaryHandle::new(190)),
                                    FoundationalCommitParentBasis::new(
                                        worth_foundational::EquivalenceBasisId::new(191),
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
                                            BoundaryArtifactId::new(190),
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
        .with_published_support(
            boundary_evidence()
                .support()
                .published_evidence()
                .with_basis_disclosure(
                    worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedBasis,
                )
                .attested_by(
                    boundary_evidence()
                        .receipt()
                        .support_publication(FoundationalBoundaryEvidenceReceiptBoundary::transition(
                            FoundationalTransitionLocator::CommitParentage(
                                FoundationalCommitParentageLocator::new(
                                    FoundationalCommitId::new(BoundaryHandle::new(192)),
                                    FoundationalCommitParentBasis::new(
                                        worth_foundational::EquivalenceBasisId::new(193),
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
                                            BoundaryArtifactId::new(192),
                                            BoundaryArtifactField::Basis,
                                        ),
                                    ),
                                )
                                .with_freshness(
                                    FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
                                )
                                .unwrap_success(),
                        ),
                )
                .expect_success("support publication"),
        )
        .materialize_under(FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness);

    accepts_support_basis_attachment(&materialized);
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

trait ExpectTransitionSuccess<T> {
    fn expect_success(self, label: &str) -> T;
}

impl<T, E> ExpectTransitionSuccess<T> for worth_proof::TransitionOutcome<T, E> {
    fn expect_success(self, label: &str) -> T {
        match self {
            worth_proof::TransitionOutcome::Success(value) => value,
            _ => panic!("expected {label} success"),
        }
    }
}
