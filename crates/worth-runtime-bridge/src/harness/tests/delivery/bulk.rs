use super::*;
use crate::harness::fixtures::NamedTruth;

#[test]
fn bridge_bulk_delivery_keeps_preplanned_snapshots_after_newer_truth_arrives() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (first, first_observation) = truth.commit_name(execution, "alice");
    let (second, second_observation) = truth.commit_name(execution, "bob");
    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_runtime(
        truth.source.clone(),
        sink.clone(),
        vec![NamedTruth::mapping()],
    );

    let plan = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(first.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(second)),
            ]),
            execution,
        )
        .expect("bridge should plan the bulk workload");

    let (_, newer) = truth.commit_name(execution, "carol");
    assert_ne!(
        newer.snapshot_identity(),
        first_observation.snapshot_identity()
    );
    assert_ne!(
        newer.snapshot_identity(),
        second_observation.snapshot_identity()
    );

    let result = runtime
        .deliver_bulk_workload_plan(plan, execution)
        .expect("bridge should deliver the preplanned bulk workload");

    assert_eq!(
        result.summary().selected_mode(),
        BridgePreparationMode::ParallelPreparation
    );
    assert_eq!(result.summary().delivered_route_count(), 2);
    assert_eq!(result.route_results().len(), 2);
    let mut delivered_snapshot_identities = result
        .route_results()
        .iter()
        .map(|route_result| {
            route_result
                .result_summary()
                .snapshot_identity()
                .relational_snapshot_parts()
                .expect("delivery snapshots should carry typed relational parts")
        })
        .collect::<Vec<_>>();
    delivered_snapshot_identities.sort();
    assert_eq!(
        delivered_snapshot_identities,
        vec![
            first_observation
                .snapshot_identity()
                .relational_snapshot_parts()
                .unwrap(),
            second_observation
                .snapshot_identity()
                .relational_snapshot_parts()
                .unwrap()
        ]
    );
    let mut sink_snapshot_identities = sink
        .deliveries()
        .iter()
        .map(|delivery| {
            delivery
                .delivery
                .source_snapshot()
                .relational_snapshot_parts()
                .expect("sink delivery snapshots should carry typed relational parts")
        })
        .collect::<Vec<_>>();
    sink_snapshot_identities.sort();
    assert_eq!(
        sink_snapshot_identities,
        vec![
            first_observation
                .snapshot_identity()
                .relational_snapshot_parts()
                .unwrap(),
            second_observation
                .snapshot_identity()
                .relational_snapshot_parts()
                .unwrap()
        ]
    );
}

#[test]
fn bridge_bulk_delivery_accepts_replayed_canonical_bulk_plan() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (first, _first_observation) = truth.commit_name(execution, "alice");
    let (second, _second_observation) = truth.commit_name(execution, "bob");
    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_runtime(truth.source, sink.clone(), vec![NamedTruth::mapping()]);

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(first.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(second)),
            ]),
            execution,
        )
        .expect("bridge should plan the bulk workload before replay-backed delivery");
    let canonical = runtime.canonicalize_bulk_workload_plan(&planned);
    let replayed = runtime
        .replay_canonical_bulk_plan_record(&canonical, execution)
        .expect("bridge should replay the canonical bulk plan before delivery");

    let result = runtime
        .deliver_bulk_workload_plan(replayed, execution)
        .expect("bridge should deliver a replayed canonical bulk plan");

    assert_eq!(
        result.summary().canonical_planning_identity(),
        planned.canonical_planning_identity()
    );
    assert_eq!(
        result.summary().execution_plan_digest(),
        planned.execution_plan().digest()
    );
    assert_eq!(
        result.summary().reduced_artifact_digest(),
        planned.execution_plan().reduced_artifact().digest()
    );
    assert_eq!(
        result.summary().counters(),
        planned.execution_plan().counters()
    );
    assert_eq!(result.summary().delivered_route_count(), 2);
    assert_eq!(result.summary().delivered_target_count(), 2);
    assert_eq!(sink.deliveries().len(), 2);
}

#[test]
fn bridge_bulk_delivery_is_stable_across_input_order() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (first, _first) = truth.commit_name(execution, "alice");
    let (second, _second) = truth.commit_name(execution, "bob");
    let left_sink = RecordingSignalBridgeSink::default();
    let right_sink = RecordingSignalBridgeSink::default();
    let left_runtime = build_runtime(
        truth.source.clone(),
        left_sink.clone(),
        vec![NamedTruth::mapping()],
    );
    let right_runtime = build_runtime(
        truth.source,
        right_sink.clone(),
        vec![NamedTruth::mapping()],
    );

    let left_plan = left_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(first.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(second.clone())),
            ]),
            execution,
        )
        .expect("left bulk workload should plan");
    let right_plan = right_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(second.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(first.clone())),
            ]),
            execution,
        )
        .expect("right bulk workload should plan");

    let left_result = left_runtime
        .deliver_bulk_workload_plan(left_plan, execution)
        .expect("left bulk workload should deliver");
    let right_result = right_runtime
        .deliver_bulk_workload_plan(right_plan, execution)
        .expect("right bulk workload should deliver");

    assert_eq!(left_result.summary(), right_result.summary());
    assert_eq!(
        left_result
            .route_results()
            .iter()
            .map(|result| result.result_summary().route_identity().clone())
            .collect::<Vec<_>>(),
        right_result
            .route_results()
            .iter()
            .map(|result| result.result_summary().route_identity().clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        left_sink
            .deliveries()
            .iter()
            .map(|delivery| delivery.delivery.invalidation_identity().clone())
            .collect::<Vec<_>>(),
        right_sink
            .deliveries()
            .iter()
            .map(|delivery| delivery.delivery.invalidation_identity().clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn bridge_bulk_delivery_replay_matches_original_serial_reduction_path() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (first, _first_observation) = truth.commit_name(execution, "alice");
    let (second, second_observation) = truth.commit_name(execution, "bob");
    let shared_snapshot = second_observation.snapshot_identity().clone();
    let original_sink = RecordingSignalBridgeSink::default();
    let original_runtime = build_runtime(
        truth.source.clone(),
        original_sink.clone(),
        vec![NamedTruth::mapping()],
    );

    let original_plan = original_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::from(
                    crate::facade::RelationalCommittedPatchRequest::at_snapshot(
                        first.clone(),
                        shared_snapshot.clone(),
                    ),
                )),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::from(
                    crate::facade::RelationalCommittedPatchRequest::at_snapshot(
                        second,
                        shared_snapshot.clone(),
                    ),
                )),
            ]),
            execution,
        )
        .expect("original serial-reduction bulk workload should plan");
    let canonical = original_runtime.canonicalize_bulk_workload_plan(&original_plan);
    let original_result = original_runtime
        .deliver_bulk_workload_plan(original_plan, execution)
        .expect("original serial-reduction bulk workload should deliver");

    let replay_sink = RecordingSignalBridgeSink::default();
    let replay_runtime = build_runtime(
        truth.source,
        replay_sink.clone(),
        vec![NamedTruth::mapping()],
    );
    let replayed_plan = replay_runtime
        .replay_canonical_bulk_plan_record(&canonical, execution)
        .expect("replayed serial-reduction bulk workload should reconstruct");
    let replayed_result = replay_runtime
        .deliver_bulk_workload_plan(replayed_plan, execution)
        .expect("replayed serial-reduction bulk workload should deliver");

    assert_eq!(original_result.summary(), replayed_result.summary());
    assert_eq!(
        original_result.summary().selected_mode(),
        BridgePreparationMode::Serial
    );
    assert_eq!(
        original_result
            .route_results()
            .iter()
            .map(|result| result.result_summary().invalidation_identity().clone())
            .collect::<Vec<_>>(),
        replayed_result
            .route_results()
            .iter()
            .map(|result| result.result_summary().invalidation_identity().clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        original_sink
            .deliveries()
            .iter()
            .map(|delivery| delivery.delivery.invalidation_identity().clone())
            .collect::<Vec<_>>(),
        replay_sink
            .deliveries()
            .iter()
            .map(|delivery| delivery.delivery.invalidation_identity().clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn bridge_bulk_delivery_rejects_invalid_parallel_upgrade_for_rejected_plan() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (first, _first_observation) = truth.commit_name(execution, "alice");
    let runtime = build_runtime(
        truth.source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let rejected_plan = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(first.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(first.clone())),
            ]),
            execution,
        )
        .expect("shared-truth-view workload should plan before rejection certification");

    let error = crate::delivery::validate_bulk_delivery_mode(
        rejected_plan.execution_plan(),
        BridgePreparationMode::ParallelPreparation,
    )
    .expect_err("bulk delivery should reject a plan that upgrades a rejected class");

    assert_eq!(error.kind(), BridgeDeliveryErrorKind::BulkDeliveryRejected);
    assert_eq!(
        rejected_plan.execution_plan().selected_mode(),
        BridgePreparationMode::Serial
    );
    assert_eq!(
        rejected_plan.execution_plan().parallel_admission().class(),
        BridgeParallelAdmissionClass::ParallelPreparationRejected
    );
}
