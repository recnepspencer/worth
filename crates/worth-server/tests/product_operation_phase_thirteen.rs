#[path = "support/test_bootstrap_policy.rs"]
mod test_bootstrap_policy;

use worth_server::{
    WorthServerProductIdempotencyKey, WorthServerProductOperationInput,
    WorthServerProductOperationOutcome, WorthServerProductOperationSurfaceDenialCode,
};

#[path = "support/product_operation_phase_thirteen/fixture.rs"]
mod fixture;

use fixture::{
    actions_payload, apply_payload, build_server, direct_mutation, direct_read, direct_session,
    finalize_payload, open_mutation_session, render_payload, select_payload,
    StatefulEditorLikeBackend,
};

#[test]
fn product_editor_fixture_has_real_pressure_shape() {
    let backend = StatefulEditorLikeBackend::new();
    let server = build_server(&backend);
    let session = direct_session(&server);
    let initial_basis = backend.basis_digest();

    let render = direct_read(
        &session,
        "product_editor.render",
        render_payload(),
        &initial_basis,
    );
    let selected = direct_read(
        &session,
        "product_editor.select",
        select_payload("node-7"),
        &initial_basis,
    );
    let actions_before = direct_read(
        &session,
        "product_editor.available_actions",
        actions_payload(),
        &initial_basis,
    );
    let finalize_session =
        open_mutation_session(&session, &initial_basis, "product_editor.finalize");
    let denied_finalize = direct_mutation(
        &session,
        "product_editor.finalize",
        finalize_payload(false),
        &initial_basis,
        &finalize_session,
    )
    .expect("finalize confirmation denial should be enveloped");
    let apply_session = open_mutation_session(&session, &initial_basis, "product_editor.apply");
    let applied = direct_mutation(
        &session,
        "product_editor.apply",
        apply_payload("Renamed"),
        &initial_basis,
        &apply_session,
    )
    .expect("apply should succeed");
    let actions_after = direct_read(
        &session,
        "product_editor.available_actions",
        actions_payload(),
        &backend.basis_digest(),
    );

    let render_body = match render.outcome() {
        WorthServerProductOperationOutcome::Success(success) => {
            success.result_artifact().body().value()
        }
        other => panic!("expected render success, got {other:?}"),
    };
    let select_body = match selected.outcome() {
        WorthServerProductOperationOutcome::Success(success) => {
            success.result_artifact().body().value()
        }
        other => panic!("expected select success, got {other:?}"),
    };
    let actions_before_digest = match actions_before.outcome() {
        WorthServerProductOperationOutcome::Success(success) => {
            success.result_artifact().artifact_digest()
        }
        other => panic!("expected actions success, got {other:?}"),
    };
    let actions_after_digest = match actions_after.outcome() {
        WorthServerProductOperationOutcome::Success(success) => {
            success.result_artifact().artifact_digest()
        }
        other => panic!("expected actions success, got {other:?}"),
    };

    assert_eq!(render_body["request_basis"], initial_basis);
    assert_eq!(select_body["node"], "node-7");
    assert_ne!(actions_before_digest, actions_after_digest);
    assert_ne!(backend.basis_digest(), initial_basis);
    match denied_finalize.outcome() {
        WorthServerProductOperationOutcome::Denied(denial) => {
            assert_eq!(denial.reason_key(), "product.finalize.confirm_required");
        }
        other => panic!("expected finalize denial, got {other:?}"),
    }
    assert_eq!(backend.title(), "Renamed");
    assert!(applied.adapter_execution_attempted());

    let replay_basis = backend.basis_digest();
    let replay_session = open_mutation_session(&session, &replay_basis, "product_editor.apply");
    let replay_key =
        WorthServerProductIdempotencyKey::new("phase-thirteen-pressure-replay").expect("key");
    let first_replay = session
        .product_operations()
        .execute(
            WorthServerProductOperationInput::new(
                "product_editor.apply",
                apply_payload("Renamed Again"),
            )
            .with_basis_digest(&replay_basis)
            .with_product_session_identity(replay_session.identity().as_str())
            .with_idempotency_key(replay_key.clone()),
        )
        .expect("authoritative replay candidate should succeed");
    let replayed = session
        .product_operations()
        .execute(
            WorthServerProductOperationInput::new(
                "product_editor.apply",
                apply_payload("Renamed Again"),
            )
            .with_basis_digest(&replay_basis)
            .with_product_session_identity(replay_session.identity().as_str())
            .with_idempotency_key(replay_key),
        )
        .expect("identical mutation should replay");

    assert!(first_replay.retry_diagnostics().is_executed());
    assert!(replayed.retry_diagnostics().is_previously_committed());
    assert!(replayed
        .retry_diagnostics()
        .adapter_execution_skipped_by_retry());
    assert_eq!(
        replayed
            .retry_receipt()
            .and_then(|receipt| receipt.original_operation_digest()),
        Some(first_replay.envelope().canonical_digest())
    );
}

#[test]
fn product_editor_like_apply_and_finalize_are_deterministic_and_stale_safe() {
    let backend = StatefulEditorLikeBackend::new();
    let server = build_server(&backend);
    let session = direct_session(&server);
    let initial_basis = backend.basis_digest();
    let mutation_session = open_mutation_session(&session, &initial_basis, "product_editor.apply");

    let first = direct_mutation(
        &session,
        "product_editor.apply",
        apply_payload("Draft One"),
        &initial_basis,
        &mutation_session,
    )
    .expect("first apply should succeed");
    let after_apply_basis = backend.basis_digest();
    let stale = direct_mutation(
        &session,
        "product_editor.apply",
        apply_payload("Stale Rewrite"),
        &after_apply_basis,
        &mutation_session,
    )
    .expect_err("stale apply should deny before adapter execution");
    let finalize_session =
        open_mutation_session(&session, &after_apply_basis, "product_editor.finalize");
    let finalized = direct_mutation(
        &session,
        "product_editor.finalize",
        finalize_payload(true),
        &after_apply_basis,
        &finalize_session,
    )
    .expect("finalize should succeed once stricter preconditions hold");

    assert_ne!(after_apply_basis, initial_basis);
    assert_eq!(
        stale.code(),
        WorthServerProductOperationSurfaceDenialCode::PreconditionDenied
    );
    assert_eq!(backend.title(), "Draft One");
    assert!(first.adapter_execution_attempted());
    assert!(finalized.adapter_execution_attempted());
}
