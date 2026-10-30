use super::*;

#[test]
fn compat_http_product_sessions_hold_server_admitted_identity_across_requests() {
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_server(vec![session_backed_editor_registration(calls.clone())]);
    let prepared_open_mutation = prepared_session_request(
        &server,
        "workspace-42",
        "branch-9",
        "product_session.open_mutation",
    );
    let prepared_open_preview = prepared_session_request(
        &server,
        "workspace-42",
        "branch-9",
        "product_session.open_preview",
    );
    let prepared_close =
        prepared_session_request(&server, "workspace-42", "branch-9", "product_session.close");

    let opened = server
        .compat_http()
        .product_sessions()
        .open_mutation_with_proof(
            &prepared_open_mutation,
            worth_server::WorthServerProductSessionCreationRequest::for_operation(
                "product_editor.apply",
            )
            .with_basis_digest("basis:head")
            .with_expiry_seconds(300),
        )
        .expect("compat session open should admit");

    assert_eq!(
        opened.plan().concurrency_class(),
        opened.scheduler_admission().concurrency_class()
    );
    assert!(opened
        .scheduler_admission()
        .scheduler_lane()
        .contains("product-session-create:"));

    let applied = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_product_mutation_request(
                &server,
                "workspace-42",
                "branch-9",
                "product_editor.apply",
                Some("basis:head"),
            ),
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.apply",
                apply_payload(),
            )
            .with_product_session_identity(opened.session().identity().as_str()),
        )
        .expect("compat product mutation should accept server-admitted session");
    assert!(applied.adapter_execution_attempted());

    let preview_opened = server
        .compat_http()
        .product_sessions()
        .open_preview(
            &prepared_open_preview,
            worth_server::WorthServerProductSessionCreationRequest::for_operation(
                "product_editor.render_preview",
            )
            .with_basis_digest("basis:head")
            .with_expiry_seconds(300),
        )
        .expect("compat preview session should open");
    let preview_completed = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_product_read_request(
                &server,
                "workspace-42",
                "branch-9",
                "product_editor.render_preview",
                Some("basis:head"),
            ),
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.render_preview",
                preview_payload(),
            )
            .with_product_session_identity(preview_opened.identity().as_str()),
        )
        .expect("compat preview read should succeed");
    assert!(preview_completed.adapter_execution_attempted());

    let closed = server
        .compat_http()
        .product_sessions()
        .close_with_proof(&prepared_close, opened.session().identity())
        .expect("compat close should succeed");
    assert!(closed
        .scheduler_admission()
        .scheduler_lane()
        .contains(opened.session().identity().as_str()));

    let denial = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_product_mutation_request(
                &server,
                "workspace-42",
                "branch-9",
                "product_editor.apply",
                Some("basis:head"),
            ),
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.apply",
                apply_payload(),
            )
            .with_product_session_identity(opened.session().identity().as_str()),
        )
        .expect_err("closed compat session should deny across requests");
    assert_eq!(
        denial.facts().and_then(|facts| facts.session_denial_code()),
        Some(worth_server::WorthServerProductSessionDenialCode::ClosedProductSession)
    );
}
