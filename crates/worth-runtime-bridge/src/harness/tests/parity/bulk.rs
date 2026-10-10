use super::*;

#[test]
fn bridge_bulk_planning_truth_is_invariant_across_diagnostics_tiers() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let request = BridgeBulkWorkloadRequest::new(vec![
        BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a())),
        BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_b())),
    ]);

    let development_source = crate::harness::fixtures::InMemoryRelationalBridgeSource::default();
    development_source.insert_committed_patch(committed_patch(
        commit_a(),
        patch_a(),
        snapshot_a(),
        name_field_key(),
    ));
    development_source.insert_committed_patch(committed_patch(
        commit_b(),
        patch_b(),
        snapshot_b(),
        name_field_key(),
    ));
    development_source.insert_snapshot(snapshot(snapshot_a(), "alice"));
    development_source.insert_snapshot(snapshot(snapshot_b(), "bob"));
    let development = crate::facade::RuntimeBridge::builder()
        .with_relational_source(development_source)
        .with_signal_sink(crate::harness::fixtures::RecordingSignalBridgeSink::default())
        .with_policy(crate::facade::BridgeRuntimePolicy::development())
        .register_mapping(registration())
        .build()
        .expect("development runtime");

    let operational_source = crate::harness::fixtures::InMemoryRelationalBridgeSource::default();
    operational_source.insert_committed_patch(committed_patch(
        commit_a(),
        patch_a(),
        snapshot_a(),
        name_field_key(),
    ));
    operational_source.insert_committed_patch(committed_patch(
        commit_b(),
        patch_b(),
        snapshot_b(),
        name_field_key(),
    ));
    operational_source.insert_snapshot(snapshot(snapshot_a(), "alice"));
    operational_source.insert_snapshot(snapshot(snapshot_b(), "bob"));
    let operational = crate::facade::RuntimeBridge::builder()
        .with_relational_source(operational_source)
        .with_signal_sink(crate::harness::fixtures::RecordingSignalBridgeSink::default())
        .with_policy(crate::facade::BridgeRuntimePolicy::operational())
        .register_mapping(registration())
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

    let admitted_source = crate::harness::fixtures::InMemoryRelationalBridgeSource::default();
    admitted_source.insert_committed_patch(committed_patch(
        commit_a(),
        patch_a(),
        snapshot_a(),
        name_field_key(),
    ));
    admitted_source.insert_committed_patch(committed_patch(
        commit_b(),
        patch_b(),
        snapshot_b(),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    admitted_source.insert_snapshot(snapshot(snapshot_a(), "alice"));
    admitted_source.insert_snapshot(snapshot(snapshot_b(), "bob"));
    let admitted_sink = crate::harness::fixtures::RecordingSignalBridgeSink::default();
    let admitted_runtime =
        build_runtime(admitted_source, admitted_sink.clone(), vec![registration()]);

    let admitted_result = admitted_runtime
        .deliver_bulk_workload_plan(
            admitted_runtime
                .plan_bulk_workload(
                    BridgeBulkWorkloadRequest::new(vec![
                        BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a())),
                        BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_b())),
                    ]),
                    execution,
                )
                .expect("parallel-admitted bulk workload should plan"),
            execution,
        )
        .expect("parallel-admitted bulk workload should deliver");

    let serial_source = crate::harness::fixtures::InMemoryRelationalBridgeSource::default();
    serial_source.insert_committed_patch(committed_patch(
        commit_a(),
        patch_a(),
        snapshot_a(),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    serial_source.insert_committed_patch(committed_patch(
        commit_b(),
        patch_b(),
        snapshot_a(),
        name_field_key(),
    ));
    serial_source.insert_snapshot(snapshot(snapshot_a(), "alice"));
    let serial_sink = crate::harness::fixtures::RecordingSignalBridgeSink::default();
    let serial_runtime = build_runtime(serial_source, serial_sink.clone(), vec![registration()]);

    let serial_result = serial_runtime
        .deliver_bulk_workload_plan(
            serial_runtime
                .plan_bulk_workload(
                    BridgeBulkWorkloadRequest::new(vec![
                        BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a())),
                        BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_b())),
                    ]),
                    execution,
                )
                .expect("serial-required bulk workload should plan"),
            execution,
        )
        .expect("serial-required bulk workload should deliver");

    assert_eq!(admitted_result.summary().delivered_route_count(), 2);
    assert_eq!(serial_result.summary().delivered_route_count(), 2);
    assert_eq!(admitted_result.summary().delivered_target_count(), 2);
    assert_eq!(serial_result.summary().delivered_target_count(), 2);
    assert_eq!(admitted_sink.deliveries().len(), 2);
    assert_eq!(serial_sink.deliveries().len(), 2);
}
