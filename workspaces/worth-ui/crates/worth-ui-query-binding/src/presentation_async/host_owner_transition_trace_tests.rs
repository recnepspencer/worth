use super::tests::{basis, installed_owner};
use super::*;

#[test]
fn before_effects_rejection_removes_the_provisional_pending_proof_event() {
    let mut owner = installed_owner();
    let pending = owner.admit_pending(basis(91)).unwrap();
    assert_eq!(owner.transition_trace.len(), 1);
    assert_eq!(
        owner.transition_trace[0].kind(),
        WorthUiPresentationTransitionKind::Pending
    );

    owner.reject_before_effects(&pending).unwrap();

    assert!(owner.transition_trace.is_empty());
}

#[test]
fn duplicate_trace_requires_a_second_owner_certified_completion_submission() {
    let mut owner = installed_owner();
    let pending = owner.admit_pending(basis(92)).unwrap();

    owner.admit_presented(&pending).unwrap();
    assert_eq!(owner.transition_trace.len(), 2);

    assert!(matches!(
        owner.admit_presented(&pending),
        Err(WorthUiPresentationSettlementDenial::InvalidPendingReceipt)
    ));
    assert_eq!(owner.transition_trace.len(), 3);
    assert_eq!(
        owner.transition_trace[2].kind(),
        WorthUiPresentationTransitionKind::DuplicateCompletionRejected
    );
}
