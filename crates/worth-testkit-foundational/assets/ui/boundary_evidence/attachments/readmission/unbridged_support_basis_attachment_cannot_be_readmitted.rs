use worth_foundational::{
    boundary_evidence, boundary_evidence_api::stronger_lane, BoundaryArtifactField,
    BoundaryArtifactId, BoundaryArtifactLocator, BoundaryHandle,
    FoundationalBoundaryEvidenceFreshnessPosture, FoundationalBoundaryEvidenceLineageSubject,
    FoundationalBoundaryEvidenceMaterializationProfile, FoundationalBoundaryEvidenceReceiptBoundary,
    FoundationalBoundaryEvidenceSourceBasis, FoundationalCommitId, FoundationalCommitParentBasis,
    FoundationalCommitParentageLocator, FoundationalTransitionLocator,
};

fn main() {
    let materialized = boundary_evidence()
        .attachment()
        .for_boundary_artifact(BoundaryArtifactLocator::new(
            BoundaryArtifactId::new(22),
            BoundaryArtifactField::Basis,
        ))
        .with_attested_continuity(
            boundary_evidence()
                .lineage()
                .continuity(FoundationalBoundaryEvidenceLineageSubject::new(
                    BoundaryHandle::new(22),
                ))
                .attested_by(
                    boundary_evidence()
                        .receipt()
                        .execution(FoundationalBoundaryEvidenceReceiptBoundary::transition(
                            FoundationalTransitionLocator::CommitParentage(
                                FoundationalCommitParentageLocator::new(
                                    FoundationalCommitId::new(BoundaryHandle::new(220)),
                                    FoundationalCommitParentBasis::new(
                                        worth_foundational::EquivalenceBasisId::new(221),
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
                                            BoundaryArtifactId::new(220),
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
                                    FoundationalCommitId::new(BoundaryHandle::new(222)),
                                    FoundationalCommitParentBasis::new(
                                        worth_foundational::EquivalenceBasisId::new(223),
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
                                            BoundaryArtifactId::new(222),
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

    let admitted = stronger_lane::readmission::admit_support_basis_boundary_evidence_attachment_bundle(
        materialized,
        stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority(),
    )
    .expect("support basis admission");

    let _ = stronger_lane::readmission::readmit_support_basis_boundary_evidence_attachment_bundle_after_boundary(
        admitted,
        stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority(),
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
