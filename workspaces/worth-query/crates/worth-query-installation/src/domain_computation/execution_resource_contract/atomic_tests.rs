use super::*;
use worth_query_declaration::facade::domain_computation::*;
fn envelope() -> WorthQueryExecutionResourceEnvelope {
    WorthQueryExecutionResourceEnvelope::atomic(
        WorthQuerySemanticScaleRequest::selective(),
        WorthQueryResourceLimitRequest::selective(),
        WorthQueryCancellationSafePointFamily::new("attempt").unwrap(),
    )
}
fn declared(
    envelope: WorthQueryExecutionResourceEnvelope,
) -> Result<WorthQueryExecutionResourceContract, &'static str> {
    WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
        WorthQueryExecutionStrategyName::new("atomic").unwrap(),
        envelope,
        WorthQueryExecutionProviderRequirements::new(
            WorthQueryExecutionProviderFamily::new("provider").unwrap(),
            WorthQueryExecutionAccessProductFamily::new("access").unwrap(),
            WorthQueryExecutionAllocatorFamily::new("arena").unwrap(),
        ),
    )])
}
#[test]
fn atomic_envelope_is_not_a_dense_bounded_step_contract() {
    let envelope = envelope();
    assert!(declared(envelope.clone()).is_ok());
    assert_eq!(
        envelope.bounded_step_contract(),
        Err("atomic-execution-is-not-bounded-step")
    );
    let bounded = WorthQueryExecutionResourceEnvelope::new(
        envelope.scale_ceilings().clone(),
        envelope.resource_ceilings().clone(),
        WorthQueryExecutionMode::Synchronous,
        None,
        envelope.cancellation_safe_point().clone(),
    );
    assert_eq!(
        declared(bounded).unwrap_err(),
        "incomplete-bounded-step-envelope"
    );
    for altered in [
        envelope
            .clone()
            .with_partial_effect_posture(WorthQueryPartialEffectPosture::PartialEffectsMayRemain),
        envelope
            .clone()
            .with_yielded_state_posture(WorthQueryYieldedStatePosture::ProviderCheckpoint),
        envelope.with_retained_progress_posture(
            WorthQueryRetainedProgressPosture::RetainAttemptCapacity,
        ),
    ] {
        assert_eq!(
            declared(altered).unwrap_err(),
            "invalid-atomic-execution-posture"
        );
    }
}
