use super::*;

#[test]
fn wrong_ready_wake_denial_preserves_the_scheduled_retry() {
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
        .and_then(|request| request.timeout_wake_id())
        .expect("timed request should own a timeout wake");

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(3),
        ))
        .expect("clock should reach timeout");
    let ready_timeout = runtime
        .promote_temporal_wake_ready(timeout_wake)
        .expect("timeout wake should become ready");
    runtime
        .admit_resource_timeout(admitted.handle(), ready_timeout)
        .expect("matching timeout wake should admit");
    let schedule_report = runtime
        .schedule_resource_retry(admitted.handle(), ResourceRetryReason::TimedOut)
        .expect("retry scheduling should use the installed policy");
    let scheduled = schedule_report
        .scheduled_retry()
        .expect("timed-out request should schedule a retry");
    let unrelated = runtime
        .schedule_temporal_wake(
            TemporalCondition::after(7).expect("positive delay is valid"),
            ClockTick::new(10),
        )
        .expect("unrelated temporal wake should schedule");

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(10),
        ))
        .expect("clock should reach the retry deadline");
    let ready_retry = runtime
        .promote_temporal_wake_ready(scheduled.backoff_wake_id())
        .expect("scheduled retry wake should become ready");
    let wrong_ready = runtime
        .promote_temporal_wake_ready(unrelated.id())
        .expect("unrelated wake should become ready");

    let denied = runtime
        .admit_scheduled_resource_retry(admitted.handle(), wrong_ready)
        .expect("wrong-wake denial should remain report-shaped");
    assert_eq!(
        denied
            .denied_retry()
            .expect("unrelated ready wake must not admit retry")
            .class(),
        ResourceRetryDenialClass::WakeMismatch
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_retry_wake_mismatch_denial_count,
        1
    );
    assert_eq!(
        runtime.telemetry().resource.resource_retry_admission_count,
        0
    );

    let admitted_retry = runtime
        .admit_scheduled_resource_retry(admitted.handle(), ready_retry)
        .expect("the original ready wake should remain consumable");
    assert!(
        admitted_retry.admitted_retry().is_some(),
        "matching scheduled authority should still admit after mismatch denial"
    );
    assert_eq!(
        runtime.telemetry().resource.resource_retry_admission_count,
        1
    );
}
