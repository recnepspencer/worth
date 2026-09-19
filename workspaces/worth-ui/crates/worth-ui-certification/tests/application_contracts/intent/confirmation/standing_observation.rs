use worth_ui::facade::intent::{
    UiIntentConfirmationContinuation, UiIntentConfirmationStopReason,
    UiIntentConfirmationTimeBasisKind,
};
use worth_ui::facade::observation_report::UiHostObservationTimeBasis;

use super::world::ConfirmationWorld;

#[test]
fn confirmation_observation_preserves_challenge_at_expiry_and_during_continuation() {
    let (mut world, provider) = ConfirmationWorld::launch_with_provider_observation();
    let issued = world.issue();
    world.publish_successor();
    // A genuine host interaction supplies a sealed target for the fixture.
    // Every subsequent observation borrows it without producing another event.
    let route = world.confirmation_route(monotonic(20), monotonic(21));
    let target = route.source().target();
    let session = &mut world.interaction.session;
    let confirmation_before = session.intent_confirmation_metrics();
    let admission_before = session.intent_admission_metrics();
    let expiry = issued.pending.expires_at_millis();
    for (time, expected) in [
        (monotonic(expiry), None),
        (
            monotonic(expiry + 1),
            Some(UiIntentConfirmationStopReason::Expired {
                expires_at_millis: expiry,
                observed_millis: expiry + 1,
            }),
        ),
        (
            monotonic(0),
            Some(UiIntentConfirmationStopReason::MonotonicTimeRegressed {
                issued_at_millis: expiry
                    - worth_ui::facade::intent::UI_INTENT_CONFIRMATION_TTL_MILLIS,
                observed_millis: 0,
            }),
        ),
        (
            UiHostObservationTimeBasis::HostWallClockMicros(21),
            Some(UiIntentConfirmationStopReason::MonotonicTimeRequired {
                observed: UiIntentConfirmationTimeBasisKind::HostWallClock,
            }),
        ),
        (
            UiHostObservationTimeBasis::PresentationRelativeTick(21),
            Some(UiIntentConfirmationStopReason::MonotonicTimeRequired {
                observed: UiIntentConfirmationTimeBasisKind::PresentationRelative,
            }),
        ),
    ] {
        let observed = session.observe_intent_confirmation(target, time);
        assert_eq!(observed.stop_reason(), expected.as_ref());
        assert_eq!(observed.is_eligible(), expected.is_none());
        assert_eq!(observed.cost().slots_inspected(), 16);
        assert_eq!(session.intent_confirmation_metrics(), confirmation_before);
        assert_eq!(session.intent_admission_metrics(), admission_before);
        assert_eq!(provider.begin_calls(), 0);
    }
    let ready = match session.continue_intent_confirmation(route) {
        UiIntentConfirmationContinuation::AdmissionReady(ready) => ready,
        UiIntentConfirmationContinuation::Stopped(stop) => panic!(
            "observation must not consume the challenge: {:?}",
            stop.reason()
        ),
    };
    assert_eq!(ready.lineage(), issued.pending.lineage());
    assert_eq!(ready.retained_payload_count(), 1);
    drop(ready);
    let continued = session.intent_confirmation_metrics();
    for _ in 0..2 {
        let observed = session.observe_intent_confirmation(target, monotonic(22));
        assert_eq!(
            observed.stop_reason(),
            Some(&UiIntentConfirmationStopReason::AlreadyContinued)
        );
        assert_eq!(session.intent_confirmation_metrics(), continued);
    }
    let replay = world.confirmation_route(monotonic(23), monotonic(24));
    let UiIntentConfirmationContinuation::Stopped(stop) = world
        .interaction
        .session
        .continue_intent_confirmation(replay)
    else {
        panic!("actual replay must still see the unconsumed terminal marker");
    };
    assert_eq!(
        stop.reason(),
        &UiIntentConfirmationStopReason::AlreadyContinued
    );
    assert_eq!(
        world
            .interaction
            .session
            .intent_confirmation_metrics()
            .replays(),
        1
    );
    assert_eq!(provider.begin_calls(), 0);
}

#[test]
fn confirmation_observation_rejects_drift_and_ambiguity_without_settling_slots() {
    let mut world = ConfirmationWorld::launch();
    let issued = world.issue();
    world.publish_successor();
    let route = world.confirmation_route(monotonic(20), monotonic(21));
    let target = route.source().target();
    assert!(world
        .interaction
        .session
        .observe_intent_confirmation(target, monotonic(21))
        .is_eligible());
    world.set_mutability(false);
    let before = world.interaction.session.intent_confirmation_metrics();
    let observed = world
        .interaction
        .session
        .observe_intent_confirmation(target, monotonic(22));
    assert_eq!(
        observed.stop_reason(),
        Some(&UiIntentConfirmationStopReason::OperabilityDependencyChanged)
    );
    assert_eq!(
        world.interaction.session.intent_confirmation_metrics(),
        before
    );
    let UiIntentConfirmationContinuation::Stopped(stop) = world
        .interaction
        .session
        .continue_intent_confirmation(route)
    else {
        panic!("continuation must reject the same owner drift");
    };
    assert_eq!(stop.reason(), observed.stop_reason().unwrap());
    assert_eq!(
        world
            .interaction
            .session
            .intent_confirmation_metrics()
            .stopped(),
        before.stopped() + 1
    );
    assert_eq!(issued.pending.lineage().diagnostic_value(), 1);

    let mut ambiguous = ConfirmationWorld::launch();
    let first = ambiguous.issue();
    let _second = ambiguous.issue();
    ambiguous.publish_successor();
    let route = ambiguous.confirmation_route(monotonic(20), monotonic(21));
    let before = ambiguous.interaction.session.intent_confirmation_metrics();
    let observed = ambiguous
        .interaction
        .session
        .observe_intent_confirmation(route.source().target(), monotonic(21));
    assert_eq!(
        observed.stop_reason(),
        Some(
            &UiIntentConfirmationStopReason::AmbiguousPendingChallenges {
                declaration: first.pending.declaration_identity().into(),
                observed: 2,
            }
        )
    );
    assert_eq!(
        ambiguous.interaction.session.intent_confirmation_metrics(),
        before
    );
    let UiIntentConfirmationContinuation::Stopped(stop) = ambiguous
        .interaction
        .session
        .continue_intent_confirmation(route)
    else {
        panic!("only continuation may settle ambiguous pending challenges");
    };
    assert_eq!(stop.reason(), observed.stop_reason().unwrap());
    assert_eq!(
        ambiguous
            .interaction
            .session
            .intent_confirmation_metrics()
            .cancelled(),
        2
    );
}

fn monotonic(millis: u64) -> UiHostObservationTimeBasis {
    UiHostObservationTimeBasis::HostMonotonicMillis(millis)
}
