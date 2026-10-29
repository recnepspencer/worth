#![allow(dead_code)]

pub(crate) use {worth_server::WorthServerEditorLikeOperationFixture, worth_server::WorthServerProductIdempotentRetryCertificationProof, worth_server::WorthServerProductMutationCertificationProof, worth_server::WorthServerProductPressureShapeCertificationProof, worth_server::WorthServerProductSharedReadCertificationProof, worth_server::WorthServerProductStaleApplyDenialCertificationProof};

pub(crate) use {worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::actions_payload, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::apply_payload, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::build_server, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::direct_mutation, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::direct_read, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::direct_session, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::finalize_payload, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::open_mutation_session, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::render_payload, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::select_payload, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::StatefulEditorLikeBackend};


pub(crate) use {crate::product_operation_phase_thirteen_certification::ordinary_context::proof_builders::route_parity::observed_route_parity};

pub(crate) async fn complete_editor_like_fixture() -> WorthServerEditorLikeOperationFixture {
    let backend = worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::StatefulEditorLikeBackend::new();
    let server = build_server(&backend);
    let session = direct_session(&server);
    let initial_basis = backend.basis_digest();
    let shared_read_batch = session
        .product_operations()
        .execute_shared_read_batch(vec![
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.render",
                render_payload(),
            )
            .with_basis_digest(&initial_basis),
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.select",
                select_payload("node-7"),
            )
            .with_basis_digest(&initial_basis),
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.available_actions",
                actions_payload(),
            )
            .with_basis_digest(&initial_basis),
        ])
        .expect("shared read batch should complete");
    let shared_read =
        worth_server::WorthServerProductSharedReadCertificationProof::from_batch(&shared_read_batch)
            .expect("shared read proof");
    let render = direct_read(
        &session,
        "product_editor.render",
        render_payload(),
        &initial_basis,
    );
    let select = direct_read(
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
    let finalize_denied = direct_mutation(
        &session,
        "product_editor.finalize",
        finalize_payload(false),
        &initial_basis,
        &finalize_session,
    )
    .expect("finalize denial should shape a completed operation");
    let apply_session = open_mutation_session(&session, &initial_basis, "product_editor.apply");
    let applied = direct_mutation(
        &session,
        "product_editor.apply",
        apply_payload("Renamed"),
        &initial_basis,
        &apply_session,
    )
    .expect("apply should succeed");
    let after_apply_basis = backend.basis_digest();
    let actions_after = direct_read(
        &session,
        "product_editor.available_actions",
        actions_payload(),
        &after_apply_basis,
    );
    let stale = direct_mutation(
        &session,
        "product_editor.apply",
        apply_payload("Stale Rewrite"),
        &after_apply_basis,
        &apply_session,
    )
    .expect_err("stale apply should deny");
    let replay_session =
        open_mutation_session(&session, &after_apply_basis, "product_editor.apply");
    let replay_key =
        worth_server::WorthServerProductIdempotencyKey::new("phase-thirteen-cert-replay")
            .expect("idempotency key");
    let authoritative = session
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.apply",
                apply_payload("Replay Rename"),
            )
            .with_basis_digest(&after_apply_basis)
            .with_product_session_identity(replay_session.identity().as_str())
            .with_idempotency_key(replay_key.clone()),
        )
        .expect("authoritative replay write");
    let replayed = session
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.apply",
                apply_payload("Replay Rename"),
            )
            .with_basis_digest(&after_apply_basis)
            .with_product_session_identity(replay_session.identity().as_str())
            .with_idempotency_key(replay_key),
        )
        .expect("replayed write");
    let finalize_success_session =
        open_mutation_session(&session, &after_apply_basis, "product_editor.finalize");
    let finalized = direct_mutation(
        &session,
        "product_editor.finalize",
        finalize_payload(true),
        &after_apply_basis,
        &finalize_success_session,
    )
    .expect("finalize success");
    let mutation = worth_server::WorthServerProductMutationCertificationProof::new(&applied, &finalized)
        .expect("mutation proof");
    let finalize_denial_reason = match finalize_denied.outcome() {
        worth_server::WorthServerProductOperationOutcome::Denied(denial) => denial.reason_key(),
        other => panic!("expected finalize denial, got {other:?}"),
    };
    let pressure = worth_server::WorthServerProductPressureShapeCertificationProof::new(
        &render,
        &select,
        &actions_before,
        &actions_after,
        &applied,
        finalize_denial_reason,
    );
    let stale = worth_server::WorthServerProductStaleApplyDenialCertificationProof::from_denial(&stale)
        .expect("stale denial proof");
    let replay =
        worth_server::WorthServerProductIdempotentRetryCertificationProof::new(&authoritative, &replayed)
            .expect("replay proof");
    let route_parity =
        observed_route_parity(&server, &session, &initial_basis, &after_apply_basis).await;

    worth_server::WorthServerEditorLikeOperationFixture::new()
        .with_shared_read_certification(shared_read)
        .with_mutation_certification(mutation)
        .with_route_parity(route_parity)
        .with_pressure_shape(pressure)
        .with_stale_apply_denial(stale)
        .with_idempotent_retry(replay)
}

pub(crate) async fn blocked_editor_like_fixture() -> WorthServerEditorLikeOperationFixture {
    let backend = worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::StatefulEditorLikeBackend::new();
    let server = build_server(&backend);
    let session = direct_session(&server);
    let initial_basis = backend.basis_digest();
    let shared_read_batch = session
        .product_operations()
        .execute_shared_read_batch(vec![worth_server::WorthServerProductOperationInput::new(
            "product_editor.render",
            render_payload(),
        )
        .with_basis_digest(&initial_basis)])
        .expect("shared read batch should complete");
    let shared_read =
        worth_server::WorthServerProductSharedReadCertificationProof::from_batch(&shared_read_batch)
            .expect("shared read proof");
    let apply_session = open_mutation_session(&session, &initial_basis, "product_editor.apply");
    let apply = direct_mutation(
        &session,
        "product_editor.apply",
        apply_payload("Blocked"),
        &initial_basis,
        &apply_session,
    )
    .expect("apply should succeed");
    let after_apply_basis = backend.basis_digest();
    let finalize_session =
        open_mutation_session(&session, &after_apply_basis, "product_editor.finalize");
    let finalize = direct_mutation(
        &session,
        "product_editor.finalize",
        finalize_payload(true),
        &after_apply_basis,
        &finalize_session,
    )
    .expect("finalize should succeed");
    let render = direct_read(
        &session,
        "product_editor.render",
        render_payload(),
        &initial_basis,
    );
    let select = direct_read(
        &session,
        "product_editor.select",
        select_payload("node-7"),
        &initial_basis,
    );
    let actions = direct_read(
        &session,
        "product_editor.available_actions",
        actions_payload(),
        &initial_basis,
    );
    worth_server::WorthServerEditorLikeOperationFixture::new()
        .with_shared_read_certification(shared_read)
        .with_mutation_certification(
            worth_server::WorthServerProductMutationCertificationProof::new(&apply, &finalize)
                .expect("mutation proof"),
        )
        .with_pressure_shape(worth_server::WorthServerProductPressureShapeCertificationProof::new(
            &render,
            &select,
            &actions,
            &actions,
            &apply,
            "product.finalize.confirm_required",
        ))
}