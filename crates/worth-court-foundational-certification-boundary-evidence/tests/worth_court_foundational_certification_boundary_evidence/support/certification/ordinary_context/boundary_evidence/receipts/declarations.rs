pub(crate) use {worth_foundational::boundary_evidence, worth_foundational::boundary_evidence_api::common_path, worth_foundational::boundary_evidence_api::lower_lane, worth_foundational::foundational_boundary_evidence_closeout_disposition_definitions, worth_foundational::foundational_boundary_evidence_receipt_kind_definitions, worth_foundational::BoundaryArtifactField, worth_foundational::BoundaryArtifactId, worth_foundational::BoundaryArtifactLocator, worth_foundational::BoundaryHandle, worth_foundational::FoundationalBoundaryEvidenceCloseoutDisposition, worth_foundational::FoundationalBoundaryEvidenceExecutionPosture, worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary, worth_foundational::FoundationalBoundaryEvidenceReceiptKind, worth_foundational::FoundationalBoundaryEvidenceSourceBasis, worth_foundational::FoundationalCommitId, worth_foundational::FoundationalCommitParentBasis, worth_foundational::FoundationalCommitParentageLocator, worth_foundational::FoundationalTransitionLocator};

pub(crate) use {crate::certification::ordinary_context::boundary_evidence::provenance::authority_path, crate::certification::ordinary_context::boundary_evidence::provenance::digest_basis, crate::certification::ordinary_context::boundary_evidence::provenance::profile_basis, crate::certification::ordinary_context::boundary_evidence::provenance::source_basis, crate::certification::ordinary_context::boundary_evidence::provenance::strategy_basis};

pub(crate) fn receipt_boundary() -> FoundationalBoundaryEvidenceReceiptBoundary {
    worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary::transition(
        worth_foundational::FoundationalTransitionLocator::CommitParentage(worth_foundational::FoundationalCommitParentageLocator::new(
            worth_foundational::FoundationalCommitId::new(worth_foundational::BoundaryHandle::new(91)),
            worth_foundational::FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(17)),
        )),
    )
}

pub(crate) fn historical_provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    worth_foundational::boundary_evidence_api::common_path::provenance()
        .historical(source_basis())
        .authority_path(authority_path())
        .strategy_basis(strategy_basis())
        .profile_basis(profile_basis())
        .canonical_digest_basis(digest_basis())
        .with_freshness(
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
        )
        .expect_success("historical provenance")
}

pub(crate) trait ExpectTransitionSuccess<T> {
    fn expect_success(self, label: &str) -> T;
}

impl<T, E> ExpectTransitionSuccess<T> for worth_proof::TransitionOutcome<T, E>
where
    E: core::fmt::Debug,
{
    fn expect_success(self, label: &str) -> T {
        match self {
            worth_proof::TransitionOutcome::Success(value) => value,
            _ => panic!("expected {label} success"),
        }
    }
}