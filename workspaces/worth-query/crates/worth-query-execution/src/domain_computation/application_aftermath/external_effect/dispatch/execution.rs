//! Runtime external-effect dispatch execution over lineage-owned dispatch models.

use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use super::super::WorthQueryAftermathDerivationFailure;
use super::causal_event::with_admitted_dispatch;
use super::observation::classify_dispatch_observation;
use super::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectCausalLadder,
    WorthQueryExternalEffectDispatch, WorthQueryExternalEffectTransport,
};

/// Dispatches one authoritative committed outbox observation through the host port.
pub fn dispatch_external_effect(
    transport: &dyn WorthQueryExternalEffectTransport,
    admitted: crate::domain_computation::primary_graph::WorthQueryAdmittedExternalDispatchAttempt<
        '_,
    >,
    starting: impl FnOnce(),
) -> Result<WorthQueryExternalEffectDispatch, WorthQueryAftermathDerivationFailure> {
    with_admitted_dispatch(admitted, |attempt, clock, progression_work| {
        let correlation = *attempt.record().correlation();
        clock
            .sample(ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds)
            .map_err(|_| WorthQueryAftermathDerivationFailure::RuntimeTimeUnavailable)?;
        let request = WorthQueryExternalDispatchRequest::for_record(attempt.record());
        starting();
        let observed = transport.dispatch(request);
        let classified = classify_dispatch_observation(observed, attempt, clock);
        let causal_ladder = WorthQueryExternalEffectCausalLadder::from_parts(
            attempt.provider_commit().clone(),
            attempt.emission().clone(),
            attempt.attempt().clone(),
            classified.observation.clone(),
        );
        Ok(WorthQueryExternalEffectDispatch::from_parts(
            attempt.runtime_authority(),
            correlation,
            classified.posture,
            causal_ladder,
            progression_work.combine(classified.canonical_work),
        ))
    })
}
