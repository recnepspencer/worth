use worth_harness::facade::{parity_suite, ExecutionProfile, ExecutionRequest};

use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};

use super::super::support::{build_runtime, committed_patch, registration, snapshot};
use super::support::{replay_audit_target, routing_target, stream_fixture};

#[test]
fn bridge_harness_stream_consumer_exports_are_host_parity_safe() {
    assert_stream_target_exports_are_host_parity_safe("stream-routing", routing_target());
    assert_stream_target_exports_are_host_parity_safe("stream-replay-audit", replay_audit_target());
}

#[test]
fn consumer_pacing_exports_remain_host_parity_safe() {
    bridge_harness_stream_consumer_exports_are_host_parity_safe();
}

fn assert_stream_target_exports_are_host_parity_safe(
    request_name: &str,
    target: BridgeHarnessTargetId,
) {
    let report = parity_suite(
        BridgeHarnessAdapter,
        stream_fixture("bridge-stream-multi-consumer"),
        ExecutionRequest::target(request_name, target),
        ExecutionProfile::development("baseline"),
    )
    .candidates([ExecutionProfile::operational("operational")])
    .compare()
    .expect("stream target parity should compare cleanly");

    assert!(report.matched);
    assert_eq!(report.results.len(), 1);
}

#[test]
fn illegal_coalescing_boundary_fails_explicitly() {
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
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        "alice",
    ));
    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
    );
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

    let error = runtime
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
        .expect_err("illegal coalescing boundaries must fail explicitly");

    assert_eq!(
        error.kind(),
        crate::error::BridgeStreamErrorKind::IllegalCoalescingBoundary
    );
}
