use super::*;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryResourceLimitRequest,
    WorthQuerySemanticScaleRequest,
};
#[test]
fn live_resource_controls_refuse_atomic_before_missing_queue_access() {
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        std::time::Instant::now() + std::time::Duration::from_secs(30),
        cancellation.token(),
    );
    let controls = WorthQueryApplicationLiveControls::bounded(request, 1, 1, 1).unwrap();
    let envelope = worth_query_installation::facade::WorthQueryExecutionResourceEnvelope::atomic(
        WorthQuerySemanticScaleRequest::selective(),
        WorthQueryResourceLimitRequest::selective(),
        WorthQueryCancellationSafePointFamily::new("attempt").unwrap(),
    );
    let denial = validate_live_envelope_controls(&envelope, &controls, "atomic").unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationLiveOpenDenialKind::BridgeBasisRejected
    );
    assert_eq!(denial.subject(), "atomic-execution-is-not-bounded-step");
}
