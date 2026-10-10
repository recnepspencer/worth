use super::retained_mapping_support::{
    binding_for, bridge_historical_evaluation_failure_reference, bridge_route_reference,
    bridge_stream_checkpoint_reference, missing_bridge_reference, query_observation_reference,
};
use super::{
    canonical_envelope, registered_source, runtime, runtime_with_source_adapter,
    BridgeRuntimePolicy, BridgeSourceCapability, BridgeTruthViewSelector, RejectingSourceAdapter,
    SnapshotReadPacket,
};
use crate::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEnvelopeDenialKind, BridgeCausalEvidenceFamily,
};

#[test]
fn causal_envelope_maps_historical_failure_and_stream_checkpoint_by_exact_identity() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime =
        runtime_with_source_adapter(BridgeRuntimePolicy::default(), RejectingSourceAdapter);
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-history-stream"),
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
    let stream_checkpoint = retain_stream_checkpoint(&runtime, "target");
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:history-stream",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:history-stream",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                crate::facade::BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:history-stream",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            bridge_historical_evaluation_failure_reference(&historical_failure),
            bridge_stream_checkpoint_reference(&stream_checkpoint),
        ],
    )
    .expect("request should be valid");

    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect("historical failure and stream checkpoint should bind");

    assert_eq!(envelope.counters().bridge_retained_lookup_count(), 3);
    assert_eq!(envelope.counters().retained_bridge_binding_count(), 3);
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
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeStreamCheckpoint,
            stream_checkpoint.checkpoint_token_identity_for_reporting()
        )
        .retained_record_digest_for_reporting(),
        Some(stream_checkpoint_digest(&stream_checkpoint).as_str())
    );
}

#[test]
fn causal_envelope_denies_missing_stream_checkpoint_without_unindexed_scan() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-causal-missing-stream-checkpoint",
            ),
            execution,
        )
        .expect("route should succeed");
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:missing-stream-checkpoint",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:missing-stream-checkpoint",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                crate::facade::BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:missing-stream-checkpoint",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            missing_bridge_reference(
                BridgeCausalEvidenceFamily::BridgeStreamCheckpoint,
                "missing-stream-checkpoint",
            ),
        ],
    )
    .expect("request should be valid");

    let denial = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect_err("missing stream checkpoint should deny");

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::MissingRetainedBridgeRecord
    );
    assert_eq!(
        denial.family(),
        BridgeCausalEvidenceFamily::BridgeStreamCheckpoint
    );
    assert_eq!(denial.counters().bridge_retained_lookup_count(), 2);
    assert_eq!(denial.counters().retained_bridge_binding_count(), 1);
    assert_eq!(denial.counters().missing_bridge_record_count(), 1);
    assert_eq!(denial.counters().bridge_record_unindexed_scan_count(), 0);
}

#[test]
fn causal_envelope_stream_checkpoint_lookup_cost_ignores_unrelated_records() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut envelope_identities = Vec::new();

    for unrelated_records in [0, 3, 8] {
        let runtime = runtime(BridgeRuntimePolicy::default());
        for index in 0..unrelated_records {
            retain_stream_checkpoint(&runtime, &format!("noise-{index}"));
        }
        let routed = runtime
            .route(
                crate::truth_identity_fixtures::truth_commit_fixture(
                    "commit-causal-stream-checkpoint-scale",
                ),
                execution,
            )
            .expect("route should succeed");
        let target_checkpoint = retain_stream_checkpoint(&runtime, "target");
        let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
            crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "query-admission:stream-checkpoint-scale",
                ),
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "causal-anchor:stream-checkpoint-scale",
                ),
            )
            .expect("query admission summary should be valid"),
            vec![
                query_observation_reference(
                    crate::facade::BridgeCausalEvidenceReferenceIdentity::query_observation(
                        crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                            "query-observation:stream-checkpoint-scale",
                        ),
                    )
                    .expect("query observation reference identity should be valid"),
                ),
                bridge_route_reference(routed.result().result_summary()),
                bridge_stream_checkpoint_reference(&target_checkpoint),
            ],
        )
        .expect("request should be valid");

        let envelope = runtime
            .diagnostics()
            .assemble_causal_explanation_envelope(request)
            .expect("target stream checkpoint should bind");

        assert_eq!(
            runtime.diagnostics().stream_checkpoints().len(),
            unrelated_records + 1
        );
        assert_eq!(envelope.counters().bridge_retained_lookup_count(), 2);
        assert_eq!(envelope.counters().retained_bridge_binding_count(), 2);
        assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);
        envelope_identities.push(
            envelope
                .identity()
                .envelope_identity_for_reporting()
                .to_string(),
        );
    }

    assert_eq!(envelope_identities[0], envelope_identities[1]);
    assert_eq!(envelope_identities[1], envelope_identities[2]);
}

fn retain_stream_checkpoint(
    runtime: &crate::facade::RuntimeBridge,
    suffix: &str,
) -> crate::stream::ConsumerCheckpointToken {
    let stream_declaration = crate::stream::ChangeStreamDeclaration::new(
        crate::stream::StreamConsumerShape::RoutingConsumer,
        crate::stream::StreamResumeMode::FromCheckpointOnly,
        crate::stream::StreamCheckpointPublicationMode::PublishEveryWindow,
        crate::stream::StreamCoalescingIntent::Prefer(
            crate::stream::StreamCoalescingFamily::RoutingWindowCoalescing,
        ),
        crate::stream::StreamReplayMode::Enabled,
        crate::stream::StreamDeliveryIntent::RouteInvalidations,
        crate::stream::StreamDiagnosticsPolicyClass::Standard,
    );
    let stream_protocol = runtime
        .validate_change_stream_declaration(stream_declaration)
        .expect("stream declaration should validate");
    let stream_contract = runtime
        .resolve_change_stream_consumer_contract(&stream_protocol)
        .expect("stream contract should resolve");
    let stream_window = runtime
        .plan_change_stream_window(
            &stream_contract,
            vec![canonical_envelope(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::truth_identity_fixtures::truth_commit_fixture(format!("commit-{suffix}")),
                crate::truth_identity_fixtures::truth_patch_fixture(format!("patch-{suffix}")),
                crate::truth_identity_fixtures::truth_snapshot_fixture(format!(
                    "snapshot-{suffix}"
                )),
            )],
        )
        .expect("stream window should plan");
    runtime.publish_consumer_checkpoint(
        &stream_contract,
        &stream_window,
        crate::stream::StreamCheckpointFrontierKind::ContiguousFrontier,
    )
}

fn historical_failure_digest(
    record: &crate::facade::BridgeHistoricalEvaluationFailureRecord,
) -> String {
    crate::diagnostics::causal_envelope::retained_mapping::retained_artifact_digest::planning_checkpoint::historical_evaluation_failure_digest(record)
        .as_str()
        .to_string()
}

fn stream_checkpoint_digest(record: &crate::stream::ConsumerCheckpointToken) -> String {
    crate::diagnostics::causal_envelope::retained_mapping::retained_artifact_digest::planning_checkpoint::stream_checkpoint_digest(record)
        .as_str()
        .to_string()
}
