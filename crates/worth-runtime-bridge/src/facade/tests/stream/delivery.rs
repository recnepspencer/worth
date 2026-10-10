use crate::facade::tests::{canonical_envelope, runtime};
use crate::policy::BridgeRuntimePolicy;

#[test]
fn runtime_delivers_routing_stream_window_through_admitted_contract() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let declaration = crate::stream::ChangeStreamDeclaration::new(
        crate::stream::StreamConsumerShape::RoutingConsumer,
        crate::stream::StreamResumeMode::FromCheckpointOnly,
        crate::stream::StreamCheckpointPublicationMode::PublishEveryWindow,
        crate::stream::StreamCoalescingIntent::None,
        crate::stream::StreamReplayMode::Enabled,
        crate::stream::StreamDeliveryIntent::RouteInvalidations,
        crate::stream::StreamDiagnosticsPolicyClass::Standard,
    );
    let protocol = runtime
        .validate_change_stream_declaration(declaration)
        .expect("stream declarations should validate");
    let contract = runtime
        .resolve_change_stream_consumer_contract(&protocol)
        .expect("stream contract should resolve");
    let window = runtime
        .plan_change_stream_window(
            &contract,
            vec![
                canonical_envelope(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                    crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                ),
                canonical_envelope(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                    crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                ),
            ],
        )
        .expect("window should plan");

    let result = runtime
        .deliver_change_stream_window(&contract, &window, execution)
        .expect("routing-consumer windows should deliver");

    assert!(window.lowered_change_set().is_some());
    assert_eq!(
        window
            .lowered_change_set()
            .and_then(|lowered| lowered.planned_routes())
            .map(|routes| routes.len()),
        Some(2)
    );
    assert_eq!(
        result.summary().stream_window_identity(),
        window.stream_window_identity()
    );
    assert_eq!(result.summary().delivered_member_count(), 2);
    assert_eq!(result.summary().delivered_route_count(), 2);
    assert_eq!(result.route_results().len(), 2);
}

#[test]
fn runtime_rejects_delivery_for_non_routing_consumer_shape() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
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
        .expect("stream declarations should validate");
    let contract = runtime
        .resolve_change_stream_consumer_contract(&protocol)
        .expect("stream contract should resolve");
    let window = runtime
        .plan_change_stream_window(
            &contract,
            vec![canonical_envelope(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            )],
        )
        .expect("window should plan");

    let error = runtime
        .deliver_change_stream_window(&contract, &window, execution)
        .expect_err("non-routing consumer delivery should be rejected explicitly");

    assert_eq!(
        error.kind(),
        crate::error::BridgeStreamErrorKind::UnsupportedConsumerShape
    );
}

#[test]
fn runtime_delivers_replay_audit_stream_window_and_retains_protocol_truth() {
    let runtime = runtime(BridgeRuntimePolicy::development());
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
        .expect("stream declarations should validate");
    let contract = runtime
        .resolve_change_stream_consumer_contract(&protocol)
        .expect("stream contract should resolve");
    let window = runtime
        .plan_change_stream_window(
            &contract,
            vec![
                canonical_envelope(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                    crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                ),
                canonical_envelope(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                    crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                ),
            ],
        )
        .expect("window should plan");

    let result = runtime
        .deliver_replay_audit_stream_window(&contract, &window)
        .expect("replay-audit windows should deliver");

    assert!(window.lowered_change_set().is_some());
    assert_eq!(
        result.summary().stream_window_identity(),
        window.stream_window_identity()
    );
    assert_eq!(result.summary().audited_member_count(), 2);
    assert_eq!(
        runtime
            .diagnostics()
            .stream_checkpoint_for_identity(result.checkpoint().checkpoint_token_identity())
            .expect("audit checkpoint should be retained")
            .checkpoint_token_identity(),
        result.checkpoint().checkpoint_token_identity()
    );
    assert_eq!(
        runtime
            .diagnostics()
            .stream_replay_record_for_identity(result.replay_record().replay_record_identity())
            .expect("audit replay record should be retained")
            .replay_record_identity(),
        result.replay_record().replay_record_identity()
    );
}
