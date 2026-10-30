use super::*;

#[test]
fn resource_supersession_retires_prior_timeout_wake_before_it_can_drive_timeout() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(timeout_resource_declaration(node, 5))
        .expect("resource declaration should lower");
    let first = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("first request should admit")
        .admitted_request()
        .handle();
    let first_wake = runtime
        .in_flight_resource_request(first)
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("first timeout wake should be attached");

    let second = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("second request should supersede first");

    let supersession = second
        .supersession_record()
        .expect("supersession should be explicit");
    assert_eq!(supersession.previous(), first);
    assert_eq!(supersession.replacing(), second.admitted_request().handle());
    assert!(
        supersession
            .policy_decision_digest()
            .as_str()
            .starts_with("resource-policy-supersession-plan:"),
        "supersession record should retain lowered supersession decision proof"
    );
    assert_eq!(runtime.temporal_wake_summary().retired_count(), 1);
    assert_eq!(runtime.temporal_wake_summary().scheduled_count(), 1);
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_supersession_policy_decision_count,
        1
    );

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(5),
        ))
        .expect("authoritative clock should advance");
    let err = runtime
        .promote_temporal_wake_ready(first_wake)
        .expect_err("superseded timeout wake must not become ready truth");
    assert!(!err.to_string().is_empty());
    assert_eq!(
        runtime
            .in_flight_resource_request(first)
            .expect("first request should remain retained as superseded")
            .superseded_by(),
        Some(second.admitted_request().handle())
    );
}

#[test]
fn resource_retry_after_timeout_preserves_attempt_lineage_and_backoff_wake_truth() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(retry_timeout_resource_declaration(node, 3, 7))
        .expect("resource declaration should lower");
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let timeout_wake = runtime
        .in_flight_resource_request(admitted.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("timeout wake should be attached");

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(3),
        ))
        .expect("clock should reach timeout");
    let ready_timeout = runtime
        .promote_temporal_wake_ready(timeout_wake)
        .expect("timeout wake should be ready");
    runtime
        .admit_resource_timeout(admitted.handle(), ready_timeout)
        .expect("timeout admission should consume temporal wake")
        .timed_out_request()
        .expect("timeout should admit");

    let schedule = runtime
        .schedule_resource_retry(admitted.handle(), ResourceRetryReason::TimedOut)
        .expect("retry scheduling should use runtime backoff");
    let scheduled = schedule
        .scheduled_retry()
        .expect("timed-out request with retry policy should schedule retry");
    let retry_plan = runtime
        .resource_descriptor_for_node(ResourceNodeId::from_node(node))
        .expect("retry descriptor should exist")
        .retry_decision_plan()
        .clone();
    assert_eq!(scheduled.previous(), admitted.handle());
    assert_eq!(scheduled.next_attempt(), ResourceAttemptId::new(1));
    assert_eq!(scheduled.scheduled_delay().get(), 7);
    assert_eq!(
        scheduled.policy_decision_digest().as_str(),
        retry_plan.decision_digest().as_str()
    );
    assert_eq!(
        schedule.performance().boundary(),
        ResourceBoundaryKind::RetrySchedule
    );
    assert_eq!(schedule.performance().temporal_wake_footprint(), 1);

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(10),
        ))
        .expect("clock should reach retry backoff");
    let ready_retry = runtime
        .promote_temporal_wake_ready(scheduled.backoff_wake_id())
        .expect("retry backoff wake should become ready");
    let report = runtime
        .admit_scheduled_resource_retry(admitted.handle(), ready_retry)
        .expect("retry admission should consume backoff wake");
    let retry = report
        .admitted_retry()
        .expect("matching backoff wake should admit retry");
    let retry_request = retry.admitted_request();

    assert_eq!(retry.scheduled().previous(), admitted.handle());
    assert_eq!(retry_request.attempt(), ResourceAttemptId::new(1));
    assert_eq!(
        retry_request.handle().generation(),
        admitted.handle().generation()
    );
    assert_eq!(
        report.performance().boundary(),
        ResourceBoundaryKind::RetryAdmission
    );
    assert_eq!(report.performance().temporal_wake_footprint(), 2);
    assert_eq!(
        runtime
            .in_flight_resource_request(retry_request.handle())
            .expect("retry request should be retained")
            .status(),
        ResourceInFlightStatus::Active
    );
    assert_eq!(
        runtime.telemetry().resource.resource_retry_schedule_count,
        1
    );
    assert_eq!(
        runtime.telemetry().resource.resource_retry_admission_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_retry_temporal_wake_footprint,
        2
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_timeout_temporal_wake_footprint,
        3
    );
}

#[test]
fn resource_retry_schedule_denies_disabled_policy_without_temporal_wake() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(timeout_resource_declaration(node, 3))
        .expect("resource declaration should lower");
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let timeout_wake = runtime
        .in_flight_resource_request(admitted.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("timeout wake should be attached");

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(3),
        ))
        .expect("clock should reach timeout");
    let ready_timeout = runtime
        .promote_temporal_wake_ready(timeout_wake)
        .expect("timeout wake should be ready");
    runtime
        .admit_resource_timeout(admitted.handle(), ready_timeout)
        .expect("timeout admission should consume temporal wake");

    let report = runtime
        .schedule_resource_retry(admitted.handle(), ResourceRetryReason::TimedOut)
        .expect("disabled policy denial should stay report-shaped");
    let denied = report
        .denied_retry()
        .expect("retry policy disabled should deny retry scheduling");

    assert_eq!(
        denied.class(),
        ResourceRetryDenialClass::RetryPolicyDisabled
    );
    assert_eq!(report.performance().temporal_wake_footprint(), 0);
    assert_eq!(
        runtime.telemetry().resource.resource_retry_schedule_count,
        0
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_retry_policy_disabled_denial_count,
        1
    );
}

#[test]
fn resource_retry_schedule_denies_duplicate_without_allocating_second_wake() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(retry_timeout_resource_declaration(node, 3, 7))
        .expect("resource declaration should lower");
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let timeout_wake = runtime
        .in_flight_resource_request(admitted.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("timeout wake should be attached");

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(3),
        ))
        .expect("clock should reach timeout");
    let ready_timeout = runtime
        .promote_temporal_wake_ready(timeout_wake)
        .expect("timeout wake should be ready");
    runtime
        .admit_resource_timeout(admitted.handle(), ready_timeout)
        .expect("timeout admission should consume temporal wake");

    let first = runtime
        .schedule_resource_retry(admitted.handle(), ResourceRetryReason::TimedOut)
        .expect("first retry scheduling should admit");
    let scheduled = first
        .scheduled_retry()
        .expect("first retry should carry a pending backoff wake");
    let second = runtime
        .schedule_resource_retry(admitted.handle(), ResourceRetryReason::TimedOut)
        .expect("duplicate retry scheduling should stay report-shaped");
    let denied = second
        .denied_retry()
        .expect("duplicate retry should be denied before temporal allocation");

    assert_eq!(
        denied.class(),
        ResourceRetryDenialClass::RetryAlreadyScheduled
    );
    assert_eq!(second.performance().temporal_wake_footprint(), 0);
    assert_eq!(runtime.temporal_wake_summary().scheduled_count(), 1);
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(10),
        ))
        .expect("clock should reach original retry backoff");
    assert_eq!(
        runtime
            .promote_temporal_wake_ready(scheduled.backoff_wake_id())
            .expect("original retry wake should remain the only schedulable wake")
            .id(),
        scheduled.backoff_wake_id()
    );
    assert_eq!(
        runtime.telemetry().resource.resource_retry_schedule_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_retry_already_scheduled_denial_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_retry_temporal_wake_footprint,
        1
    );
}

#[test]
fn resource_retry_admission_denies_if_newer_request_wins_before_backoff_ready() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(retry_timeout_resource_declaration(node, 3, 7))
        .expect("resource declaration should lower");
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let timeout_wake = runtime
        .in_flight_resource_request(admitted.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("timeout wake should be attached");
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(3),
        ))
        .expect("clock should reach timeout");
    let ready_timeout = runtime
        .promote_temporal_wake_ready(timeout_wake)
        .expect("timeout wake should be ready");
    runtime
        .admit_resource_timeout(admitted.handle(), ready_timeout)
        .expect("timeout admission should consume temporal wake");
    let schedule = runtime
        .schedule_resource_retry(admitted.handle(), ResourceRetryReason::TimedOut)
        .expect("retry scheduling should use runtime backoff");
    let scheduled = schedule
        .scheduled_retry()
        .expect("retry should be scheduled");

    runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("fresh admission should win before retry backoff fires");
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(10),
        ))
        .expect("clock should reach retry backoff");
    let promote_err = runtime
        .promote_temporal_wake_ready(scheduled.backoff_wake_id())
        .expect_err("superseded retry backoff wake must be retired before promotion");
    assert!(
        promote_err
            .to_string()
            .contains("cannot promote unknown scheduled temporal wake"),
        "unexpected promotion error: {promote_err}"
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_retry_superseded_denial_count,
        0
    );
}
