//! Real owner-issued custody cannot enter another owner's recovery routes.
use super::*;
use crate::recovery::{
    RuntimeWorldRecoveryDenial as Denial, RuntimeWorldSettledRelationalAdoptionDenial as Adoption,
};

#[test]
fn foreign_recovery_authority_refuses_release_continuation_and_adoption_without_effects() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (fixture, owner, expected) = setup();
    let (_foreign_fixture, foreign, _foreign_head) = setup();
    let before = foreign
        .state
        .branches
        .root_cell()
        .unwrap()
        .atomic_snapshot();
    let mut attempt = relational_attempt(execution, &fixture, &owner, expected.clone());
    attempt.begin_owner_execution();
    let performed = fixture.perform_relational_owner_change(execution);
    let basis = successor_basis(&owner, &expected, performed.next_basis().clone(), None);
    let retained = attempt
        .settle(CompositeAttemptProgress::new(
            RelationalAttemptProgress::performed(performed),
            SignalAttemptProgress::untouched(),
        ))
        .ready(basis)
        .expect_err("genuine unsettled owner custody");
    let handle = retained.recovery_handle();
    assert!(matches!(
        RuntimeWorldRecoveryService::release_effects(foreign.as_ref(), &handle, 0),
        Err(Denial::ForeignHandle)
    ));
    assert!(matches!(
        RuntimeWorldRecoveryService::continue_effects(
            foreign.as_ref(),
            owner.inspect_recovery(&handle).unwrap()
        ),
        Err(Denial::ForeignHandle)
    ));
    assert!(matches!(
        RuntimeWorldRecoveryService::prepare_settled_relational_adoption(
            foreign.as_ref(),
            &retained,
            &RuntimeWorldCancellationSource::new().token(),
            None
        ),
        Err(Adoption::Recovery(Denial::ForeignHandle))
    ));
    assert_eq!(foreign.recovery_record_count(), 0);
    assert_eq!(owner.recovery_record_count(), 1);
    let actual = foreign
        .state
        .branches
        .root_cell()
        .unwrap()
        .atomic_snapshot();
    assert_eq!(actual, before);
    assert!(owner.inspect_recovery(&handle).is_some());
}
