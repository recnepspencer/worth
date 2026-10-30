#[allow(unused_imports)]
use crate::product_operation_phase_eleven_stateful::ordinary_context::*;

#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_product_operation_phase_eleven_backend as backend;
#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_product_session_phase_ten_fixture as fixture;

use {
    worth_testkit_server::shared_support_product_operation_phase_eleven_backend::controlled_apply_payload,
    worth_testkit_server::shared_support_product_operation_phase_eleven_backend::stateful_editor_registration,
};
use {
    worth_testkit_server::shared_support_product_session_phase_ten_fixture::apply_payload,
    worth_testkit_server::shared_support_product_session_phase_ten_fixture::build_server,
    worth_testkit_server::shared_support_product_session_phase_ten_fixture::direct_session,
};

#[test]
fn product_mutation_stale_basis_denies_after_real_progress() {
    let backend = worth_testkit_server::shared_support_product_operation_phase_eleven_backend::StatefulProductEditorBackend::new();
    let initial_basis = backend.basis_digest();
    let server = build_server(vec![stateful_editor_registration(backend.clone())]);
    let session = direct_session(&server, "workspace-42", "branch-9");
    let first_session = session
        .product_sessions()
        .open_mutation(
            worth_server::WorthServerProductSessionCreationRequest::for_operation(
                "product_editor.apply",
            )
            .with_basis_digest(initial_basis.as_str())
            .with_expiry_seconds(300),
        )
        .expect("first mutation session should open");

    session
        .product_operations()
        .execute_mutation(
            worth_server::WorthServerWorthNativeProductMutationCommand::new(
                "product_editor.apply",
                apply_payload(),
            )
            .within(&first_session)
            .against_base_digest(base_digest(&initial_basis)),
        )
        .expect("first mutation should advance backend state");
    let advanced_basis = backend.basis_digest();
    let second_session = session
        .product_sessions()
        .open_mutation(
            worth_server::WorthServerProductSessionCreationRequest::for_operation(
                "product_editor.apply",
            )
            .with_basis_digest(advanced_basis.as_str())
            .with_expiry_seconds(300),
        )
        .expect("second mutation session should open");

    let denial = session
        .product_operations()
        .execute_mutation(
            worth_server::WorthServerWorthNativeProductMutationCommand::new(
                "product_editor.apply",
                apply_payload(),
            )
            .within(&second_session)
            .against_base_digest(base_digest(&initial_basis)),
        )
        .expect_err("stale basis should deny after real progress");

    assert_eq!(
        denial.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::PreconditionDenied
    );
    assert_eq!(
        denial
            .facts()
            .and_then(|facts| facts.stale_basis_denial())
            .map(|denial| denial.expected_base_digest()),
        Some(initial_basis.as_str())
    );
    assert_eq!(
        denial
            .facts()
            .and_then(|facts| facts.stale_basis_denial())
            .map(|denial| denial.observed_base_digest()),
        Some(advanced_basis.as_str())
    );
    assert_eq!(
        denial
            .facts()
            .and_then(|facts| facts.rebase_required())
            .map(|rebase| rebase.observed_base_digest()),
        Some(advanced_basis.as_str())
    );
    assert_eq!(backend.revision(), 1);
}

#[test]
fn failed_product_operation_is_recorded_for_replay_after_scheduler_admission() {
    let backend = worth_testkit_server::shared_support_product_operation_phase_eleven_backend::StatefulProductEditorBackend::new();
    let initial_basis = backend.basis_digest();
    let server = build_server(vec![stateful_editor_registration(backend.clone())]);
    let session = direct_session(&server, "workspace-42", "branch-9");
    let mutation_session = session
        .product_sessions()
        .open_mutation(
            worth_server::WorthServerProductSessionCreationRequest::for_operation(
                "product_editor.apply",
            )
            .with_basis_digest(initial_basis.as_str())
            .with_expiry_seconds(300),
        )
        .expect("mutation session should open");
    let idempotency_key =
        worth_server::WorthServerProductIdempotencyKey::new("failed-apply-attempt-1")
            .expect("valid key");

    let first = session
        .product_operations()
        .execute_mutation(
            worth_server::WorthServerWorthNativeProductMutationCommand::new(
                "product_editor.apply",
                controlled_apply_payload("Rename", true),
            )
            .within(&mutation_session)
            .against_base_digest(base_digest(&initial_basis))
            .idempotent(idempotency_key.clone()),
        )
        .expect("failed operation should still complete with failure envelope");
    let replayed = session
        .product_operations()
        .execute_mutation(
            worth_server::WorthServerWorthNativeProductMutationCommand::new(
                "product_editor.apply",
                controlled_apply_payload("Rename", true),
            )
            .within(&mutation_session)
            .against_base_digest(base_digest(&initial_basis))
            .idempotent(idempotency_key),
        )
        .expect("failed operation should replay");

    assert!(matches!(
        first.outcome(),
        worth_server::WorthServerProductOperationOutcome::Failed(_)
    ));
    assert!(matches!(
        replayed.outcome(),
        worth_server::WorthServerProductOperationOutcome::Failed(_)
    ));
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
    assert_eq!(backend.revision(), 0);
    assert_eq!(backend.basis_digest(), initial_basis);
}
