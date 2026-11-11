//! Pure count conservation; no physical publication authority is manufactured.
use super::*;
use worth_foundational::facade::HomeIdentity;
fn identity() -> ProgramSupportObligationIdentity {
    ProgramSupportObligationIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([1; 16]).unwrap(),
        std::num::NonZeroU64::new(2).unwrap(),
    )
}
#[test]
fn committed_transfer_replaces_local_custody_without_double_counting() {
    let mut local = 1;
    let mut membership = InterpretationMembership::Local;
    membership.begin().unwrap();
    membership.validate_pending(local).unwrap();
    // Only the production authenticated fold can call this state transition.
    membership.committed(&mut local, identity());
    let durable = 1;
    assert_eq!(local + durable, 1);
    membership.abandon(&mut local);
    assert_eq!(
        local + durable,
        1,
        "cache Drop cannot discharge durable custody"
    );
    assert!(matches!(membership, InterpretationMembership::Durable(id) if id == identity()));
}
#[test]
fn unresolved_transfer_keeps_local_custody_when_the_token_is_abandoned() {
    let mut local = 1;
    let mut membership = InterpretationMembership::Local;
    membership.begin().unwrap();
    membership.abandon(&mut local);
    assert_eq!(local, 1);
    membership.validate_pending(local).unwrap();
    membership.committed(&mut local, identity());
    assert_eq!(local, 0);
    assert!(matches!(membership, InterpretationMembership::Durable(_)));
}
#[test]
fn proven_no_effect_restores_the_live_local_grant() {
    let mut local = 1;
    let mut membership = InterpretationMembership::Local;
    membership.begin().unwrap();
    membership.validate_pending(local).unwrap();
    membership.no_effect(&mut local);
    assert_eq!(local, 1);
    assert!(matches!(membership, InterpretationMembership::Local));
    membership.abandon(&mut local);
    assert_eq!(local, 0);
}
#[test]
fn proven_no_effect_disposes_an_abandoned_local_grant_once() {
    let mut local = 1;
    let mut membership = InterpretationMembership::Local;
    membership.begin().unwrap();
    membership.abandon(&mut local);
    membership.validate_pending(local).unwrap();
    membership.no_effect(&mut local);
    assert_eq!(local, 0);
    assert!(matches!(membership, InterpretationMembership::Disposed));
}
#[test]
fn a_transfer_without_local_custody_refuses_before_the_fold() {
    let mut membership = InterpretationMembership::Local;
    assert_eq!(
        membership.validate_pending(1),
        Err(QueryProgramSupportDenial::ObligationSubjectMismatch)
    );
    membership.begin().unwrap();
    assert_eq!(
        membership.validate_pending(0),
        Err(QueryProgramSupportDenial::ObligationSubjectMismatch)
    );
    assert_eq!(
        membership.begin(),
        Err(QueryProgramSupportDenial::ObligationSubjectMismatch)
    );
}
