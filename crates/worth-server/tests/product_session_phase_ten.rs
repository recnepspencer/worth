use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Barrier, Mutex,
};

use worth_server::{
    WorthServerProductOperationExecutionBoundary, WorthServerProductOperationInput,
    WorthServerProductOperationOutcome, WorthServerProductOperationSurfaceDenialCode,
    WorthServerProductSessionCreationRequest, WorthServerProductSessionDenialCode,
    WorthServerProductSessionLifecycle, WorthServerProductSessionTermination,
    WorthServerProductSessionTerminationKind, WorthServerProductSessionTerminationObserver,
};

#[allow(unused_imports)]
pub use crate::shared_support_product_session_phase_ten_fixture as fixture;

use fixture::{
    apply_payload, build_server, build_server_with_clock_and_observers, direct_session,
    prepared_product_mutation_request, prepared_product_read_request, prepared_session_request,
    preview_payload, session_backed_editor_registration, ManualProductSessionClock,
};

#[derive(Debug, Default)]
struct RecordingTerminationObserver {
    events: Mutex<Vec<(String, WorthServerProductSessionTerminationKind)>>,
}

impl WorthServerProductSessionTerminationObserver for RecordingTerminationObserver {
    fn observe_termination(&self, termination: &WorthServerProductSessionTermination) {
        self.events.lock().expect("observer lock").push((
            termination.session().identity().as_str().to_string(),
            termination.kind(),
        ));
    }
}

#[test]
fn product_session_termination_observers_receive_close_and_expiry_once() {
    let calls = Arc::new(AtomicUsize::new(0));
    let clock = Arc::new(ManualProductSessionClock::new(1_000));
    let observer = Arc::new(RecordingTerminationObserver::default());
    let server = build_server_with_clock_and_observers(
        vec![session_backed_editor_registration(calls)],
        Some(clock.clone()),
        vec![observer.clone()],
    );
    let session = direct_session(&server, "workspace-42", "branch-9");
    let expiring = session
        .product_sessions()
        .open_mutation(
            WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest("basis:head")
                .with_expiry_seconds(1),
        )
        .expect("expiring session should open");
    let closing = session
        .product_sessions()
        .open_mutation(
            WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest("basis:head"),
        )
        .expect("closing session should open");

    clock.advance_millis(1_001);
    let _ = session.product_operations().execute(
        WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
            .with_basis_digest("basis:head")
            .with_product_session_identity(expiring.identity().as_str()),
    );
    let _ = session.product_operations().execute(
        WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
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
        WorthServerProductSessionDenialCode::ClosedProductSession,
    );
    let expired_close = session
        .product_sessions()
        .close(expiring.identity())
        .expect_err("expired sessions cannot transition to closed");
    assert_eq!(
        expired_close.code(),
        WorthServerProductSessionDenialCode::ExpiredProductSession,
    );

    assert_eq!(
        *observer.events.lock().expect("observer lock"),
        vec![
            (
                expiring.identity().as_str().to_string(),
                WorthServerProductSessionTerminationKind::Expired,
            ),
            (
                closing.identity().as_str().to_string(),
                WorthServerProductSessionTerminationKind::Closed,
            ),
        ]
    );
}

#[test]
fn concurrent_expiry_and_close_publish_one_terminal_outcome() {
    let calls = Arc::new(AtomicUsize::new(0));
    let clock = Arc::new(ManualProductSessionClock::new(1_000));
    let observer = Arc::new(RecordingTerminationObserver::default());
    let server = build_server_with_clock_and_observers(
        vec![session_backed_editor_registration(calls)],
        Some(clock.clone()),
        vec![observer.clone()],
    );
    let session = direct_session(&server, "workspace-42", "branch-9");
    let expiring = session
        .product_sessions()
        .open_mutation(
            WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest("basis:head")
                .with_expiry_seconds(1),
        )
        .expect("expiring session should open");
    clock.advance_millis(1_001);

    let barrier = Arc::new(Barrier::new(2));
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
            WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
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
        WorthServerProductSessionDenialCode::ExpiredProductSession,
    );
    lookup
        .join()
        .expect("lookup worker should finish")
        .expect_err("expired product operation should deny");
    assert_eq!(
        *observer.events.lock().expect("observer lock"),
        vec![(
            expiring.identity().as_str().to_string(),
            WorthServerProductSessionTerminationKind::Expired,
        )],
    );
}

#[test]
fn product_session_identity_is_server_admitted_not_adapter_fabricated() {
    let calls = Arc::new(AtomicUsize::new(0));
    let server = build_server(vec![session_backed_editor_registration(calls.clone())]);
    let session = direct_session(&server, "workspace-42", "branch-9");

    let denial = session
        .product_operations()
        .execute(
            WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_basis_digest("basis:head")
                .with_product_session_identity("product-session:Worthd"),
        )
        .expect_err("fabricated session identity should deny before adapter execution");

    assert_eq!(
        denial.code(),
        WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(
        denial.facts().and_then(|facts| facts.session_denial_code()),
        Some(WorthServerProductSessionDenialCode::UnknownProductSessionIdentity)
    );
    assert_eq!(
        denial.facts().and_then(|facts| facts.execution_boundary()),
        Some(&WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution)
    );
}

#[test]
fn preview_and_mutation_session_postures_remain_distinct() {
    let calls = Arc::new(AtomicUsize::new(0));
    let server = build_server(vec![session_backed_editor_registration(calls.clone())]);
    let session = direct_session(&server, "workspace-42", "branch-9");

    let preview = session
        .product_sessions()
        .open_preview(
            WorthServerProductSessionCreationRequest::for_operation(
                "product_editor.render_preview",
            )
            .with_basis_digest("basis:head")
            .with_expiry_seconds(300),
        )
        .expect("preview session should open");
    let mutation = session
        .product_sessions()
        .open_mutation(
            WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
                .with_basis_digest("basis:head")
                .with_expiry_seconds(300),
        )
        .expect("mutation session should open");

    assert_ne!(preview.canonical_digest(), mutation.canonical_digest());
    assert_eq!(
        preview.lifecycle(),
        WorthServerProductSessionLifecycle::ReadOnlyPreview
    );
    assert_eq!(
        mutation.lifecycle(),
        WorthServerProductSessionLifecycle::MutationDraft
    );

    let preview_denial = session
        .product_operations()
        .execute(
            WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_basis_digest("basis:head")
                .with_product_session_identity(preview.identity().as_str()),
        )
        .expect_err("preview session must not authorize mutation");
    assert_eq!(
        preview_denial.code(),
        WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(
        preview_denial
            .facts()
            .and_then(|facts| facts.session_denial_code()),
        Some(WorthServerProductSessionDenialCode::PreviewSessionCannotMutate)
    );

    let mutation_completed = session
        .product_operations()
        .execute(
            WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_basis_digest("basis:head")
                .with_product_session_identity(mutation.identity().as_str()),
        )
        .expect("mutation session should authorize mutation");
    assert!(matches!(
        mutation_completed.outcome(),
        WorthServerProductOperationOutcome::Success(_)
    ));

    let counters = server.product_session_registry().counter_snapshot();
    assert_eq!(counters.preview_sessions_created, 1);
    assert_eq!(counters.mutation_sessions_created, 1);
    assert_eq!(counters.lookups_denied_preview_for_mutation, 1);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[test]
fn compat_http_product_sessions_hold_server_admitted_identity_across_requests() {
    let calls = Arc::new(AtomicUsize::new(0));
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
            WorthServerProductSessionCreationRequest::for_operation("product_editor.apply")
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
            WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_product_session_identity(opened.session().identity().as_str()),
        )
        .expect("compat product mutation should accept server-admitted session");
    assert!(applied.adapter_execution_attempted());

    let preview_opened = server
        .compat_http()
        .product_sessions()
        .open_preview(
            &prepared_open_preview,
            WorthServerProductSessionCreationRequest::for_operation(
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
            WorthServerProductOperationInput::new(
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
            WorthServerProductOperationInput::new("product_editor.apply", apply_payload())
                .with_product_session_identity(opened.session().identity().as_str()),
        )
        .expect_err("closed compat session should deny across requests");
    assert_eq!(
        denial.facts().and_then(|facts| facts.session_denial_code()),
        Some(WorthServerProductSessionDenialCode::ClosedProductSession)
    );
}
