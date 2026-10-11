use worth_server::{
    WorthServerProductIdempotencyKey, WorthServerProductOperationBaseDigest,
    WorthServerProductOperationInput, WorthServerProductOperationSurfaceDenialCode,
    WorthServerProductSessionCreationRequest, WorthServerProductSnapshotPrecondition,
};

#[path = "support/product_operation_phase_eleven/backend.rs"]
pub mod backend;
#[path = "support/product_session_phase_ten/fixture.rs"]
pub mod fixture;

use backend::{stateful_editor_registration, StatefulProductEditorBackend};
use fixture::{
    apply_payload, build_server, prepared_product_mutation_request,
    prepared_product_mutation_request_with_basis_and_header, prepared_session_request,
};

#[test]
fn compat_product_operation_rejects_basis_override_against_request_authority() {
    let backend = StatefulProductEditorBackend::new();
    let initial_basis = backend.basis_digest();
    let server = build_server(vec![stateful_editor_registration(backend.clone())]);
    let session_request = prepared_session_request(
        &server,
        "workspace-42",
        "branch-9",
        "product_session.open_mutation",
    );
    let prepared_request = prepared_product_mutation_request(
        &server,
        "workspace-42",
        "branch-9",
        "product_editor.apply",
        Some("basis:request"),
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

    let denial = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_request,
            WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_product_session_identity(opened.continuation().product_session_identity())
                .with_snapshot_precondition(
                    WorthServerProductSnapshotPrecondition::at_base_digest(
                        WorthServerProductOperationBaseDigest::new("basis:input")
                            .expect("valid base digest"),
                    ),
                ),
        )
        .expect_err("compat path should not allow typed basis override");

    assert_eq!(
        denial.code(),
        WorthServerProductOperationSurfaceDenialCode::RequestDenied
    );
    assert_eq!(backend.revision(), 0);
}

#[test]
fn compat_product_operation_rejects_idempotency_key_override_against_request_authority() {
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
        "request-key",
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

    let denial = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_request,
            WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_product_session_identity(opened.continuation().product_session_identity())
                .with_snapshot_precondition(WorthServerProductSnapshotPrecondition::at_base_digest(
                    WorthServerProductOperationBaseDigest::new(initial_basis.as_str())
                        .expect("valid base digest"),
                ))
                .with_idempotency_key(
                    WorthServerProductIdempotencyKey::new("different-input-key")
                        .expect("valid key"),
                ),
        )
        .expect_err("compat path should not allow typed idempotency override");

    assert_eq!(
        denial.code(),
        WorthServerProductOperationSurfaceDenialCode::RequestDenied
    );
    assert_eq!(backend.revision(), 0);
}
