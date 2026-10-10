use super::*;

#[test]
fn advisory_materialization_rejects_mismatched_query_observation_binding() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::super::causal_truth_commit_identity("commit-query-observation-mismatch"),
            resource_request,
        )
        .unwrap();
    let reference_set = changed_reference_set(routed.route_identity());
    let flow = admit_causal_inspection(request_for(
        reference_set,
        CausalInspectionRichness::MaterializedDetail,
    ));
    let CausalInspectionProofFlow::Advisory(advisory) = flow else {
        panic!("materialized detail should narrow to advisory");
    };
    let summary = BridgeCausalInspectionAdmissionSummary::advisory(
        bridge_query_evidence(
            "causal-inspection-outcome",
            advisory.advisory_inspection_digest(),
        ),
        bridge_query_evidence(
            "causal-observation-anchor",
            advisory.subject().anchor_for_reporting(),
        ),
    )
    .expect("query advisory summary should be valid");
    let bridge_request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary,
        vec![
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(bridge_evidence(
                    "query-observation:wrong-inspection",
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
    .expect("bridge request should be valid");
    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect("bridge envelope should assemble");

    let error = materialize_advisory_causal_inspection(
        &advisory,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .unwrap_err();

    assert_eq!(
        error.kind(),
        CausalInspectionMaterializationErrorKind::QueryObservationBindingMismatch
    );
}

#[test]
fn advisory_replay_materialization_rejects_missing_requested_replay_posture() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::super::causal_truth_commit_identity(
                "commit-query-advisory-replay-posture-missing",
            ),
            resource_request,
        )
        .unwrap();
    let signal_replay_cursor = "signal-replay-cursor:advisory-missing-posture";
    let flow = admitted_replay_flow_requesting_signal_cursor(
        routed.route_identity(),
        signal_replay_cursor,
        CausalInspectionRichness::MaterializedDetail,
    );
    let CausalInspectionProofFlow::Advisory(advisory) = flow else {
        panic!("materialized replay inspection should narrow to advisory");
    };
    let envelope = bridge_route_only_envelope_for_advisory_replay(&runtime, &advisory, &routed);

    let error = materialize_advisory_causal_inspection(
        &advisory,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .unwrap_err();

    assert_eq!(
        error.kind(),
        CausalInspectionMaterializationErrorKind::ReplayPostureUnsupported
    );
}
