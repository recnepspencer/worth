pub(crate) use {
    worth_server::WorthServer, worth_server::WorthServerDirectDeclaration,
    worth_server::WorthServerDirectViewShape, worth_server::WorthServerQueryHandoffInput,
    worth_server::WorthServerQueryHandoffOperation, worth_server::WorthServerResponseInput,
    worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass,
};

pub(crate) use {
    crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest as Output,
    worth_testkit_server::shared_support_query_handoff_fixture::admit_read_posture,
    worth_testkit_server::shared_support_query_handoff_fixture::request_input,
    worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context,
    worth_testkit_server::shared_support_query_handoff_fixture::success,
    worth_testkit_server::shared_support_worth_native_assertions::admitted_named_read,
    worth_testkit_server::shared_support_worth_native_assertions::family_contract_digest,
};

pub(crate) use crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle;

pub(crate) use {crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::direct_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::direct_read_denied, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::direct_read_success, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::request_context_digest, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::support_posture_digest, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::worth_native_session_for_branch};

pub(crate) fn product_read_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    product_read_bundle_with_branch_and_shape(
        server,
        operation_name,
        None,
        worth_server::WorthServerDirectViewShape::Detail,
    )
}

pub(crate) fn branch_product_read_bundle(
    server: &WorthServer,
    operation_name: &str,
    branch_id: &str,
) -> WorthServerCertificationBundle {
    product_read_bundle_with_branch_and_shape(
        server,
        operation_name,
        Some(branch_id),
        worth_server::WorthServerDirectViewShape::Detail,
    )
}

pub(crate) fn view_shape_product_read_bundle(
    server: &WorthServer,
    operation_name: &str,
    view_shape: WorthServerDirectViewShape,
) -> WorthServerCertificationBundle {
    product_read_bundle_with_branch_and_shape(server, operation_name, None, view_shape)
}

pub(crate) fn lower_direct_read_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let read = direct_read_success(session.direct().read(&declaration));
    direct_bundle(
        server,
        session.resolved_request_context().request_context(),
        read.response_envelope().clone(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract,
        declaration.query_family_contract().contract_digest(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration, declaration.declaration_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeclarationSupport,
        declaration.support_snapshot().support_posture_digest(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff, read.handoff_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture,
        support_posture_digest(read.support_posture()),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch, read.direct_context().branch_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace, read.direct_context().workspace_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportMatrix,
        declaration.support_snapshot().support_matrix_digest(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::ViewShape,
        format!("{:?}", declaration.support_snapshot().view_shape()),
    )
}

pub(crate) fn compatibility_overlap_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let direct_session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&direct_session, operation_name);
    let request_context = resolve_request_context(
        server,
        request_input(
            worth_server::WorthServerSurfaceFamily::CompatHttp,
            worth_server::WorthServerTransportClass::CompatHttp,
        ),
    );
    let handoff = success(server.query_handoff().prepare(
        worth_server::WorthServerQueryHandoffInput::new(
            admit_read_posture(server, request_context),
            worth_server::WorthServerQueryHandoffOperation::query_read(operation_name),
        ),
    ));
    let request_context_digest = request_context_digest(handoff.admission().request_context());
    let surface_contract_digest = family_contract_digest(handoff.support_posture()).to_string();
    let support_posture_digest = support_posture_digest(handoff.support_posture());
    let branch_digest = handoff
        .admission()
        .request_context()
        .branch_target()
        .branch_digest();
    let workspace_digest = handoff
        .admission()
        .request_context()
        .workspace_target()
        .workspace_digest();
    let response = server.responses().shape_with_defaults(
        worth_server::WorthServerResponseInput::query_handoff_success(handoff),
    );
    crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest,
        response.clone(),
        worth_testkit_server::shared_support_worth_native_assertions::operator_evidence_record(server, response),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract, surface_contract_digest)
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration, declaration.declaration_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeclarationSupport,
        declaration.support_snapshot().support_posture_digest(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture, support_posture_digest)
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch, branch_digest)
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace, workspace_digest)
}

pub(crate) fn retained_artifact_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let product = session
        .direct()
        .product()
        .named_read(operation_name)
        .expect("missing operation should still admit declaration shape");
    let denial = direct_read_denied(product.read());
    let response = server
        .responses()
        .shape_with_defaults(worth_server::WorthServerResponseInput::query_handoff_denied(denial));
    crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest(session.resolved_request_context().request_context()),
        response.clone(),
        worth_testkit_server::shared_support_worth_native_assertions::operator_evidence_record(server, response),
    )
}

pub(crate) fn saved_query_intake_denial() -> worth_server::WorthServerDirectDeclarationDenial {
    let server = crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::standard_server();
    let session = worth_native_session_for_branch(&server, None);
    session
        .direct()
        .product()
        .read(worth_server::WorthServerDirectDeclaration::saved_query(
            "users.profile.saved",
        ))
        .expect_err("saved query should remain denied at declaration intake")
}

pub(crate) fn product_read_bundle_with_branch_and_shape(
    server: &WorthServer,
    operation_name: &str,
    branch_id: Option<&str>,
    view_shape: WorthServerDirectViewShape,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, branch_id);
    let product = session
        .direct()
        .product()
        .read(
            worth_server::WorthServerDirectDeclaration::named_read(operation_name)
                .with_view_shape(view_shape),
        )
        .expect("product read should admit");
    let read = direct_read_success(product.read());
    let snapshot = product.declaration_snapshot();
    direct_bundle(
        server,
        session.resolved_request_context().request_context(),
        read.response_envelope().clone(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract, snapshot.family_contract_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration, snapshot.declaration_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeclarationSupport,
        snapshot.support_posture_digest(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff, read.handoff_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture,
        support_posture_digest(read.support_posture()),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch, read.direct_context().branch_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace, read.direct_context().workspace_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportMatrix, snapshot.support_matrix_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::ViewShape, format!("{:?}", snapshot.view_shape()))
}
