pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::compare_canonical_basis,
    worth_foundational::prepare_branch_candidate_for_canonical_basis,
    worth_foundational::prepare_canonical_comparison,
    worth_foundational::prepare_commit_receipt_for_canonical_basis,
    worth_foundational::prepare_committed_authority_for_canonical_basis,
    worth_foundational::prepare_merge_verdict_for_canonical_basis,
    worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalEquivalenceBasis,
    worth_foundational::CanonicalizationRuleVersion,
};

pub(crate) fn version() -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new("milestone-5-phase-5").expect("version")
}

pub(crate) fn ready_candidate(
    candidate: worth_foundational::FoundationalBranchCandidateArtifact<&'static str>,
) -> worth_foundational::CanonicalBasisReadyArtifact {
    match prepare_branch_candidate_for_canonical_basis(version(), &candidate) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready candidate basis"),
    }
}

pub(crate) fn ready_verdict(
    verdict: worth_foundational::FoundationalMergeVerdict<&'static str>,
) -> worth_foundational::CanonicalBasisReadyArtifact {
    match prepare_merge_verdict_for_canonical_basis(version(), &verdict) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready verdict basis"),
    }
}

pub(crate) fn ready_committed(
    committed: worth_foundational::FoundationalCommittedAuthorityArtifact<&'static str>,
) -> worth_foundational::CanonicalBasisReadyArtifact {
    match prepare_committed_authority_for_canonical_basis(version(), &committed) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready committed basis"),
    }
}

pub(crate) fn ready_receipt(
    receipt: worth_foundational::FoundationalCommitReceiptArtifact,
) -> worth_foundational::CanonicalBasisReadyArtifact {
    ready_receipt_ref(&receipt)
}

pub(crate) fn ready_receipt_ref(
    receipt: &worth_foundational::FoundationalCommitReceiptArtifact,
) -> worth_foundational::CanonicalBasisReadyArtifact {
    match prepare_commit_receipt_for_canonical_basis(version(), receipt) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready receipt basis"),
    }
}

pub(crate) fn exact_compare(
    left: worth_foundational::CanonicalBasisReadyArtifact,
    right: worth_foundational::CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };
    compare_canonical_basis(&ready)
}

pub(crate) fn assert_equivalent(
    left: worth_foundational::CanonicalBasisReadyArtifact,
    right: worth_foundational::CanonicalBasisReadyArtifact,
) {
    assert!(matches!(
        exact_compare(left, right),
        worth_foundational::CanonicalComparisonOutcome::Equivalent(_)
    ));
}
