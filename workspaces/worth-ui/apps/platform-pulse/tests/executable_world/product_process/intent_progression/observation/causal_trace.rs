use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseIntentAttemptObservationReference, PlatformPulseIntentCausalTraceObservation,
    PlatformPulseIntentInteractionFamily, PlatformPulseLifecycleObservation,
    PlatformPulseMountedFrameObservation, PlatformPulseQueryProjectionEvidence,
};

use super::{next, IntentObservationFailure, Sequenced};
use crate::product_process::{NativeBoundExecutableWorld, WatchedPulseTransition};

pub(super) fn await_completed_causal_trace(
    world: &mut NativeBoundExecutableWorld,
    expected_attempt: PlatformPulseIntentAttemptObservationReference,
    expected_query: &PlatformPulseQueryProjectionEvidence,
    expected_frame: PlatformPulseMountedFrameObservation,
) -> Result<Sequenced<PlatformPulseIntentCausalTraceObservation>, IntentObservationFailure> {
    let envelope = next(world, WatchedPulseTransition::IntentCausalTrace)?;
    let PlatformPulseLifecycleObservation::IntentCausalTrace(trace) = envelope.outcome() else {
        return Err(IntentObservationFailure::QueryCompletion(format!(
            "expected intent causal trace, observed {:?}",
            envelope.outcome()
        )));
    };
    validate_trace(trace, expected_attempt, expected_query, expected_frame)?;
    Ok(Sequenced {
        value: trace.clone(),
        sequence: envelope.sequence().value(),
    })
}

fn validate_trace(
    trace: &PlatformPulseIntentCausalTraceObservation,
    expected_attempt: PlatformPulseIntentAttemptObservationReference,
    expected_query: &PlatformPulseQueryProjectionEvidence,
    expected_frame: PlatformPulseMountedFrameObservation,
) -> Result<(), IntentObservationFailure> {
    let evidence = trace.evidence_reference();
    let source = trace.source();
    let route = trace.route();
    let payload = trace.payload();
    let operability = trace.operability();
    let admission = trace.admission();
    let outcome = trace.outcome();
    let complete = evidence.session() != 0
        && evidence.generation() != 0
        && source.host_sequence() != 0
        && source.presented_frame() != 0
        && source.presentation_epoch() != 0
        && source.mounted_instance() != 0
        && source.semantic_target_digest() != 0
        && source.interaction_family() == PlatformPulseIntentInteractionFamily::Activate
        && route.graph_node() != 0
        && route.definition_digest() != 0
        && route.declaration_digest() != 0
        && payload.owner_revision_digest() != 0
        && operability.operable()
        && operability.dependency_count() != 0
        && operability.decision_digest() != 0
        && admission.slot() == expected_attempt.attempt_slot()
        && admission.generation() == expected_attempt.attempt_generation()
        && admission.lineage() != 0
        && trace.attempt() == expected_attempt
        && outcome.outcome_schema_digest() != 0
        && outcome.consequence_pending_at_completion()
        && outcome.consequence_published()
        && trace.query_projection() == expected_query
        && trace.mounted_frame() == expected_frame;
    if !complete {
        return Err(IntentObservationFailure::QueryCompletion(format!(
            "intent causal trace was incomplete or mismatched: {trace:?}"
        )));
    }
    Ok(())
}
