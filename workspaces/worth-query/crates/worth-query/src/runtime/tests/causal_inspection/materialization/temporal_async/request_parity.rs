use super::*;

#[test]
fn retained_temporal_evidence_projects_same_explanation_for_all_retained_and_explicit_requests() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::super::causal_truth_commit_identity(
                "commit-query-temporal-request-parity",
            ),
            resource_request,
        )
        .expect("temporal parity route should resolve");
    let receipt = QueryObservationReceipt::fixture(
        CausalObservationOutcome::Changed,
        vec![
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-inspection:temporal-request-parity",
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeRoute,
                routed.route_identity().bridge_admission_evidence(),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::SignalInvalidation,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "signal-invalidation:temporal-request-parity",
                ),
            ),
        ],
    );
    let all_retained_artifact = CausalInspection::for_test_observation(receipt.clone())
        .why_changed()
        .include_all_retained_evidence()
        .plan()
        .expect("all-retained temporal request should plan")
        .materialize_with_bridge(&runtime)
        .expect("all-retained temporal request should materialize");
    let explicit_artifact = CausalInspection::for_test_observation(receipt)
        .why_temporal_wake()
        .reference_only()
        .plan()
        .expect("explicit temporal request should plan")
        .materialize_with_bridge(&runtime)
        .expect("explicit temporal request should materialize");

    assert_eq!(
        all_retained_artifact.temporal_async_explanation().kind(),
        QueryCausalTemporalAsyncExplanationKind::TemporalWake
    );
    assert_eq!(
        all_retained_artifact.temporal_async_explanation().kind(),
        explicit_artifact.temporal_async_explanation().kind()
    );
}
