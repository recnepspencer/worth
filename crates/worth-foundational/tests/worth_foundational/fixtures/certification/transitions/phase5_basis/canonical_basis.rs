use worth_foundational::{
    compare_canonical_basis, prepare_branch_candidate_for_canonical_basis,
    prepare_canonical_comparison, prepare_commit_receipt_for_canonical_basis,
    prepare_committed_authority_for_canonical_basis, prepare_merge_verdict_for_canonical_basis,
    CanonicalComparisonOutcome, CanonicalEquivalenceBasis, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

pub(super) fn version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new("milestone-5-phase-5").expect("version")
}

pub(super) fn ready_candidate(
    candidate: worth_foundational::FoundationalBranchCandidateArtifact<&'static str>,
) -> worth_foundational::CanonicalBasisReadyArtifact {
    match prepare_branch_candidate_for_canonical_basis(version(), &candidate) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready candidate basis"),
    }
}

pub(super) fn ready_verdict(
    verdict: worth_foundational::FoundationalMergeVerdict<&'static str>,
) -> worth_foundational::CanonicalBasisReadyArtifact {
    match prepare_merge_verdict_for_canonical_basis(version(), &verdict) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready verdict basis"),
    }
}

pub(super) fn ready_committed(
    committed: worth_foundational::FoundationalCommittedAuthorityArtifact<&'static str>,
) -> worth_foundational::CanonicalBasisReadyArtifact {
    match prepare_committed_authority_for_canonical_basis(version(), &committed) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready committed basis"),
    }
}

pub(super) fn ready_receipt(
    receipt: worth_foundational::FoundationalCommitReceiptArtifact,
) -> worth_foundational::CanonicalBasisReadyArtifact {
    ready_receipt_ref(&receipt)
}

pub(super) fn ready_receipt_ref(
    receipt: &worth_foundational::FoundationalCommitReceiptArtifact,
) -> worth_foundational::CanonicalBasisReadyArtifact {
    match prepare_commit_receipt_for_canonical_basis(version(), receipt) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready receipt basis"),
    }
}

fn exact_compare(
    left: worth_foundational::CanonicalBasisReadyArtifact,
    right: worth_foundational::CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };
    compare_canonical_basis(&ready)
}

pub(super) fn assert_equivalent(
    left: worth_foundational::CanonicalBasisReadyArtifact,
    right: worth_foundational::CanonicalBasisReadyArtifact,
) {
    assert!(matches!(
        exact_compare(left, right),
        CanonicalComparisonOutcome::Equivalent(_)
    ));
}
