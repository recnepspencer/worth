use super::*;
fn safe() -> WorthQueryCancellationSafePointFamily {
    WorthQueryCancellationSafePointFamily::new("attempt").unwrap()
}
fn atomic(
    scale: WorthQuerySemanticScaleRequest,
    limits: WorthQueryResourceLimitRequest,
) -> WorthQueryExecutionResourceRequest {
    WorthQueryExecutionResourceRequest::atomic(scale, limits, safe()).unwrap()
}
#[test]
fn atomic_request_keeps_absence_distinct_from_explicit_zero() {
    let omitted = atomic(
        WorthQuerySemanticScaleRequest::selective(),
        WorthQueryResourceLimitRequest::selective(),
    );
    let zero = atomic(
        WorthQuerySemanticScaleRequest::selective()
            .with(WorthQuerySemanticScaleAxis::CandidateItems, 0),
        WorthQueryResourceLimitRequest::selective()
            .with(WorthQueryResourceDimension::RetainedBytes, 0),
    );
    assert_eq!(omitted.boundary(), WorthQueryExecutionBoundary::Atomic);
    assert_eq!(
        omitted
            .scale()
            .get(WorthQuerySemanticScaleAxis::CandidateItems),
        None
    );
    assert_eq!(
        zero.scale()
            .get(WorthQuerySemanticScaleAxis::CandidateItems),
        Some(0)
    );
    assert_eq!(
        zero.limits()
            .get(WorthQueryResourceDimension::RetainedBytes),
        Some(0)
    );
    assert_ne!(omitted.canonical_identity(), zero.canonical_identity());
    assert!(WorthQueryExecutionResourceRequest::new(
        WorthQuerySemanticScaleRequest::selective(),
        WorthQueryResourceLimitRequest::selective(),
        safe()
    )
    .is_err());
}
#[test]
fn atomic_request_cannot_grant_step_or_partial_effect_postures() {
    let request = atomic(
        WorthQuerySemanticScaleRequest::selective(),
        WorthQueryResourceLimitRequest::selective(),
    );
    for altered in [
        request
            .clone()
            .allow_mode(WorthQueryExecutionMode::Asynchronous),
        request
            .clone()
            .allow_degradation(WorthQueryExecutionDegradation::PartialResult),
        request
            .clone()
            .allow_partial_effect_posture(WorthQueryPartialEffectPosture::PartialEffectsMayRemain),
        request
            .clone()
            .allow_yielded_state_posture(WorthQueryYieldedStatePosture::ProviderCheckpoint),
        request.allow_retained_progress_posture(
            WorthQueryRetainedProgressPosture::RetainAttemptCapacity,
        ),
    ] {
        assert_eq!(altered.validate(), Err("invalid-atomic-execution-posture"));
    }
}

#[test]
#[cfg(target_pointer_width = "64")]
fn dense_request_retains_historical_v1_identity() {
    let request = WorthQueryExecutionResourceRequest::bounded(64, 64, safe());
    assert_eq!(request.boundary(), WorthQueryExecutionBoundary::BoundedStep);
    assert_eq!(
        request.canonical_identity(),
        "1a3b77c7d373e8d25a0ac252957a8e82706151106ec2f0a26840ec9458909ba9"
    );
}
