use crate::application::{WorthQueryDeclarationReceiptInput, WorthQueryDeclarationRoutePlanInput};
use crate::recovery_boundary::{
    WorthQueryRecoveryAction, WorthQueryRecoverySourceFamily, WorthQueryRecoveryStopFamily,
};

use super::support::{
    recovery_admitted_handle, recovery_foundational, recovery_progressed, RecoveryInput,
    RequiredIntentRouteFamily, SignalReceiptFamily,
};

#[test]
fn route_plan_recovery_preserves_typed_route_denial_cause() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = recovery_admitted_handle(request_execution, "primary");
    let progression = recovery_progressed(
        &handle,
        RecoveryInput::<RequiredIntentRouteFamily>::new("edge-a"),
    );
    let evidence = recovery_foundational(&handle, progression.clone());
    let checked = handle.plan_routes_checked(WorthQueryDeclarationRoutePlanInput::admitted(
        progression,
        evidence,
    ));
    let brief = handle
        .recover_from_declaration_route_plan_checked(checked)
        .expect("route denial should yield a recovery brief");

    assert_eq!(
        brief.stop_family(),
        WorthQueryRecoveryStopFamily::DeclarationRoutePlan
    );
    assert_eq!(
        brief.explanation().source_family(),
        WorthQueryRecoverySourceFamily::DeclarationRoutePlan
    );
    assert_eq!(
        brief.route_sensitive_explanation().route_denial_cause(),
        Some(crate::application::WorthQueryDeclarationRoutePlanDenialCause::IntentRequired)
    );
    assert_eq!(
        brief.route_sensitive_explanation().route_governing_reason(),
        Some("the declaration needs caller route intent before relational lowering")
    );
    assert_eq!(
        brief.recovery_request().explanation().route_denial_cause(),
        Some(crate::application::WorthQueryDeclarationRoutePlanDenialCause::IntentRequired)
    );
}

#[test]
fn receipt_recovery_preserves_typed_receipt_denial_cause() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = recovery_admitted_handle(request_execution, "primary");
    let progression =
        recovery_progressed(&handle, RecoveryInput::<SignalReceiptFamily>::new("edge-a"));
    let evidence = recovery_foundational(&handle, progression.clone());
    let route_checked = handle.plan_routes_checked(WorthQueryDeclarationRoutePlanInput::admitted(
        progression,
        evidence,
    ));
    let checked = handle.receipt_routes_checked(WorthQueryDeclarationReceiptInput::route_checked(
        route_checked,
    ));
    let brief = handle
        .recover_from_declaration_receipt_checked(checked)
        .expect("receipt denial should yield a recovery brief");

    assert_eq!(
        brief.stop_family(),
        WorthQueryRecoveryStopFamily::DeclarationReceipt
    );
    assert_eq!(
        brief.explanation().source_family(),
        WorthQueryRecoverySourceFamily::DeclarationReceipt
    );
    assert_eq!(
        brief.route_sensitive_explanation().receipt_denial_cause(),
        Some(crate::application::WorthQueryDeclarationReceiptDenialCause::UnsupportedReceiptKind)
    );
    assert_eq!(
        brief
            .route_sensitive_explanation()
            .receipt_governing_reason(),
        Some("this declaration route kind is not yet a supported Query receipt crossing")
    );
    assert_eq!(
        brief.recommended_action(),
        WorthQueryRecoveryAction::CheckSupport
    );
}
