use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseIntentAttemptObservationReference, PlatformPulseIntentPostureObservation,
    PlatformPulseLifecycleObservation,
};

use crate::product_process::{NativeBoundExecutableWorld, WatchedPulseTransition};

use super::super::{observation, PlatformPulseIntentJourneyFailure};

pub(super) fn await_rebind_cancellation(
    world: &mut NativeBoundExecutableWorld,
    expected_attempt: PlatformPulseIntentAttemptObservationReference,
) -> Result<u64, PlatformPulseIntentJourneyFailure> {
    let rebind = observation::next(world, WatchedPulseTransition::IntentCancellationRebind)?;
    let PlatformPulseLifecycleObservation::RebindPublished(replacement) = rebind.outcome() else {
        return Err(PlatformPulseIntentJourneyFailure::Cancellation(
            "route-removal source edit did not publish an application replacement".to_owned(),
        ));
    };
    if replacement.predecessor_generation() == replacement.active_generation() {
        return Err(PlatformPulseIntentJourneyFailure::Cancellation(
            "route removal did not change application generation".to_owned(),
        ));
    }

    let cancellation =
        observation::next_unfiltered(world, WatchedPulseTransition::IntentCancellationRebind)?;
    if !matches!(
        cancellation.outcome(),
        PlatformPulseLifecycleObservation::IntentPosturePublished(posture)
            if matches!(
                posture.posture(),
                PlatformPulseIntentPostureObservation::Cancelled { reference }
                    if *reference == expected_attempt
            )
    ) {
        return Err(PlatformPulseIntentJourneyFailure::Cancellation(format!(
            "expected cancellation of attempt {expected_attempt:?}; observed {:?}",
            cancellation.outcome()
        )));
    }
    Ok(cancellation.sequence().value())
}
