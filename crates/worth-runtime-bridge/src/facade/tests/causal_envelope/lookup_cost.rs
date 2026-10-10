use super::*;

#[test]
fn causal_envelope_lookup_cost_ignores_unrelated_retained_routes() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    for unrelated_routes in [0, 4, 12] {
        let runtime = runtime(BridgeRuntimePolicy::default());
        for index in 0..unrelated_routes {
            runtime
                .route(
                    crate::truth_identity_fixtures::truth_commit_fixture(format!(
                        "unrelated-causal-{index}"
                    )),
                    execution,
                )
                .expect("unrelated route should succeed");
        }
        let routed = runtime
            .route(
                crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-target"),
                execution,
            )
            .expect("target route should succeed");
        let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
            crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "query-admission:scale",
                ),
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "causal-anchor:scale",
                ),
            )
            .expect("query admission summary should be valid"),
            vec![
                query_observation_reference(
                    BridgeCausalEvidenceReferenceIdentity::query_observation(
                        crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                            "query-observation:scale",
                        ),
                    )
                    .expect("query observation reference identity should be valid"),
                ),
                bridge_route_reference(routed.result().result_summary()),
            ],
        )
        .expect("request should be valid");

        let envelope = runtime
            .diagnostics()
            .assemble_causal_explanation_envelope(request)
            .expect("target route should bind");

        assert_eq!(
            runtime.diagnostics().route_records().len(),
            unrelated_routes + 1
        );
        assert_eq!(envelope.counters().bridge_retained_lookup_count(), 1);
        assert_eq!(envelope.counters().retained_bridge_binding_count(), 1);
        assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);
    }
}
