pub(crate) use {worth_foundational::boundary_evidence, worth_foundational::boundary_evidence_api::common_path, worth_foundational::boundary_evidence_api::lower_lane, worth_foundational::foundational_boundary_evidence_support_basis_disclosure_definitions, worth_foundational::foundational_boundary_evidence_support_recovery_posture_definitions, worth_foundational::foundational_boundary_evidence_support_residual_debt_kind_definitions, worth_foundational::foundational_boundary_evidence_support_truth_kind_definitions, worth_foundational::BoundaryArtifactField, worth_foundational::BoundaryArtifactId, worth_foundational::BoundaryArtifactLocator, worth_foundational::BoundaryHandle, worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture, worth_foundational::FoundationalBoundaryEvidenceLineageSubject, worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary, worth_foundational::FoundationalBoundaryEvidenceSourceBasis, worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure, worth_foundational::FoundationalBoundaryEvidenceSupportConstructionDenial, worth_foundational::FoundationalBoundaryEvidenceSupportRecoveryPosture, worth_foundational::FoundationalBoundaryEvidenceSupportResidualDebtKind, worth_foundational::FoundationalBoundaryEvidenceSupportResidualDebtSet, worth_foundational::FoundationalBoundaryEvidenceSupportTruthKind, worth_foundational::FoundationalCommitId, worth_foundational::FoundationalCommitParentBasis, worth_foundational::FoundationalCommitParentageLocator, worth_foundational::FoundationalTransitionLocator};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) fn receipt_boundary(seed: u64) -> FoundationalBoundaryEvidenceReceiptBoundary {
    worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary::transition(
        worth_foundational::FoundationalTransitionLocator::CommitParentage(worth_foundational::FoundationalCommitParentageLocator::new(
            worth_foundational::FoundationalCommitId::new(worth_foundational::BoundaryHandle::new(seed)),
            worth_foundational::FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(
                seed + 10,
            )),
        )),
    )
}

pub(crate) fn source_basis() -> FoundationalBoundaryEvidenceSourceBasis {
    worth_foundational::FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(worth_foundational::BoundaryArtifactLocator::new(
        worth_foundational::BoundaryArtifactId::new(90),
        worth_foundational::BoundaryArtifactField::Basis,
    ))
}

pub(crate) fn replay_provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    boundary_evidence()
        .provenance()
        .replay_derived(source_basis())
        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay)
        .expect_success("replay provenance")
}

pub(crate) fn historical_provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    boundary_evidence()
        .provenance()
        .historical(source_basis())
        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
        .expect_success("historical provenance")
}

pub(crate) fn residual_debt() -> FoundationalBoundaryEvidenceSupportResidualDebtSet {
    worth_foundational::FoundationalBoundaryEvidenceSupportResidualDebtSet::new(vec![
        worth_foundational::FoundationalBoundaryEvidenceSupportResidualDebtKind::RebuildRequired,
        worth_foundational::FoundationalBoundaryEvidenceSupportResidualDebtKind::ReducedBasisLimitsParity,
    ])
    .expect("residual debt set")
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