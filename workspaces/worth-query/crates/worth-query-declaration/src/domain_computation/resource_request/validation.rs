use super::{
    WorthQueryExecutionBoundary, WorthQueryExecutionMode, WorthQueryExecutionResourceRequest,
    WorthQueryPartialEffectPosture, WorthQueryResourceDimension, WorthQueryRetainedProgressPosture,
    WorthQuerySemanticScaleAxis, WorthQueryYieldedStatePosture,
};

pub(super) fn validate_resource_request(
    request: &WorthQueryExecutionResourceRequest,
) -> Result<(), &'static str> {
    if request.boundary() == WorthQueryExecutionBoundary::Atomic {
        return validate_atomic_postures(request);
    }
    if WorthQuerySemanticScaleAxis::ALL.iter().any(|axis| {
        *axis != WorthQuerySemanticScaleAxis::WorkItems && request.scale().get(*axis).is_none()
    }) {
        return Err("incomplete-semantic-scale-request");
    }
    if WorthQueryResourceDimension::ALL
        .iter()
        .any(|dimension| request.limits().get(*dimension).is_none())
    {
        return Err("incomplete-resource-limit-request");
    }
    if request.modes().is_empty() {
        return Err("empty-execution-mode-set");
    }
    if request.yielded_state_postures().is_empty() {
        return Err("empty-yielded-state-posture-set");
    }
    if request.retained_progress_postures().is_empty() {
        return Err("empty-retained-progress-posture-set");
    }
    Ok(())
}

fn validate_atomic_postures(
    request: &WorthQueryExecutionResourceRequest,
) -> Result<(), &'static str> {
    if request
        .modes()
        .iter()
        .copied()
        .ne([WorthQueryExecutionMode::Synchronous])
        || !request.degradations().is_empty()
        || request
            .partial_effect_postures()
            .iter()
            .copied()
            .ne([WorthQueryPartialEffectPosture::EffectFree])
        || request
            .yielded_state_postures()
            .iter()
            .copied()
            .ne([WorthQueryYieldedStatePosture::NotYieldable])
        || request
            .retained_progress_postures()
            .iter()
            .copied()
            .ne([WorthQueryRetainedProgressPosture::ReleaseAfterAttempt])
    {
        return Err("invalid-atomic-execution-posture");
    }
    Ok(())
}
