use std::time::Duration;

use crate::installation::CanonicalPlatformPulse;
use crate::source_delta::IntentRouteRemovalSourceDelta;

use super::platform_pulse_cleanup::close_recovered_at_sequence;
use super::platform_pulse_journey::{complete_open, PlatformPulseJourneyDeltas};

const JOURNEY_CEILING: Duration = Duration::from_secs(45);
// The designed 960-by-600 Pulse still stays below the 45-second RS-01 journey
// ceiling, while this focused launch/action/capture/close proof keeps ten
// seconds of independent headroom.
const CAUSAL_PULSE_CEILING: Duration = Duration::from_secs(35);

#[test]
fn intent_causal_trace_reaches_pixels_without_becoming_authority() {
    let inherited = complete_open(
        PlatformPulseJourneyDeltas::exact().expect("derive the exact inherited source deltas"),
    );
    let completed = inherited
        .into_recovered()
        .complete_intent_causal_pulse()
        .unwrap_or_else(|failure| {
            panic!("real intent causal trace reaches an independent visible pixel: {failure}")
        });
    let trace = completed.evidence().trace();
    assert!(completed.evidence().changed_control_pixels() >= 9);
    assert!(trace.outcome().consequence_pending_at_completion());
    assert!(trace.outcome().consequence_published());
    assert_eq!(
        trace.admission().slot(),
        trace.attempt().attempt_slot(),
        "admission and attempt remain one exact generational lane"
    );
    assert_eq!(trace.query_projection().native_value(), Some("Deployed"));
    let reporting_copy = serde_json::to_vec(trace).expect("trace is disposable reporting data");
    let decoded: worth_ui_platform_pulse::observation_contract::
        PlatformPulseIntentCausalTraceObservation =
        serde_json::from_slice(&reporting_copy).expect("reporting projection round-trips");
    assert_eq!(&decoded, trace);

    let journey_started = completed.native_journey_started();
    let expected_shutdown_sequence = completed.evidence().expected_shutdown_sequence();
    let closed =
        close_recovered_at_sequence(completed.into_recovered(), expected_shutdown_sequence);
    assert!(closed.evidence().successful_exit().status().success());
    assert!(
        journey_started.elapsed() <= CAUSAL_PULSE_CEILING,
        "IA-12 exceeded its 35-second single-completion product-world budget"
    );
}

#[test]
fn canonical_platform_pulse_intent_reaches_visible_query_backed_consequence() {
    let canonical = CanonicalPlatformPulse::checked_in();
    let route_removal = IntentRouteRemovalSourceDelta::from_checked_in(canonical)
        .expect("canonical Pulse contains exactly one typed action route");
    let inherited = complete_open(
        PlatformPulseJourneyDeltas::exact().expect("derive the exact inherited source deltas"),
    );

    let completed = inherited
        .into_recovered()
        .complete_intent_journey(route_removal)
        .unwrap_or_else(|failure| {
            panic!("real native intent reaches a visible Query-backed consequence: {failure}")
        });
    let journey_started = completed.native_journey_started();
    let expected_shutdown_sequence = completed.evidence().expected_shutdown_sequence();
    assert_eq!(completed.evidence().native_activation_count(), 8);
    assert_eq!(completed.evidence().source_action_count(), 7);
    assert_eq!(completed.evidence().provider_start_count(), 3);
    assert_eq!(completed.evidence().completion_count(), 2);
    assert_eq!(completed.evidence().query_action_count(), 2);
    assert_eq!(completed.evidence().visible_posture_count(), 10);
    assert!(completed.evidence().minimum_changed_control_pixels() >= 9);
    assert_eq!(completed.evidence().causal_pixel_count(), 2);
    assert!(completed.evidence().minimum_causal_changed_control_pixels() >= 9);
    assert!(completed
        .evidence()
        .first_causal_trace()
        .outcome()
        .consequence_published());
    assert!(completed.evidence().attempts_are_distinct());

    let closed =
        close_recovered_at_sequence(completed.into_recovered(), expected_shutdown_sequence);
    assert!(closed.evidence().successful_exit().status().success());
    assert!(
        journey_started.elapsed() <= JOURNEY_CEILING,
        "IA-01 exceeded its 45-second cumulative product-world budget"
    );
}
