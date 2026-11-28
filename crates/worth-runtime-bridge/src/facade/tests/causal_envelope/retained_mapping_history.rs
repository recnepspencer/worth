use super::retained_mapping_support::{
    binding_for, bridge_historical_evaluation_failure_reference, bridge_route_reference,
    query_observation_reference,
};
use super::{
    registered_source, runtime_with_source_adapter, BridgeRuntimePolicy, BridgeSourceCapability,
    BridgeTruthViewSelector, RejectingSourceAdapter, SnapshotReadPacket,
};
use crate::facade::{BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEvidenceFamily};

#[test]
fn causal_envelope_maps_historical_failure_by_exact_identity() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime =
        runtime_with_source_adapter(BridgeRuntimePolicy::default(), RejectingSourceAdapter);
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-history-failure"),
            execution,
        )
        .expect("route should succeed");
    let contract = runtime
        .admit_source(registered_source(
            "source:analysis-history",
            BridgeTruthViewSelector::historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ],
        ))
        .expect("source should admit");
    assert!(
        runtime
            .materialize_source_packet(&contract, SnapshotReadPacket::new(vec![]), execution)
            .is_err(),
        "rejecting source adapter should record historical failure"
    );
    let historical_failure = runtime
        .diagnostics()
        .last_historical_evaluation_failure()
        .expect("historical failure should be retained");
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:history-failure",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:history-failure",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                crate::facade::BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:history-failure",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            bridge_historical_evaluation_failure_reference(&historical_failure),
        ],
    )
    .expect("request should be valid");

    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect("historical failure should bind");

    assert_eq!(envelope.counters().bridge_retained_lookup_count(), 2);
    assert_eq!(envelope.counters().retained_bridge_binding_count(), 2);
    assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeHistoricalEvaluationFailure,
            historical_failure.failure_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(historical_failure_digest(&historical_failure).as_str())
    );
}

fn historical_failure_digest(
    record: &crate::facade::BridgeHistoricalEvaluationFailureRecord,
) -> String {
    crate::diagnostics::causal_envelope::retained_mapping::retained_artifact_digest::planning_checkpoint::historical_evaluation_failure_digest(record)
        .expect("bounded digest fixture").as_str()
        .to_string()
}
