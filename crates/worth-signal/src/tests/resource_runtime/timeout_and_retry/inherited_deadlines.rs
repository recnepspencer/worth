use super::*;

#[test]
fn resource_transaction_inherited_deadline_times_out_with_transaction_authority() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(transaction_inherited_deadline_resource_declaration(node))
        .expect("transaction inherited deadline declaration should lower");
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::with_transaction_deadline(
            ResourceNodeId::from_node(node),
            TemporalDuration::temporal_duration(6).unwrap(),
        ))
        .expect("request with inherited deadline should admit")
        .admitted_request();
    let wake_id = runtime
        .in_flight_resource_request(admitted.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("timeout wake should attach");

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(6),
        ))
        .expect("clock should reach inherited deadline");
    let ready = runtime
        .promote_temporal_wake_ready(wake_id)
        .expect("deadline wake should become ready");
    let report = runtime
        .admit_resource_timeout(admitted.handle(), ready)
        .expect("timeout admission should succeed");
    let timed_out = report
        .timed_out_request()
        .expect("deadline timeout should produce timeout artifact");

    assert_eq!(timed_out.timeout_duration().get(), 6);
    assert_eq!(
        timed_out.deadline_authority(),
        ResourceTimeoutDeadlineAuthority::TransactionIntent
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_deadline_inherited_count,
        1
    );
}

#[test]
fn resource_transaction_inherited_deadline_denies_missing_transaction_deadline() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(transaction_inherited_deadline_resource_declaration(node))
        .expect("transaction inherited deadline declaration should lower");

    let err = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect_err("missing transaction deadline should deny request admission");

    assert!(err.to_string().contains("transaction-inherited deadline"));
    assert_eq!(runtime.temporal_wake_summary().scheduled_count(), 0);
}

#[test]
fn resource_runtime_inherited_deadline_uses_runtime_config_authority() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(runtime_inherited_deadline_resource_declaration(node))
        .expect("runtime inherited deadline declaration should lower");
    runtime
        .config_mut()
        .set_resource_runtime_deadline(TemporalDuration::temporal_duration(8).unwrap());

    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should inherit runtime deadline")
        .admitted_request();
    let wake_id = runtime
        .in_flight_resource_request(admitted.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("timeout wake should attach");

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(8),
        ))
        .expect("clock should reach runtime deadline");
    let ready = runtime
        .promote_temporal_wake_ready(wake_id)
        .expect("deadline wake should become ready");
    let report = runtime
        .admit_resource_timeout(admitted.handle(), ready)
        .expect("timeout admission should succeed");
    let timed_out = report
        .timed_out_request()
        .expect("runtime deadline timeout should produce timeout artifact");

    assert_eq!(timed_out.timeout_duration().get(), 8);
    assert_eq!(
        timed_out.deadline_authority(),
        ResourceTimeoutDeadlineAuthority::RuntimeConfig
    );
}

#[test]
fn resource_inherited_deadline_retry_denies_when_backoff_outlives_preserved_deadline() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(retry_transaction_inherited_deadline_resource_declaration(
            node, 7,
        ))
        .expect("inherited deadline retry declaration should lower");
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::with_transaction_deadline(
            ResourceNodeId::from_node(node),
            TemporalDuration::temporal_duration(3).unwrap(),
        ))
        .expect("request with inherited deadline should admit")
        .admitted_request();
    let timeout_wake = runtime
        .in_flight_resource_request(admitted.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("timeout wake should attach");

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(3),
        ))
        .expect("clock should reach inherited deadline");
    let ready_timeout = runtime
        .promote_temporal_wake_ready(timeout_wake)
        .expect("timeout wake should promote when due");
    runtime
        .admit_resource_timeout(admitted.handle(), ready_timeout)
        .expect("timeout admission should consume inherited deadline wake");

    let schedule = runtime
        .schedule_resource_retry(admitted.handle(), ResourceRetryReason::TimedOut)
        .expect("retry scheduling should remain report-shaped");
    let scheduled = schedule
        .scheduled_retry()
        .expect("retry backoff should still schedule before admission-time denial");

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(10),
        ))
        .expect("clock should reach retry backoff");
    let ready_retry = runtime
        .promote_temporal_wake_ready(scheduled.backoff_wake_id())
        .expect("retry wake should become ready");
    let report = runtime
        .admit_scheduled_resource_retry(admitted.handle(), ready_retry)
        .expect("expired inherited deadline should stay report-shaped");
    let performance = report.performance();
    let denied = report
        .denied_retry()
        .expect("expired inherited deadline must deny retry admission");

    assert_eq!(
        denied.class(),
        ResourceRetryDenialClass::RetryTimeoutWindowExhausted
    );
    assert_eq!(performance.temporal_wake_footprint(), 0);
    assert_eq!(runtime.temporal_wake_summary().ready_count(), 0);
    assert_eq!(
        runtime
            .resource_runtime_summary()
            .active_in_flight_node_count(),
        0
    );
    assert_eq!(
        runtime
            .in_flight_resource_request(admitted.handle())
            .expect("timed out request should stay retained for audit")
            .status(),
        ResourceInFlightStatus::TimedOut
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_retry_timeout_window_exhaustion_denial_count,
        1
    );
}
