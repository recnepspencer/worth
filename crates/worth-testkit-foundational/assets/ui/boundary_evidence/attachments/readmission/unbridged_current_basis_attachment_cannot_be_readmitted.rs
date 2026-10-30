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
            BoundaryArtifactId::new(12),
            BoundaryArtifactField::Basis,
        ))
        .with_attested_continuity(
            boundary_evidence()
                .lineage()
                .continuity(FoundationalBoundaryEvidenceLineageSubject::new(
                    BoundaryHandle::new(12),
                ))
                .attested_by(
                    boundary_evidence()
                        .receipt()
                        .execution(FoundationalBoundaryEvidenceReceiptBoundary::transition(
                            FoundationalTransitionLocator::CommitParentage(
                                FoundationalCommitParentageLocator::new(
                                    FoundationalCommitId::new(BoundaryHandle::new(120)),
                                    FoundationalCommitParentBasis::new(
                                        worth_foundational::EquivalenceBasisId::new(121),
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
                                            BoundaryArtifactId::new(120),
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

    let admitted = stronger_lane::readmission::admit_current_basis_boundary_evidence_attachment_bundle(
        materialized,
        stronger_lane::readmission::foundational_boundary_evidence_attachment_readmission_authority(),
    );

    let _ = stronger_lane::readmission::readmit_current_basis_boundary_evidence_attachment_bundle_after_boundary(
        admitted,
        stronger_lane::readmission::foundational_boundary_evidence_attachment_readmission_authority(),
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
