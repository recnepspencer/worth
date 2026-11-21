//! Proven no effect: the external owner determined the attempt changed nothing.

use super::causal_event::DispatchAttemptEvent;

pub use worth_query_execution_lineage::facade::application_aftermath::external_effect::proven_no_effect::{
    ExternalEffectNoEffectCause, ExternalEffectProvenNoEffect,
};

/// Record the external owner's proof against the exact attempt it answered.
pub(super) fn prove_no_effect(
    cause: ExternalEffectNoEffectCause,
    attempt: &DispatchAttemptEvent<'_>,
    clock: &crate::domain_computation::runtime_time::WorthQueryRuntimeClock,
) -> ExternalEffectProvenNoEffect {
    ExternalEffectProvenNoEffect::from_attempt_posture(cause, attempt.attempt().clone(), clock)
}
