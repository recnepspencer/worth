pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::attachment, worth_foundational::boundary_evidence,
    worth_foundational::boundary_evidence_api::common_path,
    worth_foundational::boundary_evidence_api::lower_lane,
    worth_foundational::boundary_evidence_api::stronger_lane,
    worth_foundational::BoundaryArtifactField, worth_foundational::BoundaryArtifactId,
    worth_foundational::BoundaryArtifactLocator, worth_foundational::BoundaryHandle,
    worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture,
    worth_foundational::FoundationalBoundaryEvidenceLineageSubject,
    worth_foundational::FoundationalBoundaryEvidenceMaterializationProfile,
    worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary,
    worth_foundational::FoundationalBoundaryEvidenceSourceBasis,
    worth_foundational::FoundationalCommitId, worth_foundational::FoundationalCommitParentBasis,
    worth_foundational::FoundationalCommitParentageLocator,
    worth_foundational::FoundationalTransitionLocator,
};

pub(crate) fn artifact_locator(id: u64) -> BoundaryArtifactLocator {
    worth_foundational::BoundaryArtifactLocator::new(
        worth_foundational::BoundaryArtifactId::new(id),
        worth_foundational::BoundaryArtifactField::Basis,
    )
}

pub(crate) fn transition_locator(id: u64) -> FoundationalTransitionLocator {
    worth_foundational::FoundationalTransitionLocator::CommitParentage(
        worth_foundational::FoundationalCommitParentageLocator::new(
            worth_foundational::FoundationalCommitId::new(worth_foundational::BoundaryHandle::new(
                id,
            )),
            worth_foundational::FoundationalCommitParentBasis::new(
                worth_foundational::EquivalenceBasisId::new(id + 10),
            ),
        ),
    )
}

pub(crate) fn provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    worth_foundational::boundary_evidence_api::common_path::provenance()
        .historical(
            worth_foundational::FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
                artifact_locator(1),
            ),
        )
        .with_freshness(
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
        )
        .expect_success("historical provenance")
}

pub(crate) fn receipt() -> worth_foundational::FoundationalBoundaryEvidenceExecutedReceiptArtifact {
    worth_foundational::boundary_evidence_api::common_path::receipt()
        .execution(
            worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary::transition(
                transition_locator(2),
            ),
        )
        .with_provenance(provenance())
}

pub(crate) fn support() -> worth_foundational::FoundationalBoundaryEvidencePublishedSupportArtifact
{
    worth_foundational::boundary_evidence_api::common_path::support()
        .published_evidence()
        .with_basis_disclosure(
            worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedBasis,
        )
        .attested_by(
            worth_foundational::boundary_evidence_api::common_path::receipt()
                .support_publication(
                    worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary::transition(
                        transition_locator(3),
                    ),
                )
                .with_provenance(provenance()),
        )
        .expect_success("support publication")
}

pub(crate) trait ExpectTransitionSuccess<T> {
    fn expect_success(self, label: &str) -> T;
}

impl<T, E> ExpectTransitionSuccess<T> for TransitionOutcome<T, E> {
    fn expect_success(self, label: &str) -> T {
        match self {
            worth_proof::TransitionOutcome::Success(value) => value,
            _ => panic!("expected {label} success"),
        }
    }
}
