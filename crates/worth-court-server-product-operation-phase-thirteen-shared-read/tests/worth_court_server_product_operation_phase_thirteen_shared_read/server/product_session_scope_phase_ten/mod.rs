use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

use {worth_server::WorthServerProductOperationExecutionBoundary, worth_server::WorthServerProductOperationInput, worth_server::WorthServerProductOperationOutcome, worth_server::WorthServerProductOperationSurfaceDenialCode, worth_server::WorthServerProductSessionCreationRequest, worth_server::WorthServerProductSessionDenialCode, worth_server::WorthServerProductSessionLifecycle};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_product_session_phase_ten_fixture as fixture};

use {worth_testkit_server::shared_support_product_session_phase_ten_fixture::apply_payload, worth_testkit_server::shared_support_product_session_phase_ten_fixture::build_server_with_clock, worth_testkit_server::shared_support_product_session_phase_ten_fixture::direct_session, worth_testkit_server::shared_support_product_session_phase_ten_fixture::preview_payload, worth_testkit_server::shared_support_product_session_phase_ten_fixture::session_backed_editor_registration, worth_testkit_server::shared_support_product_session_phase_ten_fixture::manual_clock::ManualProductSessionClock};

#[test]
fn product_session_lifecycle_denies_expired_foreign_or_moved_sessions() {
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let clock = std::sync::Arc::new(worth_testkit_server::shared_support_product_session_phase_ten_fixture::manual_clock::ManualProductSessionClock::new(1_000));
    let server = build_server_with_clock(
        vec![session_backed_editor_registration(calls.clone())],
        Some(clock.clone()),
    );
    let primary_session = direct_session(&server, "workspace-42", "branch-9");

    let preview_session = primary_session
        .product_sessions()
        .open_preview(
            worth_server::WorthServerProductSessionCreationRequest::for_operation(
                "product_editor.render_preview",
            )
            .with_basis_digest("basis:head")
            .with_expiry_seconds(300),
        )
        .expect("preview session should open");
    assert_eq!(
        preview_session.lifecycle(),
        worth_server::WorthServerProductSessionLifecycle::ReadOnlyPreview
    );
    let preview_completed = primary_session
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.render_preview",
                preview_payload(),
            )
            .with_basis_digest("basis:head")
            .with_product_session_identity(preview_session.identity().as_str()),
        )
        .expect("preview session should support read-only preview");
    assert!(matches!(
        preview_completed.outcome(),
        worth_server::WorthServerProductOperationOutcome::Success(_)
    ));

    let mutation_session = primary_session
        .product_sessions()
        .open_mutation(
            worth_server::WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest("basis:head")
                .with_expiry_seconds(300),
        )
        .expect("mutation session should open");
    assert_eq!(
        mutation_session.lifecycle(),
        worth_server::WorthServerProductSessionLifecycle::MutationDraft
    );
    let applied = primary_session
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_basis_digest("basis:head")
                .with_product_session_identity(mutation_session.identity().as_str()),
        )
        .expect("mutation session should admit mutation work");
    assert!(matches!(
        applied.outcome(),
        worth_server::WorthServerProductOperationOutcome::Success(_)
    ));

    let expired_session = primary_session
        .product_sessions()
        .open_mutation(
            worth_server::WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest("basis:head")
                .with_expiry_seconds(1),
        )
        .expect("expired test session should open");
    clock.advance_millis(1_001);
    let expired = primary_session
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_basis_digest("basis:head")
                .with_product_session_identity(expired_session.identity().as_str()),
        )
        .expect_err("expired session should deny before mutation scheduling");
    assert_eq!(
        expired.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(
        expired
            .facts()
            .and_then(|facts| facts.session_denial_code()),
        Some(worth_server::WorthServerProductSessionDenialCode::ExpiredProductSession)
    );
    assert_eq!(
        expired.facts().and_then(|facts| facts.execution_boundary()),
        Some(&worth_server::WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution)
    );

    primary_session
        .product_sessions()
        .close(mutation_session.identity())
        .expect("close should succeed");
    let closed = primary_session
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_basis_digest("basis:head")
                .with_product_session_identity(mutation_session.identity().as_str()),
        )
        .expect_err("closed session should deny");
    assert_eq!(
        closed.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(
        closed.facts().and_then(|facts| facts.session_denial_code()),
        Some(worth_server::WorthServerProductSessionDenialCode::ClosedProductSession)
    );

    let foreign_session = direct_session(&server, "workspace-else", "branch-9")
        .product_sessions()
        .open_mutation(
            worth_server::WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest("basis:head")
                .with_expiry_seconds(300),
        )
        .expect("foreign workspace session should open");
    let foreign = primary_session
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_basis_digest("basis:head")
                .with_product_session_identity(foreign_session.identity().as_str()),
        )
        .expect_err("foreign workspace session should deny");
    assert_eq!(
        foreign.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(
        foreign
            .facts()
            .and_then(|facts| facts.session_denial_code()),
        Some(worth_server::WorthServerProductSessionDenialCode::ForeignProductSession)
    );

    let moved_session = direct_session(&server, "workspace-42", "branch-else")
        .product_sessions()
        .open_mutation(
            worth_server::WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest("basis:head")
                .with_expiry_seconds(300),
        )
        .expect("moved-session test session should open");
    let moved = primary_session
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_basis_digest("basis:head")
                .with_product_session_identity(moved_session.identity().as_str()),
        )
        .expect_err("branch-moved session should deny");
    assert_eq!(
        moved.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(
        moved.facts().and_then(|facts| facts.session_denial_code()),
        Some(worth_server::WorthServerProductSessionDenialCode::SessionRebindRequired)
    );

    let counters = server.product_session_registry().counter_snapshot();
    assert_eq!(counters.sessions_created, 5);
    assert_eq!(counters.preview_sessions_created, 1);
    assert_eq!(counters.mutation_sessions_created, 4);
    assert_eq!(counters.lookups_attempted, 7);
    assert_eq!(counters.lookups_denied_expired, 1);
    assert_eq!(counters.lookups_denied_closed, 1);
    assert_eq!(counters.lookups_denied_foreign, 1);
    assert_eq!(counters.lookups_denied_moved, 1);
    assert_eq!(counters.closes_recorded, 1);
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 2);
}
