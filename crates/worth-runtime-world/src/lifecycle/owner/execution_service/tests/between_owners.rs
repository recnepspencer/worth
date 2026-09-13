//! The gate between a settled Relational effect and the Signal advance. Every
//! denial here stops before the Signal owner is contacted and retains exactly
//! the Relational effect that was already performed.

use std::thread;

use super::*;

/// A cancellation observed after the Relational effect and before the Signal
/// advance retains only that Relational effect, and the Signal owner is never
/// contacted.
#[test]
fn cancellation_between_the_relational_effect_and_the_signal_advance_retains_only_the_relational_effect(
) {
    let (fixture, owner, expected) = setup();
    let prepared = prepare_both_owners(&fixture, &owner, &expected, "cancel-between-owners");
    let (rehearsal, reached) = arm_rehearsal(
        owner.as_ref(),
        ExecutionRehearsalBoundary::BetweenOwnerEffects,
    );
    let source = RuntimeWorldCancellationSource::new();
    let token = source.token();
    let mut context = ();

    let outcome = thread::scope(|scope| {
        let source_ref = &source;
        let rehearsal_ref = &rehearsal;
        let canceller = scope.spawn(move || {
            await_boundary(&reached);
            source_ref.cancel();
            rehearsal_ref.release();
        });
        let outcome = RuntimeWorldOwnerExecutionService::execute_with_signal(
            owner.as_ref(),
            prepared,
            &mut context,
            &token,
            |_| Ok(()),
        );
        canceller
            .join()
            .expect("the cancelling thread completes within the rehearsal budget");
        outcome
    });

    assert_eq!(
        rehearsal.signal_advance_entries(),
        0,
        "the Signal owner was never contacted"
    );
    let record = retained(outcome);
    assert_retains_only_the_relational_effect(
        &record,
        ProductUnpublishedCause::CancellationAfterEffect,
    );
    drop(record);
}

/// A deadline that expires after the Relational effect and before the Signal
/// advance retains only that Relational effect, and the Signal owner is never
/// contacted.
#[test]
fn deadline_between_the_relational_effect_and_the_signal_advance_retains_only_the_relational_effect(
) {
    let clock = MutableClock::new(0);
    let (fixture, owner, expected) =
        setup_with_clock(RuntimeWorldClock::from_source(clock.clone()));
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = RuntimeWorldPreparationService::prepare_publication(
        owner.as_ref(),
        expected.clone(),
        CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate("deadline-between-owners"),
            ),
        &cancellation.token(),
        Some(crate::lifecycle::RuntimeWorldInstant::from_ticks(5)),
    )
    .expect("the future deadline admits the complete attempt reservation");
    let (rehearsal, reached) = arm_rehearsal(
        owner.as_ref(),
        ExecutionRehearsalBoundary::BetweenOwnerEffects,
    );
    let mut context = ();

    let outcome = thread::scope(|scope| {
        let clock_ref = &clock;
        let rehearsal_ref = &rehearsal;
        let expirer = scope.spawn(move || {
            await_boundary(&reached);
            clock_ref.set(5);
            rehearsal_ref.release();
        });
        let outcome = RuntimeWorldOwnerExecutionService::execute_with_signal(
            owner.as_ref(),
            prepared,
            &mut context,
            &cancellation.token(),
            |_| Ok(()),
        );
        expirer
            .join()
            .expect("the deadline thread completes within the rehearsal budget");
        outcome
    });

    assert_eq!(
        rehearsal.signal_advance_entries(),
        0,
        "the Signal owner was never contacted"
    );
    let record = retained(outcome);
    assert_retains_only_the_relational_effect(
        &record,
        ProductUnpublishedCause::DeadlineAfterEffect,
    );
    drop(record);
}

/// The third arm of the same gate, proved directly on the gate itself.
///
/// This is a gate-level unit proof, not an end-to-end race: any competing
/// publication that could replace the product head also moves a component
/// basis, so a racing attempt is denied at admission long before it reaches
/// this gate. The gate reports the cause only — it carries no observed head —
/// so the winner an attempt observes is proved where a caller is actually told
/// about it, on the no-effect surface in `failures.rs`.
#[test]
fn the_pre_advance_gate_denies_a_stale_product_head_before_the_signal_advance() {
    let (fixture, owner, expected) = setup_with_relational_source();
    let competing_ready = ready_relational_competitor(
        &fixture,
        owner.as_ref(),
        &expected,
        "stale-between-owners-competitor",
    );
    let (rehearsal, _reached) =
        arm_rehearsal(owner.as_ref(), ExecutionRehearsalBoundary::SignalAdvance);
    let cancellation = RuntimeWorldCancellationSource::new();
    assert!(
        owner
            .pre_advance_signal_gate(&expected, None, &cancellation.token())
            .is_ok(),
        "the gate admits the attempt while its exact head is current"
    );

    publish_ready_competing_head(owner.as_ref(), competing_ready, &expected);
    assert!(
        matches!(
            owner.pre_advance_signal_gate(&expected, None, &cancellation.token()),
            Err((
                ProductUnpublishedCause::StaleProductHead,
                crate::publication::NoEffectCause::StaleExpectedProductHead
            ))
        ),
        "a replaced product head stops the attempt before the Signal advance"
    );
    assert_eq!(
        rehearsal.signal_advance_entries(),
        0,
        "no outcome of this gate contacts the Signal owner"
    );
}
