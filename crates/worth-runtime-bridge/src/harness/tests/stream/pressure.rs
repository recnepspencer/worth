use crate::facade::BridgeRuntimePolicy;
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};

use super::super::support::{build_runtime, committed_patch, registration, snapshot};

#[test]
fn bridge_stream_checkpoint_fracture_equivalence_fails_explicitly_for_stale_anchor() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let runtime = crate::facade::RuntimeBridge::builder()
        .with_relational_source(source.clone())
        .with_truth_branch_head_source(source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_policy(
            BridgeRuntimePolicy::development()
                .with_route_record_limit(1)
                .with_failure_record_limit(1),
        )
        .register_mapping(registration())
        .build()
        .expect("stream runtime should build");
    let declaration = crate::stream::ChangeStreamDeclaration::new(
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
    let protocol = runtime
        .validate_change_stream_declaration(declaration)
        .expect("declaration should validate");
    let contract = runtime
        .resolve_change_stream_consumer_contract(&protocol)
        .expect("contract should resolve");
    let first_window = runtime
        .plan_change_stream_window(
            &contract,
            vec![runtime
                .ingest_committed_patch(
                    crate::facade::BridgeRouteRequest::for_commit(
                        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                    ),
                    execution,
                )
                .expect("first envelope should ingest")],
        )
        .expect("first window should plan");
    let first_checkpoint = runtime.publish_consumer_checkpoint(
        &contract,
        &first_window,
        crate::stream::StreamCheckpointFrontierKind::ContiguousFrontier,
    );
    let second_window = runtime
        .plan_change_stream_window(
            &contract,
            vec![runtime
                .ingest_committed_patch(
                    crate::facade::BridgeRouteRequest::for_commit(
                        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                    ),
                    execution,
                )
                .expect("second envelope should ingest")],
        )
        .expect("second window should plan");
    let _second_checkpoint = runtime.publish_consumer_checkpoint(
        &contract,
        &second_window,
        crate::stream::StreamCheckpointFrontierKind::ContiguousFrontier,
    );

    let error = runtime
        .resume_stream_window_from_checkpoint(
            &contract,
            vec![runtime
                .ingest_committed_patch(
                    crate::facade::BridgeRouteRequest::for_commit(
                        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                    ),
                    execution,
                )
                .expect("second envelope should ingest")],
            first_checkpoint.checkpoint_token_identity(),
        )
        .expect_err("stale anchors must fail explicitly");

    assert_eq!(
        error.kind(),
        crate::error::BridgeStreamErrorKind::CheckpointTruncated
    );
}

#[test]
fn bridge_stream_backpressure_changes_pacing_class_without_changing_member_truth() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
    );
    let declaration = crate::stream::ChangeStreamDeclaration::new(
        crate::stream::StreamConsumerShape::ReplayAuditConsumer,
        crate::stream::StreamResumeMode::FromCheckpointOnly,
        crate::stream::StreamCheckpointPublicationMode::PublishEveryWindow,
        crate::stream::StreamCoalescingIntent::Prefer(
            crate::stream::StreamCoalescingFamily::ReplayAuditWindowCoalescing,
        ),
        crate::stream::StreamReplayMode::Enabled,
        crate::stream::StreamDeliveryIntent::ReplayAudit,
        crate::stream::StreamDiagnosticsPolicyClass::Standard,
    );
    let protocol = runtime
        .validate_change_stream_declaration(declaration)
        .expect("declaration should validate");
    let contract = runtime
        .resolve_change_stream_consumer_contract(&protocol)
        .expect("contract should resolve");
    let single_window = runtime
        .plan_change_stream_window(
            &contract,
            vec![runtime
                .ingest_committed_patch(
                    crate::facade::BridgeRouteRequest::for_commit(
                        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                    ),
                    execution,
                )
                .expect("first envelope should ingest")],
        )
        .expect("single window should plan");
    let burst_window = runtime
        .plan_change_stream_window(
            &contract,
            vec![
                runtime
                    .ingest_committed_patch(
                        crate::facade::BridgeRouteRequest::for_commit(
                            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                        ),
                        execution,
                    )
                    .expect("first envelope should ingest"),
                runtime
                    .ingest_committed_patch(
                        crate::facade::BridgeRouteRequest::for_commit(
                            crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                        ),
                        execution,
                    )
                    .expect("second envelope should ingest"),
            ],
        )
        .expect("burst window should plan");

    let single_pressure = runtime.classify_stream_backpressure(&single_window);
    let burst_pressure = runtime.classify_stream_backpressure(&burst_window);

    assert_eq!(single_pressure.pressure_class(), "no-pressure");
    assert_eq!(burst_pressure.pressure_class(), "elevated-pressure");
    assert_ne!(
        single_pressure.backpressure_decision_identity(),
        burst_pressure.backpressure_decision_identity()
    );
    assert_ne!(
        single_window.member_set_digest(),
        burst_window.member_set_digest()
    );
}

#[test]
fn backpressure_changes_pacing_without_semantic_drift() {
    bridge_stream_backpressure_changes_pacing_class_without_changing_member_truth();
}
