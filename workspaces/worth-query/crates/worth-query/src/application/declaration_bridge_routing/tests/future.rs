use super::support::{
    domain::{
        admitted_handle, AsyncRuntimeRouteFamily, AsyncSignalOnlyFamily, RoutingInput,
        RuntimeRouteFamily, TemporalRuntimeRouteFamily, TemporalSignalOnlyFamily,
    },
    proof::{
        checked_from_future_supported_runtime_test_posture,
        routed_from_future_supported_runtime_test_posture,
    },
};

#[test]
fn ordinary_temporal_and_async_bridge_routes_share_one_checked_routing_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "future-bridge");

    let ordinary = super::support::proof::routed_from_progressed(
        &handle,
        RoutingInput::<RuntimeRouteFamily>::new("edge:42"),
    );
    let temporal = routed_from_future_supported_runtime_test_posture(
        &handle,
        RoutingInput::<TemporalRuntimeRouteFamily>::new("edge:42"),
    );
    let async_route = routed_from_future_supported_runtime_test_posture(
        &handle,
        RoutingInput::<AsyncRuntimeRouteFamily>::new("edge:42"),
    );

    assert_eq!(ordinary.future_projection().class().as_str(), "ordinary");
    assert_eq!(temporal.future_projection().class().as_str(), "temporal");
    assert_eq!(
        async_route.future_projection().class().as_str(),
        "async_resource"
    );
    assert_eq!(ordinary.binding().surface(), temporal.binding().surface());
    assert_eq!(
        temporal.binding().surface(),
        async_route.binding().surface()
    );
}

#[test]
fn future_projection_and_basis_support_participate_in_bridge_routing_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "future-bridge-digest");

    let temporal = routed_from_future_supported_runtime_test_posture(
        &handle,
        RoutingInput::<TemporalRuntimeRouteFamily>::new("edge:42"),
    );
    let async_route = routed_from_future_supported_runtime_test_posture(
        &handle,
        RoutingInput::<AsyncRuntimeRouteFamily>::new("edge:42"),
    );

    assert_ne!(
        temporal.future_projection().projection_digest(),
        async_route.future_projection().projection_digest()
    );
    assert_eq!(
        temporal.basis_lifecycle_support_digest(),
        async_route.basis_lifecycle_support_digest()
    );
    assert_ne!(
        temporal.bridge_routing_digest(),
        async_route.bridge_routing_digest()
    );
}

#[test]
fn future_projection_survives_route_plan_to_bridge_routing_without_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "future-bridge-stitch");

    let temporal = routed_from_future_supported_runtime_test_posture(
        &handle,
        RoutingInput::<TemporalRuntimeRouteFamily>::new("edge:42"),
    );
    let route_plan = temporal
        .envelope()
        .route_plan()
        .expect("routed bridge artifact should retain its route plan");

    assert_eq!(
        route_plan.future_projection().projection_digest(),
        temporal.future_projection().projection_digest()
    );
    assert_eq!(
        route_plan
            .progressed_declaration()
            .retained_world_basis()
            .basis_lifecycle_support_for_reporting(),
        temporal.basis_lifecycle_support_digest()
    );
}

#[test]
fn temporal_and_async_signal_only_declarations_deny_before_continuation_preparation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "future-signal-only");

    match checked_from_future_supported_runtime_test_posture(
        &handle,
        RoutingInput::<TemporalSignalOnlyFamily>::new("edge:42"),
    ) {
        crate::application::WorthQueryDeclarationBridgeRoutingChecked::Denied(routing) => {
            assert_eq!(
                routing.cause(),
                crate::application::WorthQueryDeclarationBridgeRoutingDenialCause::EnvelopeNotCoveredForBridgeRouting
            );
        }
        _ => panic!("future-bearing signal-only declarations should deny bridge continuation"),
    }

    match checked_from_future_supported_runtime_test_posture(
        &handle,
        RoutingInput::<AsyncSignalOnlyFamily>::new("edge:42"),
    ) {
        crate::application::WorthQueryDeclarationBridgeRoutingChecked::Denied(routing) => {
            assert_eq!(
                routing.cause(),
                crate::application::WorthQueryDeclarationBridgeRoutingDenialCause::EnvelopeNotCoveredForBridgeRouting
            );
        }
        _ => panic!("future-bearing signal-only declarations should deny bridge continuation"),
    }
}
