//! SPEC-P4-005. The Runtime World cancellation source reaches a Signal advance
//! that is already in flight, because the token the seam hands the Signal owner
//! is the one embedded in that source.

use std::sync::mpsc::SyncSender;
use std::thread;

use super::*;

/// The channel ends the cancelling thread owns for one in-flight cancellation.
struct InFlightCancellation {
    reached: Receiver<ReachedExecutionBoundary>,
    mutation_entered: Receiver<()>,
    release_mutation: SyncSender<()>,
}

impl InFlightCancellation {
    /// Waits for the seam to name the token, releases execution into the
    /// advance uncancelled, waits for the caller's mutation body to park inside
    /// it, then cancels and releases that body.
    fn drive(self, rehearsal: &ExecutionRehearsal, source: &RuntimeWorldCancellationSource) {
        let signal_token = awaited_signal_token(&self.reached);
        rehearsal.release();
        self.mutation_entered
            .recv_timeout(REHEARSAL_HANDSHAKE_BUDGET)
            .expect("the caller's mutation body runs inside the advance within its budget");
        assert!(
            !signal_token.is_cancelled(),
            "the advance is in flight and uncancelled while its mutation body is parked"
        );
        source.cancel();
        assert!(
            signal_token.is_cancelled(),
            "one Runtime World cancel() reaches the token the in-flight advance holds"
        );
        self.release_mutation
            .send(())
            .expect("the parked mutation body is released exactly once");
    }
}

/// The exact record an in-flight cancellation leaves behind: the advance ran to
/// completion, so both owner effects are retained and the product never moved.
fn assert_retains_both_owner_effects(record: &crate::recovery::ProductUnpublishedOwnerEffects) {
    assert_eq!(
        record.cause(),
        ProductUnpublishedCause::CancellationAfterEffect,
        "the in-flight cancellation is honoured before the product moves"
    );
    assert_eq!(
        record.owner_effect_count(),
        2,
        "the completed advance is retained beside the settled Relational effect"
    );
    assert_eq!(
        record.progress().relational_posture(),
        RelationalAttemptProgressPosture::Settled
    );
    assert_eq!(
        record.progress().signal_posture(),
        SignalAttemptProgressPosture::Performed
    );
}

/// One `RuntimeWorldCancellationSource::cancel()` issued from another thread
/// while the caller's mutation body is parked *inside* the advance is observed
/// by the token that in-flight advance is holding.
///
/// The Signal owner consults that token at `preflight_cell_wait` and again at
/// `preflight_movement`, both of which are behind us once the mutation body
/// runs: past the movement permit the canonical transaction is atomic and is
/// not interrupted. So the honest outcome of a cancel landing here is that the
/// advance completes and the cancellation is honoured at the next Runtime World
/// boundary -- the product is never published, and the record retains both
/// owner effects under `CancellationAfterEffect`. A cancel that lands one step
/// earlier, before the owner takes its permit, is denied with no movement;
/// `signal_advance_receives_the_embedded_token_not_a_caller_token` proves that
/// arm.
#[test]
fn runtime_world_cancel_reaches_an_in_flight_signal_advance() {
    let (fixture, owner, expected) = setup();
    let prepared = prepare_both_owners(&fixture, &owner, &expected, "cancel-in-flight-advance");
    let (rehearsal, reached) =
        arm_rehearsal(owner.as_ref(), ExecutionRehearsalBoundary::SignalAdvance);
    let (entered_mutation, mutation_entered) = sync_channel::<()>(1);
    let (release_mutation, mutation_released) = sync_channel::<()>(1);
    let source = RuntimeWorldCancellationSource::new();
    let token = source.token();
    let mut context = ();

    let outcome = thread::scope(|scope| {
        let source_ref = &source;
        let rehearsal_ref = &rehearsal;
        let canceller = scope.spawn(move || {
            InFlightCancellation {
                reached,
                mutation_entered,
                release_mutation,
            }
            .drive(rehearsal_ref, source_ref);
        });
        let outcome = RuntimeWorldOwnerExecutionService::execute_with_signal(
            owner.as_ref(),
            prepared,
            &mut context,
            &token,
            |_| {
                entered_mutation
                    .send(())
                    .expect("the arming test observes the mutation body");
                mutation_released
                    .recv_timeout(REHEARSAL_HANDSHAKE_BUDGET)
                    .expect("the parked mutation body is released within its budget");
                Ok(())
            },
        );
        canceller
            .join()
            .expect("the cancelling thread completes within the rehearsal budget");
        outcome
    });

    assert_eq!(
        rehearsal.signal_advance_entries(),
        1,
        "the owner entered the Signal advance exactly once"
    );
    let record = retained(outcome);
    assert_retains_both_owner_effects(&record);
    drop(record);
}

/// The Signal owner is handed the token embedded in this publication's Runtime
/// World source. An unrelated caller's source cannot reach the advance, and
/// this source's `cancel()` does. Landing the cancel here, before the owner
/// takes its movement permit, denies the advance with no movement at all.
#[test]
fn signal_advance_receives_the_embedded_token_not_a_caller_token() {
    let (_fixture, owner, expected) = setup();
    let prepared = prepare_signal(&owner, &expected, None);
    let (rehearsal, reached) =
        arm_rehearsal(owner.as_ref(), ExecutionRehearsalBoundary::SignalAdvance);
    let source = RuntimeWorldCancellationSource::new();
    let unrelated = RuntimeWorldCancellationSource::new();
    let token = source.token();
    let mut context = ();

    let outcome = thread::scope(|scope| {
        let source_ref = &source;
        let unrelated_ref = &unrelated;
        let rehearsal_ref = &rehearsal;
        let observer = scope.spawn(move || {
            let signal_token = awaited_signal_token(&reached);
            unrelated_ref.cancel();
            assert!(
                !signal_token.is_cancelled(),
                "an unrelated caller's source cannot cancel this advance"
            );
            source_ref.cancel();
            assert!(
                signal_token.is_cancelled(),
                "the advance received the Signal token embedded in this publication's source"
            );
            rehearsal_ref.release();
        });
        let outcome = RuntimeWorldOwnerExecutionService::execute_with_signal(
            owner.as_ref(),
            prepared,
            &mut context,
            &token,
            |_| Ok(()),
        );
        observer
            .join()
            .expect("the observing thread completes within the rehearsal budget");
        outcome
    });

    assert!(
        matches!(
            outcome,
            OwnerExecutionOutcome::NoEffect(no_effect)
                if no_effect.cause() == crate::publication::NoEffectCause::CancelledBeforeEffect
        ),
        "a Signal-only advance denied with no movement leaves no owner effect"
    );
}
