use worth_foundational::{
    prepare_locator_for_canonical_basis, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalIntegerWidth,
    CanonicalLocatorInput, FoundationalBranchCandidateLocator, FoundationalCommitParentageLocator,
    FoundationalCommittedDeltaLocator, FoundationalMergeConflictLocator,
    FoundationalTransitionLocator,
};
use worth_proof::TransitionOutcome;

use super::super::fixtures::branch::authority_first_candidate;
use super::super::fixtures::committed::{
    accepted_verdict, committed_authority, ordinary_commit_input,
};
use super::super::fixtures::merge::{authority_first_merge_candidate, conflict_locus};
use super::super::fixtures::receipt::commit_id;
use super::canonical_basis::version;

#[test]
fn transition_locators_point_at_exact_branch_conflict_parentage_and_delta_loci() {
    let branch_candidate = authority_first_candidate("mesh-update");
    let branch_entries = locator_entries(FoundationalTransitionLocator::BranchCandidate(
        FoundationalBranchCandidateLocator::new(
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
    let conflict_entries = locator_entries(FoundationalTransitionLocator::MergeConflict(
        FoundationalMergeConflictLocator::new(
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
    let parentage_entries = locator_entries(FoundationalTransitionLocator::CommitParentage(
        FoundationalCommitParentageLocator::new(commit_id(81), committed.parent_basis()),
    ));
    assert_eq!(
        parentage_entries,
        vec![
            transition_locator_integer_entry("transition.parentage.commit_id", 81),
            transition_locator_text_entry("transition.parentage.kind", "commit-parentage"),
            transition_locator_integer_entry("transition.parentage.parent_basis", 401),
        ]
    );

    let delta_entries = locator_entries(FoundationalTransitionLocator::CommittedDelta(
        FoundationalCommittedDeltaLocator::new(
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

fn locator_entries(
    locator: FoundationalTransitionLocator,
) -> Vec<worth_foundational::CanonicalBasisEntry> {
    match prepare_locator_for_canonical_basis(version(), CanonicalLocatorInput::Transition(locator))
    {
        TransitionOutcome::Success(ready) => ready.payload().entries().to_vec(),
        _ => panic!("expected ready locator basis"),
    }
}

fn transition_locator_text_entry(locus: &str, value: &str) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Locator,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::TransitionLocator,
        CanonicalBasisValue::ExactText(value.into()),
    )
}

fn transition_locator_integer_entry(locus: &str, value: u64) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Locator,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::TransitionLocator,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u128::from(value),
        },
    )
}
