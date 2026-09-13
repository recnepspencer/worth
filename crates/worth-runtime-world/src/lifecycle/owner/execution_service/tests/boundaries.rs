use super::*;

#[test]
fn execution_rechecks_deadline_after_reservation_before_first_owner_effect() {
    let clock = MutableClock::new(0);
    let (_fixture, owner, expected) =
        setup_with_clock(RuntimeWorldClock::from_source(clock.clone()));
    let prepared = prepare_signal_with_deadline(
        &owner,
        &expected,
        None,
        Some(crate::lifecycle::RuntimeWorldInstant::from_ticks(5)),
    )
    .expect("the future deadline admits the complete attempt reservation");
    clock.set(5);

    let outcome = execute_with_empty_signal(&owner, prepared);
    assert!(matches!(
        outcome,
        OwnerExecutionOutcome::NoEffect(no_effect)
            if no_effect.cause() == crate::publication::NoEffectCause::DeadlineBeforeEffect
    ));
    assert_eq!(owner.recovery_record_count(), 0);
}
