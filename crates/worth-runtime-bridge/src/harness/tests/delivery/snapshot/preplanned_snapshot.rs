use super::super::build_runtime;
use crate::facade::BridgeRouteRequest;
use crate::harness::fixtures::{NamedTruth, RecordingSignalBridgeSink};
use worth_execution::ExecutionRequest;

#[test]
fn bridge_snapshot_delivery_remains_stable_after_newer_truth_arrives() {
    let serial = crate::snapshot::test_serial_request();
    let execution = ExecutionRequest::serial(&serial);
    let truth = NamedTruth::new();
    let (original_commit, original) = truth.commit_name(execution, "alice");
    let (_, newer) = truth.commit_name(execution, "bob");
    let runtime = build_runtime(
        truth.source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let routed = runtime.route(original_commit, execution).unwrap();

    assert_ne!(original.snapshot_identity(), newer.snapshot_identity());
    assert_eq!(
        routed.result().receipt().snapshot_identity(),
        original.snapshot_identity()
    );
    assert_eq!(
        runtime
            .diagnostics()
            .last_route_record()
            .unwrap()
            .source_snapshot(),
        original.snapshot_identity()
    );
}

#[test]
fn bridge_delivery_keeps_preplanned_snapshot_after_newer_truth_arrives() {
    let serial = crate::snapshot::test_serial_request();
    let execution = ExecutionRequest::serial(&serial);
    let truth = NamedTruth::new();
    let (original_commit, original) = truth.commit_name(execution, "alice");
    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_runtime(
        truth.source.clone(),
        sink.clone(),
        vec![NamedTruth::mapping()],
    );
    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(original_commit), execution)
        .unwrap();
    let (_, newer) = truth.commit_name(execution, "bob");

    let result = runtime.deliver_invalidation(route, execution).unwrap();

    assert_ne!(original.snapshot_identity(), newer.snapshot_identity());
    assert_eq!(
        result.result_summary().snapshot_identity(),
        original.snapshot_identity()
    );
    assert_eq!(
        result.receipt().snapshot_identity(),
        original.snapshot_identity()
    );
    let delivered = sink.last_delivery().unwrap();
    assert!(!delivered.leased);
    assert_eq!(
        delivered.delivery.source_snapshot(),
        original.snapshot_identity()
    );
}

#[test]
fn leased_facade_delivery_carries_the_lease_to_the_signal_sink() {
    let serial = crate::snapshot::test_serial_request();
    let execution = ExecutionRequest::serial(&serial);
    let truth = NamedTruth::new();
    let (commit, _observation) = truth.commit_name(execution, "alice");
    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_runtime(truth.source, sink.clone(), vec![NamedTruth::mapping()]);
    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit), execution)
        .unwrap();
    let lease = crate::snapshot::test_execution_lease(worth_execution::CancellationToken::new());

    runtime
        .deliver_invalidation(route, ExecutionRequest::leased(&lease))
        .unwrap();

    assert!(sink.last_delivery().unwrap().leased);
}

#[test]
fn bridge_prepares_signal_evaluation_with_snapshot_context_without_sink_delivery() {
    let serial = crate::snapshot::test_serial_request();
    let execution = ExecutionRequest::serial(&serial);
    let truth = NamedTruth::new();
    let (commit, observation) = truth.commit_name(execution, "alice");
    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_runtime(truth.source, sink.clone(), vec![NamedTruth::mapping()]);
    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit), execution)
        .unwrap();

    let evaluation = runtime.prepare_signal_evaluation(route, execution).unwrap();

    assert_eq!(
        evaluation.snapshot().snapshot_identity(),
        observation.snapshot_identity()
    );
    assert!(sink.last_delivery().is_none());
}

#[test]
fn bridge_prepared_signal_evaluation_keeps_preplanned_snapshot_after_newer_truth_arrives() {
    let serial = crate::snapshot::test_serial_request();
    let execution = ExecutionRequest::serial(&serial);
    let truth = NamedTruth::new();
    let (original_commit, original) = truth.commit_name(execution, "alice");
    let runtime = build_runtime(
        truth.source.clone(),
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );
    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(original_commit), execution)
        .unwrap();
    let (_, newer) = truth.commit_name(execution, "bob");

    let evaluation = runtime.prepare_signal_evaluation(route, execution).unwrap();

    assert_ne!(original.snapshot_identity(), newer.snapshot_identity());
    assert_eq!(
        evaluation.snapshot().snapshot_identity(),
        original.snapshot_identity()
    );
}
