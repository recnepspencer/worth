use crate::application::{
    WorthQueryDeclarationBridgeRoutingChecked, WorthQueryDeclarationBridgeRoutingDenialCause,
    WorthQueryDeclarationEnvelopeInput,
};

use super::support::{
    domain::{admitted_handle, RoutingInput, RuntimeRouteFamily, SignalOnlyFamily},
    proof::checked_from_progressed,
};

#[test]
fn explicit_routing_rejects_envelopes_from_the_wrong_handle() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first = admitted_handle(request_execution, "alpha");
    let second = admitted_handle(request_execution, "beta");

    let progressed = first
        .declare_review_and_progress(RoutingInput::<RuntimeRouteFamily>::new("edge:42"))
        .unwrap_or_else(|_| panic!("progression should admit"));
    let envelope = first
        .envelope_routes_from_progressed(progressed)
        .unwrap_or_else(|_| panic!("envelope should admit"));

    match second.route_bridge_continuation_checked(
        crate::application::WorthQueryDeclarationBridgeRoutingInput::enveloped(envelope),
    ) {
        WorthQueryDeclarationBridgeRoutingChecked::Denied(routing) => {
            assert_eq!(
                routing.cause(),
                WorthQueryDeclarationBridgeRoutingDenialCause::BridgeEnvelopeMismatch
            );
        }
        _ => panic!("wrong-handle bridge routing should deny"),
    }
}

#[test]
fn signal_only_declarations_do_not_reach_bridge_continuation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "signal-only");

    match checked_from_progressed(&handle, RoutingInput::<SignalOnlyFamily>::new("edge:42")) {
        WorthQueryDeclarationBridgeRoutingChecked::Denied(routing) => {
            assert_eq!(
                routing.cause(),
                WorthQueryDeclarationBridgeRoutingDenialCause::EnvelopeNotCoveredForBridgeRouting
            );
        }
        _ => panic!("signal-only route plans should deny bridge continuation"),
    }
}

#[test]
fn denied_receipt_envelopes_stay_denied_for_bridge_routing() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "denied");

    let route_checked = handle.plan_routes_checked(
        crate::application::WorthQueryDeclarationRoutePlanInput::with_intent(
            handle
                .declare_review_and_progress(RoutingInput::<RuntimeRouteFamily>::new("edge:42"))
                .unwrap_or_else(|_| panic!("progression should admit")),
            handle
                .describe_foundational(
                    crate::application::WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(
                        handle
                            .declare_review_and_progress(RoutingInput::<RuntimeRouteFamily>::new("edge:42"))
                            .unwrap_or_else(|_| panic!("progression should admit")),
                    ),
                )
                .unwrap_or_else(|_| panic!("evidence should materialize")),
            crate::application::WorthQueryDeclarationRouteIntent::DeferredRouting,
        ),
    );
    let receipt_checked = handle.receipt_routes_checked(
        crate::application::WorthQueryDeclarationReceiptInput::route_checked(route_checked),
    );
    let envelope_checked = handle.envelope_routes_checked(
        WorthQueryDeclarationEnvelopeInput::receipt_checked(receipt_checked),
    );

    match handle.route_bridge_continuation_checked(
        crate::application::WorthQueryDeclarationBridgeRoutingInput::envelope_checked(
            envelope_checked,
        ),
    ) {
        WorthQueryDeclarationBridgeRoutingChecked::Denied(_)
        | WorthQueryDeclarationBridgeRoutingChecked::Deferred(_) => {}
        _ => panic!("non-success envelope input should not route"),
    }
}
