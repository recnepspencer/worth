use worth_foundational::{
    foundational_committed_authority_admission, EquivalenceBasisId,
    FoundationalAuthorityTransitionClass, FoundationalCommitDeltaSummary,
    FoundationalCommitParentBasis, FoundationalCommitParentage,
    FoundationalCommittedAuthorityInput, FoundationalCommittedDeltaLocus,
    FoundationalMergeAncestryBasis, FoundationalNoOpCause,
};
use worth_proof::{AuthorityWitness, TransitionOutcome};

use super::branch::branch_id;
use super::merge::authority_first_merge_candidate;

pub fn committed_authority(
) -> AuthorityWitness<worth_foundational::FoundationalCommittedAuthorityAdmission> {
    foundational_committed_authority_admission()
}

pub const fn parent_basis(value: u64) -> FoundationalCommitParentBasis {
    FoundationalCommitParentBasis::new(EquivalenceBasisId::new(value))
}

pub fn unary_parentage() -> FoundationalCommitParentage {
    FoundationalCommitParentage::new([parent_basis(401)]).expect("valid unary parentage")
}

pub fn multiparentage_unsorted() -> FoundationalCommitParentage {
    FoundationalCommitParentage::new([parent_basis(406), parent_basis(401), parent_basis(403)])
        .expect("valid multi-parent parentage")
}

pub const fn merge_ancestry_basis(value: u64) -> FoundationalMergeAncestryBasis {
    FoundationalMergeAncestryBasis::new(EquivalenceBasisId::new(value))
}

pub fn delta_locus(category: &str, detail: &str) -> FoundationalCommittedDeltaLocus {
    FoundationalCommittedDeltaLocus::new(category, detail)
}

pub fn ordinary_delta_summary() -> FoundationalCommitDeltaSummary {
    FoundationalCommitDeltaSummary::new(vec![
        delta_locus("geometry-face", "face-7 updated"),
        delta_locus("geometry-edge", "edge-2 split"),
    ])
}

pub fn empty_delta_summary() -> FoundationalCommitDeltaSummary {
    FoundationalCommitDeltaSummary::new(Vec::new())
}

pub fn ordinary_commit_input() -> FoundationalCommittedAuthorityInput {
    FoundationalCommittedAuthorityInput::new(
        FoundationalAuthorityTransitionClass::Commit,
        None,
        parent_basis(401),
        unary_parentage(),
        None,
        ordinary_delta_summary(),
    )
    .expect("ordinary commit input")
}

pub fn metadata_only_commit_input() -> FoundationalCommittedAuthorityInput {
    FoundationalCommittedAuthorityInput::new(
        FoundationalAuthorityTransitionClass::MetadataOnlyCommit,
        None,
        parent_basis(401),
        unary_parentage(),
        None,
        ordinary_delta_summary(),
    )
    .expect("metadata-only commit input")
}

pub fn promotion_commit_input() -> FoundationalCommittedAuthorityInput {
    FoundationalCommittedAuthorityInput::new(
        FoundationalAuthorityTransitionClass::PromotionCommit,
        None,
        parent_basis(401),
        multiparentage_unsorted(),
        Some(merge_ancestry_basis(499)),
        ordinary_delta_summary(),
    )
    .expect("promotion commit input")
}

pub fn replay_revalidated_commit_input() -> FoundationalCommittedAuthorityInput {
    FoundationalCommittedAuthorityInput::new(
        FoundationalAuthorityTransitionClass::ReplayRevalidatedCommit,
        None,
        parent_basis(401),
        unary_parentage(),
        Some(merge_ancestry_basis(577)),
        ordinary_delta_summary(),
    )
    .expect("replay revalidated commit input")
}

pub fn no_op_input(cause: FoundationalNoOpCause) -> FoundationalCommittedAuthorityInput {
    FoundationalCommittedAuthorityInput::new(
        FoundationalAuthorityTransitionClass::NoOp,
        Some(cause),
        parent_basis(401),
        unary_parentage(),
        None,
        empty_delta_summary(),
    )
    .expect("no-op input")
}

pub fn accepted_verdict(
    payload: &'static str,
) -> worth_foundational::FoundationalMergeVerdict<&'static str> {
    match authority_first_merge_candidate(payload).admit_as_accepted() {
        TransitionOutcome::Success(verdict) => verdict,
        other => panic!("expected accepted verdict, got {other:?}"),
    }
}

pub fn advisory_verdict(
    payload: &'static str,
) -> worth_foundational::FoundationalMergeVerdict<&'static str> {
    match authority_first_merge_candidate(payload).admit_as_advisory() {
        TransitionOutcome::Success(verdict) => verdict,
        other => panic!("expected advisory verdict, got {other:?}"),
    }
}

pub fn conflict_verdict(
    payload: &'static str,
) -> worth_foundational::FoundationalMergeVerdict<&'static str> {
    let locus = worth_foundational::FoundationalMergeConflictLocus::new(
        "geometry-face",
        "source:face-7",
        "target:face-7",
    );
    match authority_first_merge_candidate(payload).admit_as_conflict(vec![locus]) {
        TransitionOutcome::Success(verdict) => verdict,
        other => panic!("expected conflict verdict, got {other:?}"),
    }
}

pub fn superseded_verdict(
    payload: &'static str,
) -> worth_foundational::FoundationalMergeVerdict<&'static str> {
    match authority_first_merge_candidate(payload).admit_as_superseded(branch_id("release")) {
        TransitionOutcome::Success(verdict) => verdict,
        other => panic!("expected superseded verdict, got {other:?}"),
    }
}
