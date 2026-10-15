use worth_server::{
    WorthServer, WorthServerDirectDeclaration, WorthServerDirectViewShape,
    WorthServerQueryHandoffInput, WorthServerQueryHandoffOperation, WorthServerResponseInput,
    WorthServerSurfaceFamily, WorthServerTransportClass,
};

use {
    crate::certification_suite::certification_bundle::WorthServerCertificationOutputDigest as Output,
    crate::shared_support_query_handoff_fixture::{
        admit_read_posture, request_input, resolve_request_context, success,
    },
    crate::shared_support_worth_native_assertions::{admitted_named_read, family_contract_digest},
};

use crate::certification_suite::certification_bundle::WorthServerCertificationBundle;

use super::worth_native_common::{
    direct_bundle, direct_read_denied, direct_read_success, request_context_digest,
    support_posture_digest, worth_native_session_for_branch,
};

pub fn product_read_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    product_read_bundle_with_branch_and_shape(
        server,
        operation_name,
        None,
        WorthServerDirectViewShape::Detail,
    )
}

pub fn branch_product_read_bundle(
    server: &WorthServer,
    operation_name: &str,
    branch_id: &str,
) -> WorthServerCertificationBundle {
    product_read_bundle_with_branch_and_shape(
        server,
        operation_name,
        Some(branch_id),
        WorthServerDirectViewShape::Detail,
    )
}

pub fn view_shape_product_read_bundle(
    server: &WorthServer,
    operation_name: &str,
    view_shape: WorthServerDirectViewShape,
) -> WorthServerCertificationBundle {
    product_read_bundle_with_branch_and_shape(server, operation_name, None, view_shape)
}

pub fn lower_direct_read_bundle(
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
        Output::SurfaceContract,
        declaration.query_family_contract().contract_digest(),
    )
    .with_output_digest(Output::Declaration, declaration.declaration_digest())
    .with_output_digest(
        Output::DeclarationSupport,
        declaration.support_snapshot().support_posture_digest(),
    )
    .with_output_digest(Output::Handoff, read.handoff_digest())
    .with_output_digest(
        Output::SupportPosture,
        support_posture_digest(read.support_posture()),
    )
    .with_output_digest(Output::Branch, read.direct_context().branch_digest())
    .with_output_digest(Output::Workspace, read.direct_context().workspace_digest())
    .with_output_digest(
        Output::SupportMatrix,
        declaration.support_snapshot().support_matrix_digest(),
    )
    .with_output_digest(
        Output::ViewShape,
        format!("{:?}", declaration.support_snapshot().view_shape()),
    )
}

pub fn compatibility_overlap_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let direct_session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&direct_session, operation_name);
    let request_context = resolve_request_context(
        server,
        request_input(
            WorthServerSurfaceFamily::CompatHttp,
            WorthServerTransportClass::CompatHttp,
        ),
    );
    let handoff = success(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admit_read_posture(server, request_context),
                WorthServerQueryHandoffOperation::query_read(operation_name),
            )),
    );
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
    let response = server
        .responses()
        .shape_with_defaults(WorthServerResponseInput::query_handoff_success(handoff));
    WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest,
        response.clone(),
        crate::shared_support_worth_native_assertions::operator_evidence_record(server, response),
    )
    .with_output_digest(Output::SurfaceContract, surface_contract_digest)
    .with_output_digest(Output::Declaration, declaration.declaration_digest())
    .with_output_digest(
        Output::DeclarationSupport,
        declaration.support_snapshot().support_posture_digest(),
    )
    .with_output_digest(Output::SupportPosture, support_posture_digest)
    .with_output_digest(Output::Branch, branch_digest)
    .with_output_digest(Output::Workspace, workspace_digest)
}

pub fn retained_artifact_denial_bundle(
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
        .shape_with_defaults(WorthServerResponseInput::query_handoff_denied(denial));
    WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest(session.resolved_request_context().request_context()),
        response.clone(),
        crate::shared_support_worth_native_assertions::operator_evidence_record(server, response),
    )
}

pub fn saved_query_intake_denial() -> worth_server::WorthServerDirectDeclarationDenial {
    let server = super::worth_native_common::standard_server();
    let session = worth_native_session_for_branch(&server, None);
    session
        .direct()
        .product()
        .read(WorthServerDirectDeclaration::saved_query(
            "users.profile.saved",
        ))
        .expect_err("saved query should remain denied at declaration intake")
}

fn product_read_bundle_with_branch_and_shape(
    server: &WorthServer,
    operation_name: &str,
    branch_id: Option<&str>,
    view_shape: WorthServerDirectViewShape,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, branch_id);
    let product = session
        .direct()
        .product()
        .read(WorthServerDirectDeclaration::named_read(operation_name).with_view_shape(view_shape))
        .expect("product read should admit");
    let read = direct_read_success(product.read());
    let snapshot = product.declaration_snapshot();
    direct_bundle(
        server,
        session.resolved_request_context().request_context(),
        read.response_envelope().clone(),
    )
    .with_output_digest(Output::SurfaceContract, snapshot.family_contract_digest())
    .with_output_digest(Output::Declaration, snapshot.declaration_digest())
    .with_output_digest(
        Output::DeclarationSupport,
        snapshot.support_posture_digest(),
    )
    .with_output_digest(Output::Handoff, read.handoff_digest())
    .with_output_digest(
        Output::SupportPosture,
        support_posture_digest(read.support_posture()),
    )
    .with_output_digest(Output::Branch, read.direct_context().branch_digest())
    .with_output_digest(Output::Workspace, read.direct_context().workspace_digest())
    .with_output_digest(Output::SupportMatrix, snapshot.support_matrix_digest())
    .with_output_digest(Output::ViewShape, format!("{:?}", snapshot.view_shape()))
}
