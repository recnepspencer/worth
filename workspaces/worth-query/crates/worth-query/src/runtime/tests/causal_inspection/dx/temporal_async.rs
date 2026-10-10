use super::*;

#[test]
fn temporal_async_reason_helpers_materialize_bridge_backed_explanations() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-causal-dx-temporal-async"),
            resource_request,
        )
        .unwrap();
    let temporal_artifact =
        CausalInspection::for_test_observation(QueryObservationReceipt::fixture(
            CausalObservationOutcome::Changed,
            vec![
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::QueryInspection,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "query-inspection:dx-temporal-wake",
                    ),
                ),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::BridgeRoute,
                    routed.route_identity().bridge_admission_evidence(),
                ),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::SignalInvalidation,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "signal-invalidation:dx-temporal-wake",
                    ),
                ),
            ],
        ))
        .why_temporal_wake()
        .reference_only()
        .plan()
        .expect("temporal wake helper should plan")
        .materialize_with_bridge(&runtime)
        .expect("temporal wake helper should materialize");
    let async_artifact = CausalInspection::for_test_observation(QueryObservationReceipt::fixture(
        CausalObservationOutcome::Changed,
        vec![
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-inspection:dx-async-completion",
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeRoute,
                routed.route_identity().bridge_admission_evidence(),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::SignalEvaluation,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "signal-evaluation:dx-async-completion",
                ),
            ),
        ],
    ))
    .why_async_completion()
    .reference_only()
    .plan()
    .expect("async completion helper should plan")
    .materialize_with_bridge(&runtime)
    .expect("async completion helper should materialize");

    assert_eq!(
        temporal_artifact.temporal_async_explanation().kind(),
        QueryCausalTemporalAsyncExplanationKind::TemporalWake
    );
    assert_eq!(
        async_artifact.temporal_async_explanation().kind(),
        QueryCausalTemporalAsyncExplanationKind::AsyncCompletion
    );
}
