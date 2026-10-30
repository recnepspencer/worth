pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::boundary_evidence, worth_foundational::boundary_evidence_api::common_path,
    worth_foundational::boundary_evidence_api::lower_lane,
    worth_foundational::foundational_boundary_evidence_branch_divergence_posture_definitions,
    worth_foundational::foundational_boundary_evidence_lineage_outcome_kind_definitions,
    worth_foundational::foundational_boundary_evidence_lineage_partiality_posture_definitions,
    worth_foundational::foundational_boundary_evidence_promotion_posture_definitions,
    worth_foundational::BoundaryArtifactField, worth_foundational::BoundaryArtifactId,
    worth_foundational::BoundaryArtifactLocator, worth_foundational::BoundaryHandle,
    worth_foundational::FoundationalBoundaryEvidenceBranchDivergencePosture,
    worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture,
    worth_foundational::FoundationalBoundaryEvidenceLineageConstructionDenial,
    worth_foundational::FoundationalBoundaryEvidenceLineageOutcomeKind,
    worth_foundational::FoundationalBoundaryEvidenceLineagePartialityPosture,
    worth_foundational::FoundationalBoundaryEvidenceLineageSubject,
    worth_foundational::FoundationalBoundaryEvidenceLineageSubjectSet,
    worth_foundational::FoundationalBoundaryEvidenceLocality,
    worth_foundational::FoundationalBoundaryEvidencePromotionPosture,
    worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary,
    worth_foundational::FoundationalBoundaryEvidenceReceiptKind,
    worth_foundational::FoundationalBoundaryEvidenceSourceBasis,
    worth_foundational::FoundationalCommitId, worth_foundational::FoundationalCommitParentBasis,
    worth_foundational::FoundationalCommitParentageLocator,
    worth_foundational::FoundationalTransitionLocator,
};

pub(crate) use {
    crate::certification::ordinary_context::boundary_evidence::provenance::authority_path,
    crate::certification::ordinary_context::boundary_evidence::provenance::digest_basis,
    crate::certification::ordinary_context::boundary_evidence::provenance::profile_basis,
    crate::certification::ordinary_context::boundary_evidence::provenance::source_basis,
    crate::certification::ordinary_context::boundary_evidence::provenance::strategy_basis,
};

pub(crate) fn receipt_boundary(seed: u64) -> FoundationalBoundaryEvidenceReceiptBoundary {
    worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary::transition(
        worth_foundational::FoundationalTransitionLocator::CommitParentage(
            worth_foundational::FoundationalCommitParentageLocator::new(
                worth_foundational::FoundationalCommitId::new(
                    worth_foundational::BoundaryHandle::new(seed),
                ),
                worth_foundational::FoundationalCommitParentBasis::new(
                    worth_foundational::EquivalenceBasisId::new(seed + 10),
                ),
            ),
        ),
    )
}

pub(crate) fn subject(seed: u64) -> FoundationalBoundaryEvidenceLineageSubject {
    worth_foundational::FoundationalBoundaryEvidenceLineageSubject::new(
        worth_foundational::BoundaryHandle::new(seed),
    )
}

pub(crate) fn historical_provenance(
) -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
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

pub(crate) fn replay_provenance(
) -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    boundary_evidence()
        .provenance()
        .replay_derived(source_basis())
        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay)
        .expect_success("replay provenance")
}

pub(crate) fn restored_provenance(
) -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    boundary_evidence()
        .provenance()
        .restored_readmitted(worth_foundational::FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
            worth_foundational::BoundaryArtifactLocator::new(worth_foundational::BoundaryArtifactId::new(77), worth_foundational::BoundaryArtifactField::Basis),
        ))
        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::RestoredFromCheckpoint)
        .expect_success("restored provenance")
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
