use super::*;

#[test]
fn resource_request_admission_with_timeout_policy_schedules_temporal_wake() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(timeout_resource_declaration(node, 5))
        .expect("resource declaration should lower");

    let report = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("timeout policy should schedule a runtime-owned wake");
    let handle = report.admitted_request().handle();

    let in_flight = runtime
        .in_flight_resource_request(handle)
        .expect("admitted request should be retained in flight");
    assert_eq!(in_flight.timeout_wake_id(), Some(TemporalWakeId::new(0)));
    assert_eq!(report.performance().temporal_wake_footprint(), 1);
    assert_eq!(runtime.temporal_wake_summary().scheduled_count(), 1);
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_timeout_temporal_wake_footprint,
        1
    );
}

#[test]
fn resource_timeout_wake_owner_does_not_alias_node_temporal_owner() {
    let mut graph = SignalGraph::new();
    let node = graph
        .node()
        .after(5)
        .expect("temporal evaluation condition should be valid")
        .build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(timeout_resource_declaration(node, 5))
        .expect("resource declaration should lower");

    runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("resource timeout policy should schedule resource-owned wake");
    let node_wake = runtime
        .admit_node_temporal_wake(node)
        .expect("node temporal wake admission should remain independent");

    assert!(node_wake.is_some());
    assert_eq!(runtime.temporal_wake_summary().scheduled_count(), 2);
}

#[test]
fn resource_timeout_admission_requires_ready_temporal_wake_truth() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(timeout_resource_declaration(node, 5))
        .expect("resource declaration should lower");
    let handle = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request()
        .handle();
    let wake_id = runtime
        .in_flight_resource_request(handle)
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("timeout wake should be attached to request");

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(5),
        ))
        .expect("authoritative clock should advance to timeout tick");
    let ready = runtime
        .promote_temporal_wake_ready(wake_id)
        .expect("timeout wake should promote when due");
    let report = runtime
        .admit_resource_timeout(handle, ready)
        .expect("timeout admission should consume temporal wake cleanly");

    let timed_out = report
        .timed_out_request()
        .expect("matching ready wake should admit timeout");
    let timeout_plan = runtime
        .resource_descriptor_for_node(ResourceNodeId::from_node(node))
        .expect("timeout descriptor should exist")
        .timeout_decision_plan()
        .clone();
    assert_eq!(timed_out.handle(), handle);
    assert_eq!(timed_out.timeout_duration().get(), 5);
    assert!(
        timed_out
            .policy_decision_digest()
            .as_str()
            .starts_with("resolved-timeout-decision:"),
        "timeout artifact should retain resolved timeout admission proof"
    );
    assert!(
        timed_out
            .policy_decision_digest()
            .as_str()
            .contains(timeout_plan.decision_digest().as_str()),
        "resolved timeout digest should remain anchored to the lowered timeout plan digest"
    );
    assert_eq!(
        timed_out.lifecycle_transition().kind(),
        ResourceLifecycleTransitionKind::RequestTimedOut
    );
    assert_eq!(
        report
            .lifecycle()
            .expect("timeout should report lifecycle")
            .lifecycle(),
        ResourceLifecycleClass::TimedOut
    );
    assert_eq!(
        report.performance().boundary(),
        ResourceBoundaryKind::TimeoutAdmission
    );
    assert_eq!(report.performance().temporal_wake_footprint(), 1);
    assert_eq!(runtime.temporal_wake_summary().ready_count(), 0);
    assert_eq!(runtime.temporal_wake_summary().retired_count(), 1);
    assert_eq!(
        runtime
            .resource_runtime_summary()
            .active_in_flight_node_count(),
        0
    );
    assert_eq!(
        runtime
            .in_flight_resource_request(handle)
            .expect("timed out request remains retained for late completion denial")
            .status(),
        ResourceInFlightStatus::TimedOut
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_timeout_admission_count,
        1
    );
}

#[test]
fn resource_total_request_lifetime_timeout_denies_timeout_triggered_retry_after_lineage_deadline() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(retry_total_request_lifetime_timeout_resource_declaration(
            node, 5, 7,
        ))
        .expect("total-lifetime timeout declaration should lower");
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let first_timeout_wake = runtime
        .in_flight_resource_request(admitted.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("initial timeout wake should attach");

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(5),
        ))
        .expect("clock should reach lifetime timeout");
    let first_ready_timeout = runtime
        .promote_temporal_wake_ready(first_timeout_wake)
        .expect("initial timeout wake should become ready");
    runtime
        .admit_resource_timeout(admitted.handle(), first_ready_timeout)
        .expect("initial timeout admission should succeed");

    let retry_schedule_report = runtime
        .schedule_resource_retry(admitted.handle(), ResourceRetryReason::TimedOut)
        .expect("retry scheduling should stay report-shaped");
    let denied = retry_schedule_report
        .denied_retry()
        .expect("total request lifetime timeout should deny timeout-triggered retry");

    assert_eq!(
        denied.class(),
        ResourceRetryDenialClass::RetryTimeoutWindowExhausted
    );
    assert_eq!(
        retry_schedule_report
            .performance()
            .temporal_wake_footprint(),
        0
    );
    assert_eq!(
        runtime
            .resource_descriptor_for_node(ResourceNodeId::from_node(node))
            .expect("descriptor should exist")
            .timeout_decision_plan()
            .class(),
        ResourceTimeoutDecisionClass::TotalRequestLifetimeTimeout
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_retry_timeout_window_exhaustion_denial_count,
        1
    );
}

#[test]
fn resource_timeout_admission_denies_wrong_ready_wake_without_mutation() {
    let mut graph = SignalGraph::new();
    let first_node = graph.node().build();
    let second_node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(timeout_resource_declaration(first_node, 5))
        .expect("first declaration should lower");
    runtime
        .declare_resource_node(timeout_resource_declaration(second_node, 5))
        .expect("second declaration should lower");
    let first = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(
            first_node,
        )))
        .expect("first request should admit")
        .admitted_request()
        .handle();
    let second = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(
            second_node,
        )))
        .expect("second request should admit")
        .admitted_request()
        .handle();
    let second_wake = runtime
        .in_flight_resource_request(second)
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("second timeout wake should be attached");

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(5),
        ))
        .expect("authoritative clock should advance");
    let wrong_ready = runtime
        .promote_temporal_wake_ready(second_wake)
        .expect("second wake should promote");
    let report = runtime
        .admit_resource_timeout(first, wrong_ready)
        .expect("wrong wake denial should not trip temporal cleanup");

    let denied = report
        .denied_timeout()
        .expect("wrong ready wake should be denied");
    assert_eq!(denied.class(), ResourceTimeoutDenialClass::WakeMismatch);
    assert!(report.timed_out_request().is_none());
    assert_eq!(
        runtime
            .in_flight_resource_request(first)
            .expect("first request should remain active")
            .status(),
        ResourceInFlightStatus::Active
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_timeout_wake_mismatch_denial_count,
        1
    );
}

#[test]
fn resource_timeout_heartbeat_extension_reschedules_active_timeout_wake() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(heartbeat_extension_timeout_resource_declaration(node, 5, 7))
        .expect("heartbeat timeout declaration should lower");
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let first_wake = runtime
        .in_flight_resource_request(admitted.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("initial timeout wake should exist");

    let report = runtime
        .extend_resource_timeout_heartbeat(admitted.handle())
        .expect("heartbeat extension should return report");
    let extended = report
        .extended_heartbeat()
        .expect("active request should admit heartbeat extension");

    assert_eq!(extended.previous_timeout_wake_id(), first_wake);
    assert_eq!(extended.extension_duration().get(), 7);
    assert_eq!(
        extended.extended_timeout_wake().due_tick(),
        ClockTick::new(runtime.clock_basis().current_tick().get().saturating_add(7))
    );
    assert_ne!(extended.extended_timeout_wake().id(), first_wake);
    assert_eq!(
        runtime
            .in_flight_resource_request(admitted.handle())
            .and_then(|in_flight| in_flight.timeout_wake_id()),
        Some(extended.extended_timeout_wake().id())
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_progress_heartbeat_extension_count,
        1
    );
}

#[test]
fn resource_timeout_heartbeat_extension_denies_terminal_request() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(heartbeat_extension_timeout_resource_declaration(node, 5, 7))
        .expect("heartbeat timeout declaration should lower");
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let wake_id = runtime
        .in_flight_resource_request(admitted.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("timeout wake should attach");
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(5),
        ))
        .expect("clock should reach timeout");
    let ready = runtime
        .promote_temporal_wake_ready(wake_id)
        .expect("timeout wake should become ready");
    runtime
        .admit_resource_timeout(admitted.handle(), ready)
        .expect("timeout admission should succeed");

    let report = runtime
        .extend_resource_timeout_heartbeat(admitted.handle())
        .expect("heartbeat extension denial should still return report");
    let denied = report
        .denied_extension()
        .expect("timed out request should deny heartbeat extension");

    assert_eq!(
        denied.class(),
        ResourceTimeoutHeartbeatExtensionDenialClass::NonActiveRequest
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_timeout_heartbeat_extension_denial_count,
        1
    );
}
