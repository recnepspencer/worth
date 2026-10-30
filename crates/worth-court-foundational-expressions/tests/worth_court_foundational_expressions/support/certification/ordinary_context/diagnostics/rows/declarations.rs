pub(crate) use worth_foundational::EquivalenceBasisId;
pub(crate) use worth_foundational::FoundationalMergeConflictLocator;
pub(crate) use {
    worth_foundational::foundational_diagnostic_boundary_artifact_subject,
    worth_foundational::foundational_diagnostic_branch_candidate_subject,
    worth_foundational::foundational_diagnostic_branch_discard_subject,
    worth_foundational::foundational_diagnostic_code,
    worth_foundational::foundational_diagnostic_commit_receipt_subject,
    worth_foundational::foundational_diagnostic_committed_authority_subject,
    worth_foundational::foundational_diagnostic_locator_boundary_artifact,
    worth_foundational::foundational_diagnostic_locator_transition,
    worth_foundational::foundational_diagnostic_merge_verdict_subject,
    worth_foundational::foundational_diagnostic_scope,
    worth_foundational::sort_foundational_diagnostic_rows,
    worth_foundational::BoundaryArtifactField, worth_foundational::BoundaryArtifactId,
    worth_foundational::BoundaryArtifactLocator, worth_foundational::BoundaryHandle,
    worth_foundational::FoundationalBranchCandidateId, worth_foundational::FoundationalBranchId,
    worth_foundational::FoundationalCommitId, worth_foundational::FoundationalCommitParentBasis,
    worth_foundational::FoundationalCommitReceiptIdentity,
    worth_foundational::FoundationalDiagnosticAbsenceCause,
    worth_foundational::FoundationalDiagnosticBreachClass,
    worth_foundational::FoundationalDiagnosticComparisonRow,
    worth_foundational::FoundationalDiagnosticDecisionRow,
    worth_foundational::FoundationalDiagnosticDenialClass,
    worth_foundational::FoundationalDiagnosticEvidencePosture,
    worth_foundational::FoundationalDiagnosticFailureRow,
    worth_foundational::FoundationalDiagnosticLocalityClaim,
    worth_foundational::FoundationalDiagnosticOutcomeKind,
    worth_foundational::FoundationalDiagnosticProvenanceReadyRow,
    worth_foundational::FoundationalDiagnosticRow,
    worth_foundational::FoundationalDiagnosticRowFamily,
    worth_foundational::FoundationalDiagnosticSemanticLabelSet,
    worth_foundational::FoundationalDiagnosticSeverity,
    worth_foundational::FoundationalDiagnosticSupportEvidencePosture,
    worth_foundational::FoundationalDiagnosticSupportRow,
    worth_foundational::FoundationalDiagnosticWidenedFalloutPosture,
    worth_foundational::FoundationalMergeConflictLocus,
    worth_foundational::FoundationalTransitionLocator,
};
pub(crate) use {
    worth_foundational::FoundationalCommittedDeltaLocator,
    worth_foundational::FoundationalCommittedDeltaLocus,
};

pub(crate) fn code(value: &str) -> worth_foundational::FoundationalDiagnosticCodeId {
    foundational_diagnostic_code(value).expect("valid diagnostic code")
}

pub(crate) fn scope(value: &str) -> worth_foundational::FoundationalDiagnosticScopeId {
    foundational_diagnostic_scope(value).expect("valid diagnostic scope")
}

pub(crate) fn branch(name: &str) -> FoundationalBranchId {
    worth_foundational::FoundationalBranchId::new(name).expect("valid branch id")
}

pub(crate) fn candidate(value: u64) -> FoundationalBranchCandidateId {
    worth_foundational::FoundationalBranchCandidateId::new(worth_foundational::BoundaryHandle::new(
        value,
    ))
}

pub(crate) fn commit(value: u64) -> FoundationalCommitId {
    worth_foundational::FoundationalCommitId::new(worth_foundational::BoundaryHandle::new(value))
}

pub(crate) fn receipt(value: u64) -> FoundationalCommitReceiptIdentity {
    worth_foundational::FoundationalCommitReceiptIdentity::new(
        worth_foundational::BoundaryHandle::new(value),
    )
}

pub(crate) fn delta_locus(category: &str, detail: &str) -> FoundationalCommittedDeltaLocus {
    worth_foundational::FoundationalCommittedDeltaLocus::new(category, detail)
}

pub(crate) fn labels<const N: usize>(values: [&str; N]) -> FoundationalDiagnosticSemanticLabelSet {
    worth_foundational::FoundationalDiagnosticSemanticLabelSet::new(values.into_iter().map(code))
}
