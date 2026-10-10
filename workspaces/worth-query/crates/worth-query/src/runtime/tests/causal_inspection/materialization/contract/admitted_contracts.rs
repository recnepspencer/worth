use super::*;

#[test]
fn admitted_replay_materialization_rejects_missing_requested_replay_posture() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::super::causal_truth_commit_identity(
                "commit-query-replay-posture-missing",
            ),
            resource_request,
        )
        .unwrap();
    let signal_replay_cursor = "signal-replay-cursor:missing-posture";
    let flow = admitted_replay_flow_requesting_signal_cursor(
        routed.route_identity(),
        signal_replay_cursor,
        CausalInspectionRichness::ReferenceOnly,
    );
    let CausalInspectionProofFlow::Admitted(admitted) = flow else {
        panic!("replay inspection should admit before bridge envelope materialization");
    };
    let envelope = bridge_route_only_envelope_for_admitted_replay(&runtime, &admitted, &routed);

    let error = materialize_admitted_causal_inspection(
        &admitted,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .unwrap_err();

    assert_eq!(
        error.kind(),
        CausalInspectionMaterializationErrorKind::ReplayPostureUnsupported
    );
    assert!(!error.failure_digest().is_empty());
}

#[test]
fn bridge_request_rejects_missing_query_observation_binding_before_materialization() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::super::causal_truth_commit_identity("commit-query-observation-missing"),
            resource_request,
        )
        .unwrap();
    let reference_set = changed_reference_set(routed.route_identity());
    let flow = admit_causal_inspection(request_for(
        reference_set,
        CausalInspectionRichness::ReferenceOnly,
    ));
    let CausalInspectionProofFlow::Admitted(admitted) = flow else {
        panic!("reference-only inspection should admit");
    };
    let bridge_request_denial = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary_for_admitted(&admitted),
        vec![bridge_reference(
            BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
                BridgeCausalEvidenceFamily::BridgeRoute,
                routed.route_identity().bridge_admission_evidence(),
            )
            .expect("route evidence reference identity should be valid"),
        )],
    )
    .expect_err("bridge request should reject a missing query observation anchor");

    assert_eq!(
        bridge_request_denial.kind(),
        BridgeCausalEnvelopeDenialKind::MissingQueryObservationAnchor
    );
}

#[test]
fn bridge_request_rejects_multiple_query_observation_bindings_before_materialization() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::super::causal_truth_commit_identity("commit-query-observation-overclaim"),
            resource_request,
        )
        .unwrap();
    let reference_set = changed_reference_set(routed.route_identity());
    let flow = admit_causal_inspection(request_for(
        reference_set,
        CausalInspectionRichness::ReferenceOnly,
    ));
    let CausalInspectionProofFlow::Admitted(admitted) = flow else {
        panic!("reference-only inspection should admit");
    };
    let bridge_request_denial = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary_for_admitted(&admitted),
        vec![
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    admitted
                        .subject()
                        .query_observation_bridge_evidence_identity(),
                )
                .expect("query observation reference identity should be valid"),
            ),
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(bridge_evidence(
                    "query-observation:unrelated-overclaim",
                ))
                .expect("query observation reference identity should be valid"),
            ),
            bridge_reference(
                BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
                    BridgeCausalEvidenceFamily::BridgeRoute,
                    routed.route_identity().bridge_admission_evidence(),
                )
                .expect("route evidence reference identity should be valid"),
            ),
        ],
    )
    .expect_err("bridge request should reject multiple query observation anchors");

    assert_eq!(
        bridge_request_denial.kind(),
        BridgeCausalEnvelopeDenialKind::QueryObservationAnchorOverclaim
    );
}
