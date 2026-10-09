use super::*;

#[test]
fn opt_in_availability_tail_expires_old_diagnostics_without_authorizing_late_completion() {
    let mut graph = SignalGraph::new();
    let nodes = (0..3).map(|_| graph.node().build()).collect::<Vec<_>>();
    let mut runtime = TestRuntime::build(graph);
    let mut admitted = Vec::new();
    for node in &nodes {
        runtime
            .declare_resource_node(terminal_summaries_only_resource_declaration(*node))
            .expect("resource declaration should lower");
        let request = runtime
            .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(*node)))
            .expect("request should admit")
            .admitted_request();
        runtime
            .cancel_resource_request(request.handle(), ResourceCancellationReason::HostRequested)
            .expect("cancellation should admit");
        admitted.push(request);
    }

    let report = runtime.compact_resource_lifecycle_history_with_budget(
        3,
        ResourceRetentionCompactionBudget::unbounded().with_pruned_availability_limits(1, 1, 1),
    );
    assert_eq!(report.reclaimed_in_flight_count(), 3);
    assert_eq!(report.expired_lifecycle_availability_count(), 2);
    assert_eq!(report.total_expired_lifecycle_availability(), 2);
    assert!(runtime
        .retained_history_availability_for_request(admitted[0].handle().request_id())
        .is_none());
    assert!(runtime
        .retained_history_availability_for_request(admitted[2].handle().request_id())
        .is_some());

    let expected_unavailable = runtime
        .resource_runtime_summary()
        .retained_history_unavailable_count();
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("resource-retention-omission-fork")
        .expect("branch must fork the retained omission state");
    runtime.switch_branch(feature).expect("fork must activate");
    assert_eq!(
        runtime
            .resource_runtime_summary()
            .retained_history_unavailable_count(),
        expected_unavailable
    );
    assert!(runtime
        .retained_history_availability_for_request(admitted[2].handle().request_id())
        .is_some());
    let snapshot = runtime
        .capture_snapshot()
        .expect("forked omission-bearing resource state must be snapshot-capable");
    let fork_replay = runtime
        .reconstruct_resource_replay_summary()
        .replay_digest()
        .to_owned();
    runtime.compact_resource_lifecycle_history_with_budget(
        0,
        ResourceRetentionCompactionBudget::unbounded().with_pruned_availability_limits(0, 1, 1),
    );
    assert_ne!(
        runtime
            .reconstruct_resource_replay_summary()
            .replay_digest(),
        fork_replay,
        "changing the omitted history must change replay proof even at equal unavailable count"
    );
    runtime
        .restore_snapshot(&snapshot)
        .expect("restore must reinstate the omission-bearing snapshot");
    assert_eq!(
        runtime
            .resource_runtime_summary()
            .retained_history_unavailable_count(),
        expected_unavailable,
        "restore must retain the expired count as well as the exact tail"
    );
    assert!(runtime
        .retained_history_availability_for_request(admitted[0].handle().request_id())
        .is_none());
    assert!(runtime
        .retained_history_availability_for_request(admitted[2].handle().request_id())
        .is_some());
    runtime
        .switch_branch(main)
        .expect("main branch must remain independent");
    assert_eq!(
        runtime
            .resource_runtime_summary()
            .retained_history_unavailable_count(),
        expected_unavailable
    );

    let old = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            nodes[0],
            admitted[0].handle(),
            admitted[0].attempt(),
            64,
        ))
        .denied_completion()
        .expect("expired evidence cannot admit old completion");
    assert_eq!(old.class(), CompletionDenialClass::UnknownRequest);
    let recent = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            nodes[2],
            admitted[2].handle(),
            admitted[2].attempt(),
            64,
        ))
        .denied_completion()
        .expect("retained tail classifies recent completion exactly");
    assert_eq!(recent.class(), CompletionDenialClass::Cancelled);
    assert!(
        runtime
            .reconstruct_resource_replay_summary()
            .retained_history_unavailable_count()
            >= 2
    );
}
