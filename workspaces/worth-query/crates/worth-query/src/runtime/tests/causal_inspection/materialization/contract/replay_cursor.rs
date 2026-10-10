use super::*;

#[test]
fn admitted_replay_materialization_accepts_signal_owned_replay_cursor_posture() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::super::causal_truth_commit_identity("commit-query-replay-posture-bound"),
            resource_request,
        )
        .unwrap();
    let signal_replay_cursor = "signal-replay-cursor:bound-posture";
    let flow = admitted_replay_flow_requesting_signal_cursor(
        routed.route_identity(),
        signal_replay_cursor,
        CausalInspectionRichness::ReferenceOnly,
    );
    let CausalInspectionProofFlow::Admitted(admitted) = flow else {
        panic!("replay inspection should admit before bridge envelope materialization");
    };
    let envelope = signal_replay_cursor_envelope_for_admitted_replay(
        &runtime,
        &admitted,
        &routed,
        signal_replay_cursor,
    );

    let artifact = materialize_admitted_causal_inspection(
        &admitted,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .expect("signal replay cursor should satisfy replay posture materialization");
    let QueryCausalInspectionArtifact::Admitted(artifact) = artifact else {
        panic!("expected admitted query causal artifact");
    };

    assert!(artifact.evidence_references().iter().any(|reference| {
        reference.owner() == BridgeCausalEvidenceOwner::Signal.as_str()
            && reference.family() == BridgeCausalEvidenceFamily::SignalReplayCursor.as_str()
            && reference.reference_evidence_for_reporting()
                == crate::runtime::tests::causal_test_bridge_binding_reference_for_reporting(
                    BridgeCausalEvidenceOwner::Signal.as_str(),
                    BridgeCausalEvidenceFamily::SignalReplayCursor.as_str(),
                    bridge_evidence(signal_replay_cursor),
                )
    }));
    assert_eq!(artifact.performance().bridge_unindexed_scan_count(), 0);
}
