use super::domain::{
    admitted_handle, future_supported_route_input, AsyncBridgeRouteFamily, RelationalRouteFamily,
    RouteInput, TemporalBridgeRouteFamily,
};

#[test]
fn ordinary_temporal_and_async_declarations_plan_through_the_same_public_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "future-route");

    let ordinary = handle
        .plan_routes(future_supported_route_input(
            &handle,
            RouteInput::<RelationalRouteFamily>::new("edge:42"),
        ))
        .unwrap_or_else(|_| panic!("ordinary route plan should admit"));
    let temporal = handle
        .plan_routes(future_supported_route_input(
            &handle,
            RouteInput::<TemporalBridgeRouteFamily>::new("edge:42"),
        ))
        .unwrap_or_else(|_| panic!("temporal route plan should admit"));
    let async_route = handle
        .plan_routes(future_supported_route_input(
            &handle,
            RouteInput::<AsyncBridgeRouteFamily>::new("edge:42"),
        ))
        .unwrap_or_else(|_| panic!("async route plan should admit"));

    assert_eq!(ordinary.future_projection().class().as_str(), "ordinary");
    assert_eq!(temporal.future_projection().class().as_str(), "temporal");
    assert_eq!(
        async_route.future_projection().class().as_str(),
        "async_resource"
    );
    assert_eq!(ordinary.route_count(), 1);
    assert_eq!(temporal.route_count(), 1);
    assert_eq!(async_route.route_count(), 1);
    assert_eq!(temporal.route_families(), async_route.route_families());
}

#[test]
fn future_projection_digest_participates_in_route_plan_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "future-digest");

    let temporal = handle
        .plan_routes(future_supported_route_input(
            &handle,
            RouteInput::<TemporalBridgeRouteFamily>::new("edge:42"),
        ))
        .unwrap_or_else(|_| panic!("temporal route plan should admit"));
    let async_route = handle
        .plan_routes(future_supported_route_input(
            &handle,
            RouteInput::<AsyncBridgeRouteFamily>::new("edge:42"),
        ))
        .unwrap_or_else(|_| panic!("async route plan should admit"));

    assert_ne!(
        temporal.future_projection().projection_digest(),
        async_route.future_projection().projection_digest()
    );
    assert_ne!(
        temporal.route_plan_digest(),
        async_route.route_plan_digest()
    );
    assert!(temporal
        .explain()
        .retained_facts()
        .iter()
        .any(|reason: &String| reason.contains("future-projection-class:temporal")));
    assert!(async_route
        .explain()
        .retained_facts()
        .iter()
        .any(|reason: &String| reason.contains("future-projection-class:async_resource")));
}
