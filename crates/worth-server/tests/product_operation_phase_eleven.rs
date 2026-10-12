use worth_server::{
    WorthServerCompatibilityAdmittedProductMutationCommand, WorthServerProductIdempotencyKey,
    WorthServerProductOperationBaseDigest, WorthServerProductOperationSurfaceDenialCode,
    WorthServerProductSessionCreationRequest, WorthServerWorthNativeProductMutationCommand,
};

#[path = "support/product_operation_phase_eleven/backend.rs"]
pub mod backend;
#[path = "support/product_session_phase_ten/fixture.rs"]
pub mod fixture;

use backend::{stateful_editor_registration, StatefulProductEditorBackend};
use fixture::{
    apply_payload, apply_payload_with_title, build_server, direct_session,
    prepared_product_mutation_request_with_basis_and_header, prepared_session_request,
};

#[test]
fn worth_native_product_operation_retries_identical_idempotent_mutation_without_second_state_change(
) {
    let backend = StatefulProductEditorBackend::new();
    let initial_basis = backend.basis_digest();
    let server = build_server(vec![stateful_editor_registration(backend.clone())]);
    let session = direct_session(&server, "workspace-42", "branch-9");
    let mutation_session = session
        .product_sessions()
        .open_mutation(
            WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest(initial_basis.as_str())
                .with_expiry_seconds(300),
        )
        .expect("mutation session should open");
    let idempotency_key =
        WorthServerProductIdempotencyKey::new("apply-rename-attempt-1").expect("valid key");

    let first = session
        .product_operations()
        .execute_mutation(
            WorthServerWorthNativeProductMutationCommand::new(
                "product_editor.apply",
                apply_payload(),
            )
            .within(&mutation_session)
            .against_base_digest(base_digest(&initial_basis))
            .idempotent(idempotency_key.clone()),
        )
        .expect("first mutation should succeed");
    let basis_after_first = backend.basis_digest();
    let replayed = session
        .product_operations()
        .execute_mutation(
            WorthServerWorthNativeProductMutationCommand::new(
                "product_editor.apply",
                apply_payload(),
            )
            .within(&mutation_session)
            .against_base_digest(base_digest(&initial_basis))
            .idempotent(idempotency_key),
        )
        .expect("identical mutation should replay");

    assert_eq!(backend.title(), "Rename");
    assert_ne!(basis_after_first, initial_basis);
    assert_eq!(backend.basis_digest(), basis_after_first);
    assert!(first.retry_diagnostics().is_executed());
    assert!(first.retry_diagnostics().adapter_execution_attempted());
    assert!(replayed.retry_diagnostics().is_previously_committed());
    assert!(replayed
        .retry_diagnostics()
        .adapter_execution_skipped_by_retry());
    assert_eq!(
        replayed
            .retry_receipt()
            .and_then(|receipt| receipt.original_operation_digest()),
        Some(first.envelope().canonical_digest())
    );
}

#[test]
fn compat_product_operation_preserves_phase_eleven_replay_contract_via_request_shaped_command() {
    let backend = StatefulProductEditorBackend::new();
    let initial_basis = backend.basis_digest();
    let server = build_server(vec![stateful_editor_registration(backend.clone())]);
    let session_request = prepared_session_request(
        &server,
        "workspace-42",
        "branch-9",
        "product_session.open_mutation",
    );
    let prepared_request = prepared_product_mutation_request_with_basis_and_header(
        &server,
        "workspace-42",
        "branch-9",
        "product_editor.apply",
        &initial_basis,
        "idempotency-key",
        "compat-apply-attempt-1",
    );
    let opened = server
        .compat_http()
        .product_sessions()
        .open_mutation_for_product_operation(
            &session_request,
            WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest(initial_basis.as_str())
                .with_expiry_seconds(300),
        )
        .expect("compat mutation session should open");

    let first = server
        .compat_http()
        .product_operations()
        .execute_admitted_mutation(
            &prepared_request,
            WorthServerCompatibilityAdmittedProductMutationCommand::new(
                "product_editor.apply",
                apply_payload(),
            )
            .with_session(opened.continuation()),
        )
        .expect("first compat mutation should succeed");
    let basis_after_first = backend.basis_digest();
    let replayed = server
        .compat_http()
        .product_operations()
        .execute_admitted_mutation(
            &prepared_request,
            WorthServerCompatibilityAdmittedProductMutationCommand::new(
                "product_editor.apply",
                apply_payload(),
            )
            .with_session(opened.continuation()),
        )
        .expect("compat mutation replay should succeed");

    assert_eq!(backend.title(), "Rename");
    assert_ne!(basis_after_first, initial_basis);
    assert_eq!(backend.basis_digest(), basis_after_first);
    assert!(first.retry_diagnostics().is_executed());
    assert!(replayed.retry_diagnostics().is_previously_committed());
    assert!(replayed
        .retry_diagnostics()
        .adapter_execution_skipped_by_retry());
    assert_eq!(
        replayed
            .retry_receipt()
            .and_then(|receipt| receipt.original_operation_digest()),
        Some(first.envelope().canonical_digest())
    );
}

#[test]
fn compat_product_operation_reuses_key_with_different_payload_as_conflict() {
    let backend = StatefulProductEditorBackend::new();
    let initial_basis = backend.basis_digest();
    let server = build_server(vec![stateful_editor_registration(backend.clone())]);
    let session_request = prepared_session_request(
        &server,
        "workspace-42",
        "branch-9",
        "product_session.open_mutation",
    );
    let prepared_request = prepared_product_mutation_request_with_basis_and_header(
        &server,
        "workspace-42",
        "branch-9",
        "product_editor.apply",
        &initial_basis,
        "idempotency-key",
        "compat-apply-attempt-2",
    );
    let opened = server
        .compat_http()
        .product_sessions()
        .open_mutation_for_product_operation(
            &session_request,
            WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest(initial_basis.as_str())
                .with_expiry_seconds(300),
        )
        .expect("compat mutation session should open");

    server
        .compat_http()
        .product_operations()
        .execute_admitted_mutation(
            &prepared_request,
            WorthServerCompatibilityAdmittedProductMutationCommand::new(
                "product_editor.apply",
                apply_payload(),
            )
            .with_session(opened.continuation()),
        )
        .expect("first compat mutation should succeed");

    let denial = server
        .compat_http()
        .product_operations()
        .execute_admitted_mutation(
            &prepared_request,
            WorthServerCompatibilityAdmittedProductMutationCommand::new(
                "product_editor.apply",
                apply_payload_with_title("Different"),
            )
            .with_session(opened.continuation()),
        )
        .expect_err("changing payload under the same key should conflict");

    assert_eq!(
        denial.code(),
        WorthServerProductOperationSurfaceDenialCode::IdempotencyConflict
    );
    assert_eq!(
        denial
            .facts()
            .and_then(|facts| facts.idempotency_conflict())
            .map(|conflict| conflict.idempotency_key()),
        Some("compat-apply-attempt-2")
    );
    assert_eq!(backend.revision(), 1);
}

#[test]
fn product_mutation_without_snapshot_precondition_denies_before_adapter_execution() {
    let backend = StatefulProductEditorBackend::new();
    let initial_basis = backend.basis_digest();
    let server = build_server(vec![stateful_editor_registration(backend.clone())]);
    let session = direct_session(&server, "workspace-42", "branch-9");
    let mutation_session = session
        .product_sessions()
        .open_mutation(
            WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest(initial_basis.as_str())
                .with_expiry_seconds(300),
        )
        .expect("mutation session should open");

    let denial = session
        .product_operations()
        .execute_mutation(
            WorthServerWorthNativeProductMutationCommand::new(
                "product_editor.apply",
                apply_payload(),
            )
            .within(&mutation_session),
        )
        .expect_err("missing snapshot precondition should deny");

    assert_eq!(
        denial.code(),
        WorthServerProductOperationSurfaceDenialCode::PreconditionDenied
    );
    assert_eq!(backend.revision(), 0);
    assert_eq!(backend.title(), "Untitled");
}

fn base_digest(value: &str) -> WorthServerProductOperationBaseDigest {
    WorthServerProductOperationBaseDigest::new(value).expect("valid base digest")
}
