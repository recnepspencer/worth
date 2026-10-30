pub use {
    worth_foundational::foundational_committed_authority_admission,
    worth_foundational::EquivalenceBasisId,
    worth_foundational::FoundationalAuthorityTransitionClass,
    worth_foundational::FoundationalCommitDeltaSummary,
    worth_foundational::FoundationalCommitParentBasis,
    worth_foundational::FoundationalCommitParentage,
    worth_foundational::FoundationalCommittedAuthorityInput,
    worth_foundational::FoundationalCommittedDeltaLocus,
    worth_foundational::FoundationalMergeAncestryBasis, worth_foundational::FoundationalNoOpCause,
};
pub use {worth_proof::AuthorityWitness, worth_proof::TransitionOutcome};

pub use crate::certification::ordinary_context::transitions::fixtures::branch::branch_id;
pub use crate::certification::ordinary_context::transitions::fixtures::merge::authority_first_merge_candidate;

pub fn committed_authority(
) -> AuthorityWitness<worth_foundational::FoundationalCommittedAuthorityAdmission> {
    foundational_committed_authority_admission()
}

pub const fn parent_basis(value: u64) -> FoundationalCommitParentBasis {
    worth_foundational::FoundationalCommitParentBasis::new(
        worth_foundational::EquivalenceBasisId::new(value),
    )
}

pub fn unary_parentage() -> FoundationalCommitParentage {
    worth_foundational::FoundationalCommitParentage::new([parent_basis(401)])
        .expect("valid unary parentage")
}

pub fn multiparentage_unsorted() -> FoundationalCommitParentage {
    worth_foundational::FoundationalCommitParentage::new([
        parent_basis(406),
        parent_basis(401),
        parent_basis(403),
    ])
    .expect("valid multi-parent parentage")
}

pub const fn merge_ancestry_basis(value: u64) -> FoundationalMergeAncestryBasis {
    worth_foundational::FoundationalMergeAncestryBasis::new(
        worth_foundational::EquivalenceBasisId::new(value),
    )
}

pub fn delta_locus(category: &str, detail: &str) -> FoundationalCommittedDeltaLocus {
    worth_foundational::FoundationalCommittedDeltaLocus::new(category, detail)
}

pub fn ordinary_delta_summary() -> FoundationalCommitDeltaSummary {
    worth_foundational::FoundationalCommitDeltaSummary::new(vec![
        delta_locus("geometry-face", "face-7 updated"),
        delta_locus("geometry-edge", "edge-2 split"),
    ])
}

pub fn empty_delta_summary() -> FoundationalCommitDeltaSummary {
    worth_foundational::FoundationalCommitDeltaSummary::new(Vec::new())
}

pub fn ordinary_commit_input() -> FoundationalCommittedAuthorityInput {
    worth_foundational::FoundationalCommittedAuthorityInput::new(
        worth_foundational::FoundationalAuthorityTransitionClass::Commit,
        None,
        parent_basis(401),
        unary_parentage(),
        None,
        ordinary_delta_summary(),
    )
    .expect("ordinary commit input")
}

pub fn metadata_only_commit_input() -> FoundationalCommittedAuthorityInput {
    worth_foundational::FoundationalCommittedAuthorityInput::new(
        worth_foundational::FoundationalAuthorityTransitionClass::MetadataOnlyCommit,
        None,
        parent_basis(401),
        unary_parentage(),
        None,
        ordinary_delta_summary(),
    )
    .expect("metadata-only commit input")
}

pub fn promotion_commit_input() -> FoundationalCommittedAuthorityInput {
    worth_foundational::FoundationalCommittedAuthorityInput::new(
        worth_foundational::FoundationalAuthorityTransitionClass::PromotionCommit,
        None,
        parent_basis(401),
        multiparentage_unsorted(),
        Some(merge_ancestry_basis(499)),
        ordinary_delta_summary(),
    )
    .expect("promotion commit input")
}

pub fn replay_revalidated_commit_input() -> FoundationalCommittedAuthorityInput {
    worth_foundational::FoundationalCommittedAuthorityInput::new(
        worth_foundational::FoundationalAuthorityTransitionClass::ReplayRevalidatedCommit,
        None,
        parent_basis(401),
        unary_parentage(),
        Some(merge_ancestry_basis(577)),
        ordinary_delta_summary(),
    )
    .expect("replay revalidated commit input")
}

pub fn no_op_input(cause: FoundationalNoOpCause) -> FoundationalCommittedAuthorityInput {
    worth_foundational::FoundationalCommittedAuthorityInput::new(
        worth_foundational::FoundationalAuthorityTransitionClass::NoOp,
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
        worth_proof::TransitionOutcome::Success(verdict) => verdict,
        other => panic!("expected accepted verdict, got {other:?}"),
    }
}

pub fn advisory_verdict(
    payload: &'static str,
) -> worth_foundational::FoundationalMergeVerdict<&'static str> {
    match authority_first_merge_candidate(payload).admit_as_advisory() {
        worth_proof::TransitionOutcome::Success(verdict) => verdict,
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
        worth_proof::TransitionOutcome::Success(verdict) => verdict,
        other => panic!("expected conflict verdict, got {other:?}"),
    }
}

pub fn superseded_verdict(
    payload: &'static str,
) -> worth_foundational::FoundationalMergeVerdict<&'static str> {
    match authority_first_merge_candidate(payload).admit_as_superseded(branch_id("release")) {
        worth_proof::TransitionOutcome::Success(verdict) => verdict,
        other => panic!("expected superseded verdict, got {other:?}"),
    }
}
