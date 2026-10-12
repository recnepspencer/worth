use crate::application::{
    WorthQueryDeclarationBridgeBinding, WorthQueryDeclarationBridgeContinuationFamily,
    WorthQueryDeclarationBridgeRoutingChecked, WorthQueryDeclarationBridgeRoutingClass,
};

use super::support::{
    domain::{
        admitted_handle, MixedAuthorityFamily, PreviewPromotionFamily, PreviewSessionFamily,
        RoutingInput, RuntimeRouteFamily, TruthViewCurrentFamily, TruthViewHistoricalFamily,
    },
    proof::{checked_from_progressed, routed_from_progressed},
};

#[test]
fn common_lane_routes_bridge_continuation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "common");

    let routing = handle
        .declare_review_progress_describe_plan_receipt_envelope_and_route_bridge_continuation(
            RoutingInput::<PreviewSessionFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("common bridge lane should admit"));

    assert_eq!(
        routing.continuation_family(),
        WorthQueryDeclarationBridgeContinuationFamily::PreviewSession
    );
    assert_eq!(
        routing.continuation_request().truth_context(),
        crate::application::WorthQueryDeclarationBridgeTruthContext::Preview
    );
    assert_eq!(
        routing.binding().surface(),
        "worth_runtime_bridge::facade::BridgeSpeculativeSessionRequest"
    );
}

#[test]
fn equivalent_retained_truth_yields_identical_bridge_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "digest");

    let left = handle
        .route_bridge_continuation_from_progressed(
            handle
                .declare_review_and_progress(RoutingInput::<RuntimeRouteFamily>::new("edge:42"))
                .unwrap_or_else(|_| panic!("progression should admit")),
        )
        .unwrap_or_else(|_| panic!("bridge routing should admit"));
    let right = handle
        .route_bridge_continuation_from_progressed(
            handle
                .declare_review_and_progress(RoutingInput::<RuntimeRouteFamily>::new("edge:42"))
                .unwrap_or_else(|_| panic!("progression should admit")),
        )
        .unwrap_or_else(|_| panic!("bridge routing should admit"));

    assert_eq!(left.bridge_routing_digest(), right.bridge_routing_digest());
}

#[test]
fn mixed_authority_families_keep_common_bridge_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "mixed");

    let routing = handle
        .declare_review_progress_describe_plan_receipt_envelope_and_route_bridge_continuation(
            RoutingInput::<MixedAuthorityFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("mixed authority bridge lane should admit"));

    assert_eq!(
        routing.class(),
        WorthQueryDeclarationBridgeRoutingClass::MixedAuthorityBridgeContinuation
    );
}

#[test]
fn bridge_bindings_remain_distinct_by_family() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "binding");

    let historical = handle
        .declare_review_progress_describe_plan_receipt_envelope_and_route_bridge_continuation(
            RoutingInput::<TruthViewHistoricalFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("historical truth view should admit"));
    let promotion = handle
        .declare_review_progress_describe_plan_receipt_envelope_and_route_bridge_continuation(
            RoutingInput::<PreviewPromotionFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("preview promotion should admit"));

    assert_ne!(
        historical.bridge_routing_digest(),
        promotion.bridge_routing_digest()
    );
    assert!(matches!(
        historical.binding(),
        WorthQueryDeclarationBridgeBinding::TruthView(_)
    ));
    assert!(matches!(
        promotion.binding(),
        WorthQueryDeclarationBridgeBinding::PreviewPromotion(_)
    ));
}

#[test]
fn truth_context_sensitive_truth_view_routes_diverge_in_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "truth-context");

    let current = handle
        .declare_review_progress_describe_plan_receipt_envelope_and_route_bridge_continuation(
            RoutingInput::<TruthViewCurrentFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("current truth view should admit"));
    let historical = handle
        .declare_review_progress_describe_plan_receipt_envelope_and_route_bridge_continuation(
            RoutingInput::<TruthViewHistoricalFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("historical truth view should admit"));

    assert_ne!(
        current.bridge_routing_digest(),
        historical.bridge_routing_digest()
    );
    assert_eq!(
        current.truth_context(),
        crate::application::WorthQueryDeclarationBridgeTruthContext::Current
    );
    assert_eq!(
        historical.truth_context(),
        crate::application::WorthQueryDeclarationBridgeTruthContext::Historical
    );
}

#[test]
fn advanced_lane_envelope_checked_input_routes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "advanced");

    match checked_from_progressed(
        &handle,
        RoutingInput::<PreviewSessionFamily>::new("edge:42"),
    ) {
        WorthQueryDeclarationBridgeRoutingChecked::Routed(routing) => {
            assert_eq!(
                routing.continuation_family(),
                WorthQueryDeclarationBridgeContinuationFamily::PreviewSession
            );
        }
        _ => panic!("advanced bridge lane should route"),
    }
}

#[test]
fn advanced_lane_envelope_input_routes_without_checked_wrapper_loss() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "advanced-direct");

    let routing = routed_from_progressed(
        &handle,
        RoutingInput::<PreviewSessionFamily>::new("edge:42"),
    );

    assert_eq!(
        routing.continuation_family(),
        WorthQueryDeclarationBridgeContinuationFamily::PreviewSession
    );
    assert_eq!(
        routing.binding().surface(),
        "worth_runtime_bridge::facade::BridgeSpeculativeSessionRequest"
    );
}
