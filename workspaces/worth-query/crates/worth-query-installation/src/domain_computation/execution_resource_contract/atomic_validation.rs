//! Atomic native attempts do not carry managed step algorithms.
use super::WorthQueryExecutionResourceEnvelope;
use worth_query_declaration::facade::domain_computation::{
    WorthQueryExecutionMode, WorthQueryPartialEffectPosture, WorthQueryRetainedProgressPosture,
    WorthQueryYieldedStatePosture,
};
pub(super) fn validate(envelope: &WorthQueryExecutionResourceEnvelope) -> Result<(), &'static str> {
    if envelope.mode() != WorthQueryExecutionMode::Synchronous
        || envelope.degradation().is_some()
        || envelope.partial_effect_posture() != WorthQueryPartialEffectPosture::EffectFree
        || envelope.yielded_state_posture() != WorthQueryYieldedStatePosture::NotYieldable
        || envelope.retained_progress_posture()
            != WorthQueryRetainedProgressPosture::ReleaseAfterAttempt
    {
        return Err("invalid-atomic-execution-posture");
    }
    Ok(())
}
