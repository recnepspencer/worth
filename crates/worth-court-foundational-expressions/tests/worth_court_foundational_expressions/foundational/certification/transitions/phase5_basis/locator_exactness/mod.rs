#[allow(unused_imports)] use crate::certification::ordinary_context::transitions::phase5_basis::locator_exactness::*;
use {worth_foundational::prepare_locator_for_canonical_basis, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalLocatorInput, worth_foundational::FoundationalBranchCandidateLocator, worth_foundational::FoundationalCommitParentageLocator, worth_foundational::FoundationalCommittedDeltaLocator, worth_foundational::FoundationalMergeConflictLocator, worth_foundational::FoundationalTransitionLocator};
use {worth_proof::TransitionOutcome};

use {crate::certification::ordinary_context::transitions::fixtures::branch::authority_first_candidate};
use {crate::certification::ordinary_context::transitions::fixtures::committed::accepted_verdict, crate::certification::ordinary_context::transitions::fixtures::committed::committed_authority, crate::certification::ordinary_context::transitions::fixtures::committed::ordinary_commit_input};
use {crate::certification::ordinary_context::transitions::fixtures::merge::authority_first_merge_candidate, crate::certification::ordinary_context::transitions::fixtures::merge::conflict_locus};
use {crate::certification::ordinary_context::transitions::fixtures::receipt::commit_id};
use {crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::version};

#[test]
fn transition_locators_point_at_exact_branch_conflict_parentage_and_delta_loci() {
    let branch_candidate = authority_first_candidate("mesh-update");
    let branch_entries = locator_entries(worth_foundational::FoundationalTransitionLocator::BranchCandidate(
        worth_foundational::FoundationalBranchCandidateLocator::new(
            branch_candidate.branch_id().clone(),
            branch_candidate.candidate_id(),
        ),
    ));
    assert_eq!(
        branch_entries,
        vec![
            transition_locator_text_entry(
                "transition.branch_candidate.branch_id",
                "feature/geometry"
            ),
            transition_locator_integer_entry("transition.branch_candidate.candidate_id", 17),
            transition_locator_text_entry("transition.branch_candidate.kind", "branch-candidate"),
        ]
    );

    let merge_candidate = authority_first_merge_candidate("mesh-update");
    let conflict_entries = locator_entries(worth_foundational::FoundationalTransitionLocator::MergeConflict(
        worth_foundational::FoundationalMergeConflictLocator::new(
            merge_candidate.source_branch().clone(),
            merge_candidate.target_branch().clone(),
            conflict_locus(),
        ),
    ));
    assert_eq!(
        conflict_entries,
        vec![
            transition_locator_text_entry("transition.merge_conflict.category", "geometry-face"),
            transition_locator_text_entry("transition.merge_conflict.kind", "merge-conflict"),
            transition_locator_text_entry(
                "transition.merge_conflict.source_branch",
                "feature/geometry"
            ),
            transition_locator_text_entry(
                "transition.merge_conflict.source_detail",
                "source:face-7"
            ),
            transition_locator_text_entry("transition.merge_conflict.target_branch", "main"),
            transition_locator_text_entry(
                "transition.merge_conflict.target_detail",
                "target:face-7"
            ),
        ]
    );

    let committed = accepted_verdict("mesh-update")
        .commit_with(ordinary_commit_input(), committed_authority())
        .expect("committed authority");
    let parentage_entries = locator_entries(worth_foundational::FoundationalTransitionLocator::CommitParentage(
        worth_foundational::FoundationalCommitParentageLocator::new(commit_id(81), committed.parent_basis()),
    ));
    assert_eq!(
        parentage_entries,
        vec![
            transition_locator_integer_entry("transition.parentage.commit_id", 81),
            transition_locator_text_entry("transition.parentage.kind", "commit-parentage"),
            transition_locator_integer_entry("transition.parentage.parent_basis", 401),
        ]
    );

    let delta_entries = locator_entries(worth_foundational::FoundationalTransitionLocator::CommittedDelta(
        worth_foundational::FoundationalCommittedDeltaLocator::new(
            commit_id(81),
            committed.committed_delta_summary().loci()[0].clone(),
        ),
    ));
    assert_eq!(
        delta_entries,
        vec![
            transition_locator_text_entry("transition.delta.category", "geometry-face"),
            transition_locator_integer_entry("transition.delta.commit_id", 81),
            transition_locator_text_entry("transition.delta.detail", "face-7 updated"),
            transition_locator_text_entry("transition.delta.kind", "committed-delta"),
        ]
    );
}
