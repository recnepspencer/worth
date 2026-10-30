use super::*;

#[test]
fn resource_timeout_revalidation_eligible_classification_is_retained_in_timeout_artifact() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(revalidation_eligible_timeout_resource_declaration(node, 5))
        .expect("revalidation eligible timeout declaration should lower");
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
    let report = runtime
        .admit_resource_timeout(admitted.handle(), ready)
        .expect("timeout admission should succeed");
    let timed_out = report
        .timed_out_request()
        .expect("revalidation-eligible timeout should still admit timeout");

    assert_eq!(
        timed_out.outcome_class(),
        ResourceTimeoutOutcomeClass::RevalidationEligible
    );
    assert_eq!(
        runtime
            .resource_descriptor_for_node(ResourceNodeId::from_node(node))
            .expect("descriptor should exist")
            .timeout_decision_plan()
            .class(),
        ResourceTimeoutDecisionClass::RevalidationEligibleTimeout
    );
}
