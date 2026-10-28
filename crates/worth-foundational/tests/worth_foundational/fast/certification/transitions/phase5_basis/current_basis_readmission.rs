use worth_foundational::{
    admit_current_basis_commit_receipt, admit_current_basis_committed_authority,
    foundational_transition_current_basis_authority,
    foundational_transition_current_basis_readmission_authority,
    readmit_current_basis_commit_receipt_after_boundary,
    readmit_current_basis_committed_authority_after_boundary,
};
use worth_proof::TransitionOutcome;

use super::super::fixtures::committed::{
    accepted_verdict, committed_authority, ordinary_commit_input,
};
use super::super::fixtures::receipt::{commit_id, receipt_authority, receipt_identity};
use super::canonical_basis::{ready_committed, ready_receipt, version};

#[test]
fn current_basis_transition_lane_reuses_real_basis_preparation_and_explicit_readmission() {
    let committed = accepted_verdict("mesh-update")
        .commit_with(ordinary_commit_input(), committed_authority())
        .expect("committed authority");
    let expected_committed_basis = ready_committed(
        accepted_verdict("mesh-update")
            .commit_with(ordinary_commit_input(), committed_authority())
            .expect("committed authority"),
    );
    let strengthened_committed = match admit_current_basis_committed_authority(
        version(),
        committed,
        foundational_transition_current_basis_authority(),
    ) {
        TransitionOutcome::Success(artifact) => artifact,
        _ => panic!("expected current-basis committed artifact"),
    };
    assert_eq!(
        strengthened_committed.strong_basis().payload().entries(),
        expected_committed_basis.payload().entries()
    );

    let readmitted_committed = readmit_current_basis_committed_authority_after_boundary(
        worth_foundational::bridge_current_basis_committed_authority_trust_boundary(
            strengthened_committed,
        ),
        expected_committed_basis,
        foundational_transition_current_basis_readmission_authority(),
    );
    assert_eq!(
        readmitted_committed.committed().transition_class(),
        worth_foundational::FoundationalAuthorityTransitionClass::Commit
    );

    let receipt = accepted_verdict("mesh-update")
        .commit_with(ordinary_commit_input(), committed_authority())
        .expect("committed authority")
        .issue_receipt(receipt_identity(55), commit_id(45), receipt_authority())
        .expect("receipt");
    let expected_receipt_basis = ready_receipt(
        accepted_verdict("mesh-update")
            .commit_with(ordinary_commit_input(), committed_authority())
            .expect("committed authority")
            .issue_receipt(receipt_identity(55), commit_id(45), receipt_authority())
            .expect("receipt"),
    );
    let strengthened_receipt = match admit_current_basis_commit_receipt(
        version(),
        receipt,
        foundational_transition_current_basis_authority(),
    ) {
        TransitionOutcome::Success(artifact) => artifact,
        _ => panic!("expected current-basis receipt artifact"),
    };
    let readmitted_receipt = readmit_current_basis_commit_receipt_after_boundary(
        worth_foundational::bridge_current_basis_commit_receipt_trust_boundary(
            strengthened_receipt,
        ),
        expected_receipt_basis,
        foundational_transition_current_basis_readmission_authority(),
    );
    assert_eq!(readmitted_receipt.receipt().commit_id(), commit_id(45));
}
