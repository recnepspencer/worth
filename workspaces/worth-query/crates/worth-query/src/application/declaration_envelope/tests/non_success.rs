use crate::application::{
    WorthQueryDeclarationEnvelopeChecked, WorthQueryDeclarationEnvelopeInput,
    WorthQueryDeclarationReceiptDenialCause, WorthQueryDeclarationRoutePlanDenialCause,
};

use super::support::{
    admitted_handle, route_checked_from_input, route_checked_with_intent, DeferredEnvelopeFamily,
    EnvelopeInput, FailedEnvelopeFamily, RequiredIntentEnvelopeFamily, SignalEnvelopeFamily,
};

#[test]
fn deferred_receipts_become_deferred_envelopes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.envelope_routes_checked(WorthQueryDeclarationEnvelopeInput::receipt_checked(
        handle.receipt_routes_checked(
            crate::application::WorthQueryDeclarationReceiptInput::route_checked(
                route_checked_with_intent(
                    &handle,
                    EnvelopeInput::<DeferredEnvelopeFamily>::new("edge:42"),
                    crate::application::WorthQueryDeclarationRouteIntent::DeferredRouting,
                ),
            ),
        ),
    )) {
        WorthQueryDeclarationEnvelopeChecked::Deferred(envelope) => {
            assert!(envelope
                .envelope()
                .explain()
                .crossing_posture()
                .contains("deferred"));
        }
        _ => panic!("deferred receipts should become deferred envelopes"),
    }
}

#[test]
fn denied_envelopes_preserve_route_and_receipt_denial_layers() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.envelope_routes_checked(WorthQueryDeclarationEnvelopeInput::receipt_checked(
        handle.receipt_routes_checked(
            crate::application::WorthQueryDeclarationReceiptInput::route_checked(
                route_checked_from_input(
                    &handle,
                    EnvelopeInput::<RequiredIntentEnvelopeFamily>::new("edge:42"),
                ),
            ),
        ),
    )) {
        WorthQueryDeclarationEnvelopeChecked::Denied(envelope) => {
            assert_eq!(
                envelope.route_cause(),
                Some(WorthQueryDeclarationRoutePlanDenialCause::IntentRequired)
            );
            assert_eq!(envelope.receipt_cause(), None);
        }
        _ => panic!("required intent denial should envelope as denied"),
    }
}

#[test]
fn failed_receipts_remain_distinct_from_denied_envelopes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.envelope_routes_checked(WorthQueryDeclarationEnvelopeInput::receipt_checked(
        handle.receipt_routes_checked(
            crate::application::WorthQueryDeclarationReceiptInput::route_checked(
                route_checked_with_intent(
                    &handle,
                    EnvelopeInput::<FailedEnvelopeFamily>::new("edge:42"),
                    crate::application::WorthQueryDeclarationRouteIntent::Auto,
                ),
            ),
        ),
    )) {
        WorthQueryDeclarationEnvelopeChecked::Failed(envelope) => {
            assert!(envelope
                .envelope()
                .explain()
                .crossing_posture()
                .contains("failed"));
        }
        _ => panic!("failed receipts should remain failed envelopes"),
    }
}

#[test]
fn unsupported_successful_receipt_kinds_preserve_receipt_boundary_denial() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.envelope_routes_checked(WorthQueryDeclarationEnvelopeInput::receipt_checked(
        handle.receipt_routes_checked(
            crate::application::WorthQueryDeclarationReceiptInput::route_checked(
                route_checked_with_intent(
                    &handle,
                    EnvelopeInput::<SignalEnvelopeFamily>::new("edge:42"),
                    crate::application::WorthQueryDeclarationRouteIntent::Auto,
                ),
            ),
        ),
    )) {
        WorthQueryDeclarationEnvelopeChecked::Denied(envelope) => {
            assert_eq!(
                envelope.receipt_cause(),
                Some(WorthQueryDeclarationReceiptDenialCause::UnsupportedReceiptKind)
            );
            assert!(envelope
                .envelope()
                .explain()
                .route_reference()
                .expect("unsupported planned route should retain context")
                .contains("planned-route:signal"));
        }
        _ => panic!("unsupported receipt kinds should become denied envelopes"),
    }
}

#[test]
fn distinct_denial_topology_changes_envelope_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    let route_denied = match handle.envelope_routes_checked(
        WorthQueryDeclarationEnvelopeInput::receipt_checked(handle.receipt_routes_checked(
            crate::application::WorthQueryDeclarationReceiptInput::route_checked(
                route_checked_from_input(
                    &handle,
                    EnvelopeInput::<RequiredIntentEnvelopeFamily>::new("edge:42"),
                ),
            ),
        )),
    ) {
        WorthQueryDeclarationEnvelopeChecked::Denied(envelope) => envelope,
        _ => panic!("route denial should produce denied envelope"),
    };

    let receipt_denied = match handle.envelope_routes_checked(
        WorthQueryDeclarationEnvelopeInput::receipt_checked(handle.receipt_routes_checked(
            crate::application::WorthQueryDeclarationReceiptInput::route_checked(
                route_checked_with_intent(
                    &handle,
                    EnvelopeInput::<SignalEnvelopeFamily>::new("edge:42"),
                    crate::application::WorthQueryDeclarationRouteIntent::Auto,
                ),
            ),
        )),
    ) {
        WorthQueryDeclarationEnvelopeChecked::Denied(envelope) => envelope,
        _ => panic!("receipt denial should produce denied envelope"),
    };

    assert_ne!(
        route_denied.envelope().envelope_digest(),
        receipt_denied.envelope().envelope_digest()
    );
}
