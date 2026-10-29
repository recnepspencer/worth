use {worth_proof::TransitionOutcome};

use {crate::certification::ordinary_context::transitions::fixtures::branch::authority_first_candidate, crate::certification::ordinary_context::transitions::fixtures::branch::projection_shaped_candidate};
use {crate::certification::ordinary_context::transitions::fixtures::committed::accepted_verdict, crate::certification::ordinary_context::transitions::fixtures::committed::committed_authority, crate::certification::ordinary_context::transitions::fixtures::committed::ordinary_commit_input};
use {crate::certification::ordinary_context::transitions::fixtures::merge::authority_first_merge_candidate, crate::certification::ordinary_context::transitions::fixtures::merge::projection_shaped_merge_candidate};
use {crate::certification::ordinary_context::transitions::fixtures::receipt::commit_id, crate::certification::ordinary_context::transitions::fixtures::receipt::receipt_authority, crate::certification::ordinary_context::transitions::fixtures::receipt::receipt_identity};
use {crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::assert_equivalent, crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::ready_candidate, crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::ready_committed, crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::ready_receipt, crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::ready_verdict};

#[test]
fn transition_surfaces_canonicalize_the_same_across_independent_producers() {
    let left_candidate = ready_candidate(authority_first_candidate("mesh-update"));
    let right_candidate = ready_candidate(projection_shaped_candidate("mesh-update"));
    assert_equivalent(left_candidate, right_candidate);

    let left_verdict = ready_verdict(
        match authority_first_merge_candidate("mesh-update").admit_as_accepted() {
            worth_proof::TransitionOutcome::Success(verdict) => verdict,
            _ => panic!("expected accepted verdict"),
        },
    );
    let right_verdict = ready_verdict(
        match projection_shaped_merge_candidate("mesh-update").admit_as_accepted() {
            worth_proof::TransitionOutcome::Success(verdict) => verdict,
            _ => panic!("expected accepted verdict"),
        },
    );
    assert_equivalent(left_verdict, right_verdict);

    let left_committed = ready_committed(
        accepted_verdict("mesh-update")
            .commit_with(ordinary_commit_input(), committed_authority())
            .expect("committed authority"),
    );
    let right_committed = ready_committed(
        match projection_shaped_merge_candidate("mesh-update").admit_as_accepted() {
            worth_proof::TransitionOutcome::Success(verdict) => verdict,
            _ => panic!("expected accepted verdict"),
        }
        .commit_with(ordinary_commit_input(), committed_authority())
        .expect("committed authority"),
    );
    assert_equivalent(left_committed, right_committed);

    let left_receipt = ready_receipt(
        accepted_verdict("mesh-update")
            .commit_with(ordinary_commit_input(), committed_authority())
            .expect("committed authority")
            .issue_receipt(receipt_identity(90), commit_id(80), receipt_authority())
            .expect("receipt"),
    );
    let right_receipt = ready_receipt(
        match projection_shaped_merge_candidate("mesh-update").admit_as_accepted() {
            worth_proof::TransitionOutcome::Success(verdict) => verdict,
            _ => panic!("expected accepted verdict"),
        }
        .commit_with(ordinary_commit_input(), committed_authority())
        .expect("committed authority")
        .issue_receipt(receipt_identity(90), commit_id(80), receipt_authority())
        .expect("receipt"),
    );
    assert_equivalent(left_receipt, right_receipt);
}
