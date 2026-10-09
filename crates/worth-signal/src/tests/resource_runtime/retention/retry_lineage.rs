use super::*;

#[test]
fn resource_retention_budget_prunes_retry_lineage_with_typed_availability() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(retry_timeout_resource_declaration(node, 3, 7))
        .expect("retry declaration should lower");

    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("initial request should admit")
        .admitted_request();
    let first_timeout_wake = runtime
        .in_flight_resource_request(admitted.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("initial timeout wake should attach");
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(runtime.clock_basis().current_tick().get().saturating_add(3)),
        ))
        .expect("clock should reach initial timeout");
    let first_ready_timeout = runtime
        .promote_temporal_wake_ready(first_timeout_wake)
        .expect("initial timeout wake should become ready");
    runtime
        .admit_resource_timeout(admitted.handle(), first_ready_timeout)
        .expect("initial timeout should admit");

    let first_schedule = runtime
        .schedule_resource_retry(admitted.handle(), ResourceRetryReason::TimedOut)
        .expect("first retry schedule should return report");
    let first_scheduled = first_schedule
        .scheduled_retry()
        .expect("first retry should schedule");
    let first_retry_ordinal = first_scheduled.retry_ordinal();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(
                runtime
                    .clock_basis()
                    .current_tick()
                    .get()
                    .saturating_add(first_scheduled.scheduled_delay().get()),
            ),
        ))
        .expect("clock should reach first retry backoff");
    let first_ready_retry = runtime
        .promote_temporal_wake_ready(first_scheduled.backoff_wake_id())
        .expect("first retry wake should become ready");
    let first_retry_report = runtime
        .admit_scheduled_resource_retry(admitted.handle(), first_ready_retry)
        .expect("first scheduled retry should admit");
    let first_retry = first_retry_report
        .admitted_retry()
        .expect("first retry should produce admitted retry artifact");
    let first_retry_request = first_retry.admitted_request();

    let second_timeout_wake = runtime
        .in_flight_resource_request(first_retry_request.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("retried request should attach timeout wake");
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(runtime.clock_basis().current_tick().get().saturating_add(3)),
        ))
        .expect("clock should reach second timeout");
    let second_ready_timeout = runtime
        .promote_temporal_wake_ready(second_timeout_wake)
        .expect("second timeout wake should become ready");
    runtime
        .admit_resource_timeout(first_retry_request.handle(), second_ready_timeout)
        .expect("second timeout should admit");

    let second_schedule = runtime
        .schedule_resource_retry(first_retry_request.handle(), ResourceRetryReason::TimedOut)
        .expect("second retry schedule should return report");
    let second_scheduled = second_schedule
        .scheduled_retry()
        .expect("second retry should schedule");
    let second_retry_ordinal = second_scheduled.retry_ordinal();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(
                runtime
                    .clock_basis()
                    .current_tick()
                    .get()
                    .saturating_add(second_scheduled.scheduled_delay().get()),
            ),
        ))
        .expect("clock should reach second retry backoff");
    let second_ready_retry = runtime
        .promote_temporal_wake_ready(second_scheduled.backoff_wake_id())
        .expect("second retry wake should become ready");
    runtime
        .admit_scheduled_resource_retry(first_retry_request.handle(), second_ready_retry)
        .expect("second scheduled retry should admit");

    let report = runtime.compact_resource_lifecycle_history_with_budget(
        0,
        ResourceRetentionCompactionBudget::unbounded().with_retained_retry_lineage_limit(1),
    );

    assert_eq!(report.selected_terminal_count(), 0);
    assert_eq!(report.retained_retry_lineage_pruned_count(), 1);
    assert_eq!(report.retained_retry_lineage_width(), 1);
    assert_eq!(
        runtime
            .resource_runtime_summary()
            .retained_retry_lineage_count(),
        1
    );
    let availability = runtime
        .retained_retry_lineage_availability(first_retry_ordinal)
        .expect("oldest retry lineage should become typed unavailable history");
    assert_eq!(
        availability.class(),
        ResourceRetainedRetryLineageAvailabilityClass::PrunedByRetainedRetryLineageLimit
    );
    assert_eq!(availability.retry_ordinal(), first_retry_ordinal);
    assert_eq!(
        availability.previous(),
        admitted.handle(),
        "pruned lineage should still identify the source request handle"
    );
    assert_eq!(availability.reason(), ResourceRetryReason::TimedOut);
    assert_eq!(availability.next_attempt(), ResourceAttemptId::new(1));
    assert_eq!(availability.scheduled_delay().get(), 7);
    let retained = runtime
        .retained_retry_lineage(second_retry_ordinal)
        .expect("newest retry lineage should remain retained");
    assert_eq!(retained.retry_ordinal(), second_retry_ordinal);
    assert_eq!(retained.reason(), ResourceRetryReason::TimedOut);
    let replay = runtime.reconstruct_resource_replay_summary();
    assert_eq!(replay.retained_retry_lineage_width(), 1);
    assert_eq!(replay.retry_lineage_unavailable_count(), 1);
    assert_eq!(replay.denied_completion_unavailable_count(), 0);
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_retained_retry_lineage_count,
        1
    );

    let expiry = runtime.compact_resource_lifecycle_history_with_budget(
        0,
        ResourceRetentionCompactionBudget::unbounded().with_pruned_availability_limits(1, 1, 0),
    );
    assert_eq!(expiry.expired_retry_availability_count(), 1);
    assert_eq!(expiry.total_expired_retry_availability(), 1);
    assert!(runtime
        .retained_retry_lineage_availability(first_retry_ordinal)
        .is_none());
    assert_eq!(
        runtime
            .reconstruct_resource_replay_summary()
            .retry_lineage_unavailable_count(),
        1,
        "replay must count omitted retry evidence without reconstructing it"
    );
}
