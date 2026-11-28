use super::*;
use crate::harness::fixtures::NamedTruth;

#[test]
fn bridge_bulk_planning_truth_is_invariant_across_diagnostics_tiers() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let (commit_b, _observation_b) = truth.commit_name(execution, "bob");
    let request = BridgeBulkWorkloadRequest::new(vec![
        BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a)),
        BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_b)),
    ]);
    let development_source = truth.source.clone();

    let development = crate::facade::RuntimeBridge::builder()
        .with_relational_source(development_source)
        .with_signal_sink(crate::harness::fixtures::RecordingSignalBridgeSink::default())
        .with_policy(crate::facade::BridgeRuntimePolicy::development())
        .register_mapping(NamedTruth::mapping())
        .build()
        .expect("development runtime");

    let operational_source = truth.source.clone();

    let operational = crate::facade::RuntimeBridge::builder()
        .with_relational_source(operational_source)
        .with_signal_sink(crate::harness::fixtures::RecordingSignalBridgeSink::default())
        .with_policy(crate::facade::BridgeRuntimePolicy::operational())
        .register_mapping(NamedTruth::mapping())
        .build()
        .expect("operational runtime");

    let development_plan = development
        .plan_bulk_workload(request.clone(), execution)
        .expect("development bulk workload should plan");
    let operational_plan = operational
        .plan_bulk_workload(request, execution)
        .expect("operational bulk workload should plan");

    assert_eq!(
        development_plan.workload_identity(),
        operational_plan.workload_identity()
    );
    assert_eq!(
        development_plan.canonical_request().digest(),
        operational_plan.canonical_request().digest()
    );
    assert_eq!(
        development_plan.normalized_summary().digest(),
        operational_plan.normalized_summary().digest()
    );
    assert_eq!(
        development_plan.canonical_planning_identity(),
        operational_plan.canonical_planning_identity()
    );
    assert_eq!(
        development_plan.packet_set().digest(),
        operational_plan.packet_set().digest()
    );
    assert_eq!(
        development_plan
            .execution_plan()
            .reduced_artifact()
            .digest(),
        operational_plan
            .execution_plan()
            .reduced_artifact()
            .digest()
    );
    assert_eq!(
        development_plan.execution_plan().legality_decision(),
        operational_plan.execution_plan().legality_decision()
    );
    assert_eq!(
        development_plan.execution_plan().profitability_decision(),
        operational_plan.execution_plan().profitability_decision()
    );
}

#[test]
fn parallel_preparation_admission_remains_parity_safe_with_serial_required_path() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let (commit_b, observation_b) = truth.commit_name(execution, "bob");
    let snapshot = observation_b.snapshot_identity().clone();
    let compare =
        |request_a: BridgeRouteRequest, request_b: BridgeRouteRequest, expected_admission| {
            let sink = crate::harness::fixtures::RecordingSignalBridgeSink::default();
            let runtime = build_runtime(
                truth.source.clone(),
                sink.clone(),
                vec![NamedTruth::mapping()],
            );
            let plan = runtime
                .plan_bulk_workload(
                    BridgeBulkWorkloadRequest::new(vec![
                        BridgeBulkWorkloadSegment::new(request_a),
                        BridgeBulkWorkloadSegment::new(request_b),
                    ]),
                    execution,
                )
                .expect("native workload should plan");
            assert_eq!(
                plan.execution_plan().parallel_admission().class(),
                expected_admission
            );
            let result = runtime
                .deliver_bulk_workload_plan(plan, execution)
                .expect("native workload should deliver");
            assert_eq!(result.summary().delivered_route_count(), 2);
            assert_eq!(result.summary().delivered_target_count(), 2);
            assert_eq!(sink.deliveries().len(), 2);
        };

    compare(
        BridgeRouteRequest::for_commit(commit_a.clone()),
        BridgeRouteRequest::for_commit(commit_b.clone()),
        crate::facade::BridgeParallelAdmissionClass::ParallelPreparationAdmitted,
    );
    let at_snapshot = |commit| {
        crate::facade::RelationalCommittedPatchRequest::at_snapshot(commit, snapshot.clone()).into()
    };
    compare(
        at_snapshot(commit_a),
        at_snapshot(commit_b),
        crate::facade::BridgeParallelAdmissionClass::SerialRequired,
    );
}
