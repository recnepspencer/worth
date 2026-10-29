#[allow(unused_imports)] use crate::product_session_phase_ten::ordinary_context::*;
use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc, std::sync::Barrier, std::sync::Mutex};

use {worth_server::WorthServerProductOperationExecutionBoundary, worth_server::WorthServerProductOperationInput, worth_server::WorthServerProductOperationOutcome, worth_server::WorthServerProductOperationSurfaceDenialCode, worth_server::WorthServerProductSessionCreationRequest, worth_server::WorthServerProductSessionDenialCode, worth_server::WorthServerProductSessionLifecycle, worth_server::WorthServerProductSessionTermination, worth_server::WorthServerProductSessionTerminationKind, worth_server::WorthServerProductSessionTerminationObserver};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_product_session_phase_ten_fixture as fixture};

use {worth_testkit_server::shared_support_product_session_phase_ten_fixture::apply_payload, worth_testkit_server::shared_support_product_session_phase_ten_fixture::build_server, worth_testkit_server::shared_support_product_session_phase_ten_fixture::build_server_with_clock_and_observers, worth_testkit_server::shared_support_product_session_phase_ten_fixture::direct_session, worth_testkit_server::shared_support_product_session_phase_ten_fixture::prepared_product_mutation_request, worth_testkit_server::shared_support_product_session_phase_ten_fixture::prepared_product_read_request, worth_testkit_server::shared_support_product_session_phase_ten_fixture::prepared_session_request, worth_testkit_server::shared_support_product_session_phase_ten_fixture::preview_payload, worth_testkit_server::shared_support_product_session_phase_ten_fixture::session_backed_editor_registration, worth_testkit_server::shared_support_product_session_phase_ten_fixture::manual_clock::ManualProductSessionClock};

#[test]
fn product_session_termination_observers_receive_close_and_expiry_once() {
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let clock = std::sync::Arc::new(worth_testkit_server::shared_support_product_session_phase_ten_fixture::manual_clock::ManualProductSessionClock::new(1_000));
    let observer = std::sync::Arc::new(crate::product_session_phase_ten::ordinary_context::RecordingTerminationObserver::default());
    let server = build_server_with_clock_and_observers(
        vec![session_backed_editor_registration(calls)],
        Some(clock.clone()),
        vec![observer.clone()],
    );
    let session = direct_session(&server, "workspace-42", "branch-9");
    let expiring = session
        .product_sessions()
        .open_mutation(
            worth_server::WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest("basis:head")
                .with_expiry_seconds(1),
        )
        .expect("expiring session should open");
    let closing = session
        .product_sessions()
        .open_mutation(
            worth_server::WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest("basis:head"),
        )
        .expect("closing session should open");

    clock.advance_millis(1_001);
    let _ = session.product_operations().execute(
        worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
            .with_basis_digest("basis:head")
            .with_product_session_identity(expiring.identity().as_str()),
    );
    let _ = session.product_operations().execute(
        worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
            .with_basis_digest("basis:head")
            .with_product_session_identity(expiring.identity().as_str()),
    );
    session
        .product_sessions()
        .close(closing.identity())
        .expect("session should close");
    let repeated_close = session
        .product_sessions()
        .close(closing.identity())
        .expect_err("closed sessions cannot terminate twice");
    assert_eq!(
        repeated_close.code(),
        worth_server::WorthServerProductSessionDenialCode::ClosedProductSession,
    );
    let expired_close = session
        .product_sessions()
        .close(expiring.identity())
        .expect_err("expired sessions cannot transition to closed");
    assert_eq!(
        expired_close.code(),
        worth_server::WorthServerProductSessionDenialCode::ExpiredProductSession,
    );

    assert_eq!(
        *observer.events.lock().expect("observer lock"),
        vec![
            (
                expiring.identity().as_str().to_string(),
                worth_server::WorthServerProductSessionTerminationKind::Expired,
            ),
            (
                closing.identity().as_str().to_string(),
                worth_server::WorthServerProductSessionTerminationKind::Closed,
            ),
        ]
    );
}

#[test]
fn concurrent_expiry_and_close_publish_one_terminal_outcome() {
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let clock = std::sync::Arc::new(worth_testkit_server::shared_support_product_session_phase_ten_fixture::manual_clock::ManualProductSessionClock::new(1_000));
    let observer = std::sync::Arc::new(crate::product_session_phase_ten::ordinary_context::RecordingTerminationObserver::default());
    let server = build_server_with_clock_and_observers(
        vec![session_backed_editor_registration(calls)],
        Some(clock.clone()),
        vec![observer.clone()],
    );
    let session = direct_session(&server, "workspace-42", "branch-9");
    let expiring = session
        .product_sessions()
        .open_mutation(
            worth_server::WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest("basis:head")
                .with_expiry_seconds(1),
        )
        .expect("expiring session should open");
    clock.advance_millis(1_001);

    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let close_session = session.clone();
    let close_identity = expiring.identity().clone();
    let close_barrier = barrier.clone();
    let close = std::thread::spawn(move || {
        close_barrier.wait();
        close_session.product_sessions().close(&close_identity)
    });
    let lookup_session = session.clone();
    let lookup_identity = expiring.identity().as_str().to_string();
    let lookup = std::thread::spawn(move || {
        barrier.wait();
        lookup_session.product_operations().execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_basis_digest("basis:head")
                .with_product_session_identity(lookup_identity),
        )
    });

    let close_denial = close
        .join()
        .expect("close worker should finish")
        .expect_err("expired close should deny");
    assert_eq!(
        close_denial.code(),
        worth_server::WorthServerProductSessionDenialCode::ExpiredProductSession,
    );
    lookup
        .join()
        .expect("lookup worker should finish")
        .expect_err("expired product operation should deny");
    assert_eq!(
        *observer.events.lock().expect("observer lock"),
        vec![(
            expiring.identity().as_str().to_string(),
            worth_server::WorthServerProductSessionTerminationKind::Expired,
        )],
    );
}

#[test]
fn product_session_identity_is_server_admitted_not_adapter_fabricated() {
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_server(vec![session_backed_editor_registration(calls.clone())]);
    let session = direct_session(&server, "workspace-42", "branch-9");

    let denial = session
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_basis_digest("basis:head")
                .with_product_session_identity("product-session:Worthd"),
        )
        .expect_err("fabricated session identity should deny before adapter execution");

    assert_eq!(
        denial.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(
        denial.facts().and_then(|facts| facts.session_denial_code()),
        Some(worth_server::WorthServerProductSessionDenialCode::UnknownProductSessionIdentity)
    );
    assert_eq!(
        denial.facts().and_then(|facts| facts.execution_boundary()),
        Some(&worth_server::WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution)
    );
}

#[test]
fn preview_and_mutation_session_postures_remain_distinct() {
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_server(vec![session_backed_editor_registration(calls.clone())]);
    let session = direct_session(&server, "workspace-42", "branch-9");

    let preview = session
        .product_sessions()
        .open_preview(
            worth_server::WorthServerProductSessionCreationRequest::for_operation(
                "product_editor.render_preview",
            )
            .with_basis_digest("basis:head")
            .with_expiry_seconds(300),
        )
        .expect("preview session should open");
    let mutation = session
        .product_sessions()
        .open_mutation(
            worth_server::WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest("basis:head")
                .with_expiry_seconds(300),
        )
        .expect("mutation session should open");

    assert_ne!(preview.canonical_digest(), mutation.canonical_digest());
    assert_eq!(
        preview.lifecycle(),
        worth_server::WorthServerProductSessionLifecycle::ReadOnlyPreview
    );
    assert_eq!(
        mutation.lifecycle(),
        worth_server::WorthServerProductSessionLifecycle::MutationDraft
    );

    let preview_denial = session
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_basis_digest("basis:head")
                .with_product_session_identity(preview.identity().as_str()),
        )
        .expect_err("preview session must not authorize mutation");
    assert_eq!(
        preview_denial.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(
        preview_denial
            .facts()
            .and_then(|facts| facts.session_denial_code()),
        Some(worth_server::WorthServerProductSessionDenialCode::PreviewSessionCannotMutate)
    );

    let mutation_completed = session
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_basis_digest("basis:head")
                .with_product_session_identity(mutation.identity().as_str()),
        )
        .expect("mutation session should authorize mutation");
    assert!(matches!(
        mutation_completed.outcome(),
        worth_server::WorthServerProductOperationOutcome::Success(_)
    ));

    let counters = server.product_session_registry().counter_snapshot();
    assert_eq!(counters.preview_sessions_created, 1);
    assert_eq!(counters.mutation_sessions_created, 1);
    assert_eq!(counters.lookups_denied_preview_for_mutation, 1);
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
}

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
            worth_server::WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
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
            worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
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
            worth_server::WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_product_session_identity(opened.session().identity().as_str()),
        )
        .expect_err("closed compat session should deny across requests");
    assert_eq!(
        denial.facts().and_then(|facts| facts.session_denial_code()),
        Some(worth_server::WorthServerProductSessionDenialCode::ClosedProductSession)
    );
}
