pub(crate) use {worth_server::WorthServer};

pub(crate) use {worth_testkit_server::shared_support_worth_native_assertions::admitted_named_read};

pub(crate) use {crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle, crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest as Output};

pub(crate) use {crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::direct_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::direct_projection_success, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::direct_retained_posture_success, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::direct_state_success, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::projection_request, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::support_posture_digest, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::worth_native_session_for_branch};

pub(crate) fn product_retained_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let product = session
        .direct()
        .product()
        .named_read(operation_name)
        .expect("retained posture should admit");
    let retained = direct_retained_posture_success(product.product_retained_posture());
    let state = direct_state_success(product.state());
    direct_bundle(
        server,
        session.resolved_request_context().request_context(),
        state.response_envelope().clone(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract,
        product.declaration_snapshot().family_contract_digest(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration,
        product.declaration_snapshot().declaration_digest(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff, state.handoff_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture,
        support_posture_digest(retained.support_posture()),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch, retained.direct_context().branch_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace,
        retained.direct_context().workspace_digest(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::RetainedState,
        retained
            .runtime_state()
            .state_digest()
            .terminal_projection_for_reporting(),
    )
    .with_optional_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis, retained.basis_digest())
    .with_optional_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Remask, retained.remask_posture().remask_digest())
    .with_optional_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::AsyncResult,
        retained
            .async_result_state()
            .map(|state| state.inner().result_state_for_reporting()),
    )
    .with_optional_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::TemporalState,
        retained.temporal_state().map(|state| {
            state
                .inner()
                .state_digest()
                .terminal_projection_for_reporting()
        }),
    )
}

pub(crate) fn lower_direct_state_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let state = direct_state_success(session.direct().state(&declaration));
    direct_bundle(
        server,
        session.resolved_request_context().request_context(),
        state.response_envelope().clone(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract,
        declaration.query_family_contract().contract_digest(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration, declaration.declaration_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff, state.handoff_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture,
        support_posture_digest(state.support_posture()),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch, state.direct_context().branch_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace, state.direct_context().workspace_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::RetainedState,
        state
            .runtime_state()
            .state_digest()
            .terminal_projection_for_reporting(),
    )
    .with_optional_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis, state.direct_context().basis_digest())
    .with_optional_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Remask,
        state.direct_context().remask_posture().remask_digest(),
    )
    .with_optional_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::AsyncResult,
        state
            .async_result_state()
            .as_ref()
            .map(|value| value.inner().result_state_for_reporting()),
    )
    .with_optional_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::TemporalState,
        state.temporal_state().as_ref().map(|value| {
            value
                .inner()
                .state_digest()
                .terminal_projection_for_reporting()
        }),
    )
}

pub(crate) fn product_projection_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let product = session
        .direct()
        .product()
        .named_read(operation_name)
        .expect("projection should admit");
    let projection = direct_projection_success(product.project(&projection_request()));
    direct_bundle(
        server,
        session.resolved_request_context().request_context(),
        projection.response_envelope().clone(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract,
        product.declaration_snapshot().family_contract_digest(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration,
        product.declaration_snapshot().declaration_digest(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff, projection.handoff_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture,
        support_posture_digest(projection.support_posture()),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch, projection.direct_context().branch_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace,
        projection.direct_context().workspace_digest(),
    )
    .with_optional_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis, projection.basis_digest())
    .with_optional_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Remask,
        projection.direct_context().remask_posture().remask_digest(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Policy, projection.policy_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::FactReceipt,
        projection.fact_receipt().receipt_digest(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Materialization,
        projection.materialization_digest().as_str(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::CounterSnapshot,
        projection.fact_receipt().counter_snapshot_digest(),
    )
}

pub(crate) fn lower_direct_projection_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let projection = direct_projection_success(
        session
            .direct()
            .project(&declaration, &projection_request()),
    );
    direct_bundle(
        server,
        session.resolved_request_context().request_context(),
        projection.response_envelope().clone(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract,
        declaration.query_family_contract().contract_digest(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration, declaration.declaration_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff, projection.handoff_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture,
        support_posture_digest(projection.support_posture()),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch, projection.direct_context().branch_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace,
        projection.direct_context().workspace_digest(),
    )
    .with_optional_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis, projection.basis_digest())
    .with_optional_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Remask,
        projection.direct_context().remask_posture().remask_digest(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Policy, projection.policy_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::FactReceipt,
        projection.fact_receipt().receipt_digest(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Materialization,
        projection.materialization_digest().as_str(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::CounterSnapshot,
        projection.fact_receipt().counter_snapshot_digest(),
    )
}