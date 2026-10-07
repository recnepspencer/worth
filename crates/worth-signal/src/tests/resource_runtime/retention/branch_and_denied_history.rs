use super::*;

#[test]
fn resource_branch_restore_accounts_for_retained_lifecycle_history_width() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(retain_all_transitions_resource_declaration(node))
        .expect("resource declaration should lower");
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    runtime
        .cancel_resource_request(admitted.handle(), ResourceCancellationReason::HostRequested)
        .expect("cancellation should admit");
    let compaction = runtime.compact_resource_lifecycle_history(1);
    assert_eq!(compaction.retained_history_width(), 1);
    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("post-snapshot request should mutate resource state");
    runtime
        .restore_snapshot(&snapshot)
        .expect("restore should reinstate retained lifecycle history");
    let restore = runtime
        .latest_resource_branch_restore_report()
        .expect("restore should publish resource branch evidence");

    assert_eq!(restore.restored_in_flight_width(), 0);
    assert_eq!(restore.retained_summary_width(), 2);
    assert_eq!(restore.performance().input_width(), 2);
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_branch_restore_retained_summary_width,
        2
    );
}

#[test]
fn resource_retention_budget_prunes_denied_completion_history_with_typed_availability() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(resource_declaration(node))
        .expect("resource declaration should lower");
    let digest = runtime
        .resource_descriptor_for_node(ResourceNodeId::from_node(node))
        .expect("descriptor should exist")
        .payload_contract_digest()
        .clone();

    let first_denied = runtime
        .admit_resource_completion(RawCompletionEnvelope::new(
            ResourceRequestId::new(900),
            ResourceGeneration::new(1),
            ResourceBranchEpoch::new(runtime.graph().current_branch().id, 0),
            ResourceAttemptId::ZERO,
            digest.clone(),
            32,
        ))
        .denied_completion()
        .expect("unknown request should retain denied completion evidence");
    let second_denied = runtime
        .admit_resource_completion(RawCompletionEnvelope::new(
            ResourceRequestId::new(901),
            ResourceGeneration::new(1),
            ResourceBranchEpoch::new(runtime.graph().current_branch().id, 0),
            ResourceAttemptId::ZERO,
            digest,
            48,
        ))
        .denied_completion()
        .expect("second unknown request should retain denied completion evidence");

    let report = runtime.compact_resource_lifecycle_history_with_budget(
        0,
        ResourceRetentionCompactionBudget::unbounded().with_retained_denied_completion_limit(1),
    );

    assert_eq!(report.selected_terminal_count(), 0);
    assert_eq!(report.reclaimed_in_flight_count(), 0);
    assert_eq!(report.retained_denied_completion_pruned_count(), 1);
    assert_eq!(report.retained_denied_completion_width(), 1);
    assert_eq!(
        runtime.resource_runtime_summary().denied_completion_count(),
        1
    );
    assert_eq!(
        runtime
            .resource_runtime_summary()
            .retained_denied_completion_count(),
        1
    );
    let availability = runtime
        .retained_denied_completion_availability(first_denied.denial_id())
        .expect("oldest denied completion should become typed unavailable history");
    assert_eq!(
        availability.class(),
        ResourceRetainedDeniedCompletionAvailabilityClass::PrunedByRetainedDeniedCompletionLimit
    );
    assert_eq!(availability.denial_id(), first_denied.denial_id());
    assert_eq!(availability.request_id(), first_denied.request_id());
    assert_eq!(availability.node(), first_denied.node());
    assert_eq!(availability.denial_class(), first_denied.class());
    assert!(
        runtime
            .retained_denied_completion_availability(second_denied.denial_id())
            .is_none(),
        "newest denied completion should remain retained rather than pruned"
    );
    let replay = runtime.reconstruct_resource_replay_summary();
    assert_eq!(replay.denied_completion_width(), 1);
    assert_eq!(replay.denied_completion_unavailable_count(), 1);
    assert_eq!(replay.retry_lineage_unavailable_count(), 0);
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_retained_denied_completion_count,
        1
    );

    let expiry = runtime.compact_resource_lifecycle_history_with_budget(
        0,
        ResourceRetentionCompactionBudget::unbounded().with_pruned_availability_limits(1, 0, 1),
    );
    assert_eq!(expiry.expired_denied_availability_count(), 1);
    assert_eq!(expiry.total_expired_denied_availability(), 1);
    assert!(runtime
        .retained_denied_completion_availability(first_denied.denial_id())
        .is_none());
    assert_eq!(
        runtime
            .reconstruct_resource_replay_summary()
            .denied_completion_unavailable_count(),
        1,
        "replay must report the omission without inventing the expired denial"
    );
}
