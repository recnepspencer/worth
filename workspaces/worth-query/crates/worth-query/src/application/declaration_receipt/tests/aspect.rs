use crate::application::{
    assert_declaration_aspect_projections, test_declaration_aspect_key,
    WorthQueryDeclarationReceiptChecked, WorthQueryDeclarationReceiptInput,
    WorthQueryDeclarationRouteIntent,
};
use crate::target_binding::WorthQueryBindingTargetWitness;

use super::support::{
    admitted_handle, route_checked_from_input, route_checked_with_intent,
    AspectDeferredReceiptFamily, AspectFailedReceiptFamily, AspectRichReceiptFamily,
    AspectSignalReceiptFamily, ReceiptInput,
};

#[test]
fn planned_receipts_preserve_route_scoped_aspect_contract() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let receipt = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_and_receipt(
            ReceiptInput::<AspectRichReceiptFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("aspect-rich receipt should issue"));

    assert_declaration_aspect_projections(
        receipt.aspect_contract().required(),
        &["selection.active_edge"],
    );
    assert_declaration_aspect_projections(
        receipt.aspect_contract().preserved(),
        &["selection.local_topology"],
    );
    assert!(receipt.aspect_contract().published().is_empty());
    assert!(!receipt
        .aspect_publication()
        .present()
        .contains(&test_declaration_aspect_key("selection.material_edit")));
    assert!(receipt
        .aspect_publication()
        .masked()
        .contains(&test_declaration_aspect_key("selection.private_authority")));
}

#[test]
fn receipt_binding_target_retains_crossing_aspect_state() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let receipt = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_and_receipt(
            ReceiptInput::<AspectRichReceiptFamily>::new("edge:42"),
        )
        .unwrap_or_else(|_| panic!("aspect-rich receipt should issue"));

    let binding = receipt.binding_target();
    let semantics = binding.erased_target().semantics();
    let (_, _, _, _, contract, coverage, publication) = semantics
        .declaration_receipt()
        .expect("receipt binding target should retain crossing semantics");

    assert_eq!(contract, receipt.aspect_contract());
    assert_eq!(coverage, receipt.aspect_coverage());
    assert_eq!(publication, receipt.aspect_publication());
}

#[test]
fn denied_receipts_keep_scoped_aspect_publication_without_widening() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.receipt_routes_checked(WorthQueryDeclarationReceiptInput::route_checked(
        route_checked_with_intent(
            &handle,
            ReceiptInput::<AspectSignalReceiptFamily>::new("edge:42"),
            WorthQueryDeclarationRouteIntent::Auto,
        ),
    )) {
        WorthQueryDeclarationReceiptChecked::Denied(denial) => {
            let receipt = denial.receipt();
            assert_declaration_aspect_projections(
                receipt.aspect_contract().required(),
                &["selection.active_edge"],
            );
            assert!(!receipt
                .aspect_publication()
                .present()
                .contains(&test_declaration_aspect_key("selection.material_edit")));
        }
        _ => panic!("unsupported receipt kind should deny with retained aspect truth"),
    }
}

#[test]
fn deferred_and_failed_receipts_keep_honest_scoped_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match handle.receipt_routes_checked(WorthQueryDeclarationReceiptInput::route_checked(
        route_checked_from_input(
            &handle,
            ReceiptInput::<AspectDeferredReceiptFamily>::new("edge:42"),
        ),
    )) {
        WorthQueryDeclarationReceiptChecked::Deferred(receipt) => {
            assert!(receipt
                .receipt()
                .aspect_publication()
                .masked()
                .contains(&test_declaration_aspect_key("selection.private_authority")));
        }
        _ => panic!("aspect-rich deferred receipt should remain deferred"),
    }

    match handle.receipt_routes_checked(WorthQueryDeclarationReceiptInput::route_checked(
        route_checked_from_input(
            &handle,
            ReceiptInput::<AspectFailedReceiptFamily>::new("edge:42"),
        ),
    )) {
        WorthQueryDeclarationReceiptChecked::Failed(receipt) => {
            assert!(receipt
                .receipt()
                .aspect_publication()
                .masked()
                .contains(&test_declaration_aspect_key("selection.private_authority")));
        }
        _ => panic!("aspect-rich failed receipt should remain failed"),
    }
}
