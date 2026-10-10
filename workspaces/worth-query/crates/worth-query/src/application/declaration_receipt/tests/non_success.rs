use crate::application::{
    WorthQueryDeclarationReceiptChecked, WorthQueryDeclarationReceiptDenialCause,
    WorthQueryDeclarationReceiptInput, WorthQueryDeclarationReceiptKind,
    WorthQueryDeclarationRoutePlanDenialCause,
};

use super::support::{
    admitted_handle, route_checked_from_input, route_checked_with_intent, DeferredReceiptFamily,
    FailedReceiptFamily, ForbiddenIntentReceiptFamily, ReceiptInput, RequiredIntentReceiptFamily,
    SignalReceiptFamily,
};

#[test]
fn denied_route_receipts_preserve_typed_phase_nine_cause() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.receipt_routes_checked(WorthQueryDeclarationReceiptInput::route_checked(
        route_checked_from_input(
            &handle,
            ReceiptInput::<RequiredIntentReceiptFamily>::new("edge:42"),
        ),
    )) {
        WorthQueryDeclarationReceiptChecked::Denied(denial) => {
            assert_eq!(
                denial.route_cause(),
                Some(WorthQueryDeclarationRoutePlanDenialCause::IntentRequired)
            );
            assert_eq!(
                denial.receipt().route_denial_cause(),
                Some(WorthQueryDeclarationRoutePlanDenialCause::IntentRequired)
            );
        }
        _ => panic!("missing route intent should produce a denied receipt"),
    }
}

#[test]
fn deferred_route_receipts_do_not_disappear() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.receipt_routes_checked(WorthQueryDeclarationReceiptInput::route_checked(
        route_checked_from_input(
            &handle,
            ReceiptInput::<DeferredReceiptFamily>::new("edge:42"),
        ),
    )) {
        WorthQueryDeclarationReceiptChecked::Deferred(receipt) => {
            assert_eq!(
                receipt.receipt().kind(),
                WorthQueryDeclarationReceiptKind::Deferred
            );
        }
        _ => panic!("deferred route plans should produce deferred receipts"),
    }
}

#[test]
fn failed_route_receipts_remain_distinct_from_denied_receipts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.receipt_routes_checked(WorthQueryDeclarationReceiptInput::route_checked(
        route_checked_from_input(&handle, ReceiptInput::<FailedReceiptFamily>::new("edge:42")),
    )) {
        WorthQueryDeclarationReceiptChecked::Failed(receipt) => {
            assert_eq!(
                receipt.receipt().kind(),
                WorthQueryDeclarationReceiptKind::Failed
            );
        }
        _ => panic!("failed route plans should produce failed receipts"),
    }
}

#[test]
fn unsupported_signal_receipts_become_typed_denials() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.receipt_routes_checked(WorthQueryDeclarationReceiptInput::route_checked(
        route_checked_from_input(&handle, ReceiptInput::<SignalReceiptFamily>::new("edge:42")),
    )) {
        WorthQueryDeclarationReceiptChecked::Denied(denial) => {
            assert_eq!(
                denial.receipt_cause(),
                Some(WorthQueryDeclarationReceiptDenialCause::UnsupportedReceiptKind)
            );
            assert!(denial
                .receipt()
                .explain()
                .route_reference()
                .expect("unsupported planned route should retain route context")
                .contains("planned-route:signal"));
        }
        _ => panic!("unsupported receipt kinds should still produce denied receipts"),
    }
}

#[test]
fn public_denial_lanes_diverge_for_distinct_denial_topology() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    let required_intent = match handle.receipt_routes_checked(
        WorthQueryDeclarationReceiptInput::route_checked(route_checked_from_input(
            &handle,
            ReceiptInput::<RequiredIntentReceiptFamily>::new("edge:42"),
        )),
    ) {
        WorthQueryDeclarationReceiptChecked::Denied(denial) => denial,
        _ => panic!("required intent receipt should deny"),
    };

    let forbidden_intent = match handle.receipt_routes_checked(
        WorthQueryDeclarationReceiptInput::route_checked(route_checked_with_intent(
            &handle,
            ReceiptInput::<ForbiddenIntentReceiptFamily>::new("edge:42"),
            crate::application::WorthQueryDeclarationRouteIntent::RelationalOnly,
        )),
    ) {
        WorthQueryDeclarationReceiptChecked::Denied(denial) => denial,
        _ => panic!("forbidden intent receipt should deny"),
    };

    assert_eq!(
        required_intent.route_cause(),
        Some(WorthQueryDeclarationRoutePlanDenialCause::IntentRequired)
    );
    assert_eq!(
        forbidden_intent.route_cause(),
        Some(WorthQueryDeclarationRoutePlanDenialCause::IntentForbidden)
    );
    assert_ne!(
        required_intent.receipt().receipt_digest(),
        forbidden_intent.receipt().receipt_digest()
    );
}

#[test]
fn deferred_receipts_explain_route_contract_context() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.receipt_routes_checked(WorthQueryDeclarationReceiptInput::route_checked(
        route_checked_from_input(
            &handle,
            ReceiptInput::<DeferredReceiptFamily>::new("edge:42"),
        ),
    )) {
        WorthQueryDeclarationReceiptChecked::Deferred(receipt) => {
            assert!(receipt
                .receipt()
                .explain()
                .route_reference()
                .expect("deferred receipt should retain route context")
                .contains("contract:"));
        }
        _ => panic!("deferred route plans should produce deferred receipts"),
    }
}
