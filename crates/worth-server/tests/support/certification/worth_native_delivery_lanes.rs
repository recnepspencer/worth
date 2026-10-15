use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServer, WorthServerDirectDeliveryClass, WorthServerDirectDeliveryRequest,
    WorthServerDirectFreshnessMode, WorthServerQueryHandoffDenial, WorthServerQueryHandoffInput,
    WorthServerQueryHandoffOperation, WorthServerQueryRequestedResume, WorthServerResponseInput,
    WorthServerSurfaceFamily, WorthServerTransportClass,
};

use {crate::certification_suite::certification_bundle::{
        WorthServerCertificationBundle, WorthServerCertificationOutputDigest as Output,
    }, crate::shared_support_query_handoff_fixture::{
        admit_delivery_posture, request_input, resolve_request_context, success,
    }, crate::shared_support_worth_native_assertions::{admitted_named_read, operator_evidence_record}};

use super::worth_native_common::{
    direct_bundle, direct_delivery_denied, direct_delivery_success, direct_lease_success,
    request_context_digest, support_posture_digest, worth_native_session_for_branch,
    worth_native_session_for_target,
};

pub fn runtime_backed_delivery_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = runtime_backed_request(lease.resume_basis_digest());
    let delivery = direct_delivery_success(session.direct().negotiate_delivery(&lease, &request));
    direct_bundle(
        server,
        session.resolved_request_context().request_context(),
        delivery.response_envelope().clone(),
    )
    .with_output_digest(
        Output::SurfaceContract,
        delivery
            .downstream_delivery_contract()
            .contract_for_reporting(),
    )
    .with_output_digest(Output::Declaration, lease.declaration_digest())
    .with_output_digest(Output::Handoff, delivery.handoff_digest())
    .with_output_digest(
        Output::SupportPosture,
        support_posture_digest(delivery.support_posture()),
    )
    .with_output_digest(Output::Branch, delivery.direct_context().branch_digest())
    .with_output_digest(
        Output::Workspace,
        delivery.direct_context().workspace_digest(),
    )
    .with_output_digest(Output::Basis, lease.resume_basis_digest())
    .with_output_digest(Output::DeliveryRequest, delivery.request().request_digest())
    .with_output_digest(Output::ResumeMode, resume_mode_label(delivery.request()))
    .with_output_digest(
        Output::FreshnessMode,
        delivery.request().freshness_mode().as_str(),
    )
    .with_output_digest(
        Output::DeliveryClass,
        delivery.request().delivery_class().as_str(),
    )
}

pub fn compatibility_runtime_backed_delivery_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let requested_resume =
        WorthServerQueryRequestedResume::runtime_backed(Some(lease.resume_basis_digest()));
    let request = delivery_request(requested_resume.clone());
    let request_context = resolve_request_context(
        server,
        request_input(
            WorthServerSurfaceFamily::CompatHttp,
            WorthServerTransportClass::CompatHttp,
        ),
    );
    let request_context_digest_value = request_context_digest(request_context.request_context());
    let handoff = success(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admit_delivery_posture(server, request_context, lease.resume_basis_digest()),
                WorthServerQueryHandoffOperation::downstream_delivery(
                    operation_name,
                    request.freshness_mode(),
                    request.delivery_class(),
                    requested_resume,
                ),
            )),
    );
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
    let handoff_digest = handoff.canonical_digest().to_string();
    let support_posture = support_posture_digest(handoff.support_posture()).to_string();
    let surface_contract = handoff
        .downstream_delivery_contract()
        .contract_for_reporting()
        .to_string();
    let response = server
        .responses()
        .shape_with_defaults(WorthServerResponseInput::query_handoff_success(handoff));
    WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest_value,
        response.clone(),
        operator_evidence_record(server, response),
    )
    .with_output_digest(Output::SurfaceContract, surface_contract)
    .with_output_digest(Output::Declaration, declaration.declaration_digest())
    .with_output_digest(Output::Handoff, handoff_digest)
    .with_output_digest(Output::SupportPosture, support_posture)
    .with_output_digest(Output::Branch, branch_digest)
    .with_output_digest(Output::Workspace, workspace_digest)
    .with_output_digest(Output::Basis, lease.resume_basis_digest())
    .with_output_digest(Output::DeliveryRequest, request.request_digest())
    .with_output_digest(Output::ResumeMode, resume_mode_label(&request))
    .with_output_digest(Output::FreshnessMode, request.freshness_mode().as_str())
    .with_output_digest(Output::DeliveryClass, request.delivery_class().as_str())
}

pub fn durable_delivery_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = delivery_request(WorthServerQueryRequestedResume::durable());
    let denial = direct_delivery_denied(session.direct().negotiate_delivery(&lease, &request));
    denial_bundle(
        server,
        session.resolved_request_context().request_context(),
        denial,
        &lease,
        &request,
    )
}

pub fn compatibility_durable_delivery_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = delivery_request(WorthServerQueryRequestedResume::durable());
    let request_context = resolve_request_context(
        server,
        request_input(
            WorthServerSurfaceFamily::CompatHttp,
            WorthServerTransportClass::CompatHttp,
        ),
    );
    let durable_request_context = request_context.request_context().clone();
    let denial = compatibility_delivery_denied(
        server,
        WorthServerQueryHandoffInput::new(
            admit_delivery_posture(server, request_context, lease.resume_basis_digest()),
            WorthServerQueryHandoffOperation::downstream_delivery(
                operation_name,
                request.freshness_mode(),
                request.delivery_class(),
                WorthServerQueryRequestedResume::durable(),
            ),
        ),
    );
    denial_bundle(server, &durable_request_context, denial, &lease, &request)
}

pub fn runtime_backed_missing_basis_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = delivery_request(WorthServerQueryRequestedResume::runtime_backed(
        None::<String>,
    ));
    let denial = direct_delivery_denied(session.direct().negotiate_delivery(&lease, &request));
    denial_bundle(
        server,
        session.resolved_request_context().request_context(),
        denial,
        &lease,
        &request,
    )
}

pub fn runtime_backed_stale_basis_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = delivery_request(WorthServerQueryRequestedResume::runtime_backed(Some(
        "basis:drifted",
    )));
    let denial = direct_delivery_denied(session.direct().negotiate_delivery(&lease, &request));
    denial_bundle(
        server,
        session.resolved_request_context().request_context(),
        denial,
        &lease,
        &request,
    )
}

pub fn cross_workspace_lease_reuse_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let main_session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&main_session, operation_name);
    let lease = direct_lease_success(main_session.direct().declare_lease(&declaration));
    let hostile_session = worth_native_session_for_target(server, Some("workspace-84"), None);
    let request = delivery_request(WorthServerQueryRequestedResume::none());
    let denial = direct_delivery_denied(
        hostile_session
            .direct()
            .negotiate_delivery(&lease, &request),
    );
    denial_bundle(
        server,
        hostile_session.resolved_request_context().request_context(),
        denial,
        &lease,
        &request,
    )
}

pub fn cross_branch_lease_reuse_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let main_session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&main_session, operation_name);
    let lease = direct_lease_success(main_session.direct().declare_lease(&declaration));
    let hostile_session = worth_native_session_for_target(server, None, Some("branch-9"));
    let request = delivery_request(WorthServerQueryRequestedResume::none());
    let denial = direct_delivery_denied(
        hostile_session
            .direct()
            .negotiate_delivery(&lease, &request),
    );
    denial_bundle(
        server,
        hostile_session.resolved_request_context().request_context(),
        denial,
        &lease,
        &request,
    )
}

fn denial_bundle(
    server: &WorthServer,
    request_context: &worth_server::WorthServerRequestContext,
    denial: WorthServerQueryHandoffDenial,
    lease: &worth_server::WorthServerDirectLeaseDeclaration,
    request: &WorthServerDirectDeliveryRequest,
) -> WorthServerCertificationBundle {
    let denial_code = format!("{:?}", denial.code());
    let denial_detail = denial.detail().to_string();
    let response = server
        .responses()
        .shape_with_defaults(WorthServerResponseInput::query_handoff_denied(denial));
    WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest(request_context),
        response.clone(),
        operator_evidence_record(server, response),
    )
    .with_output_digest(Output::Declaration, lease.declaration_digest())
    .with_output_digest(Output::Basis, lease.resume_basis_digest())
    .with_output_digest(
        Output::Branch,
        request_context.branch_target().branch_digest(),
    )
    .with_output_digest(
        Output::Workspace,
        request_context.workspace_target().workspace_digest(),
    )
    .with_output_digest(Output::DeliveryRequest, request.request_digest())
    .with_output_digest(Output::ResumeMode, resume_mode_label(request))
    .with_output_digest(Output::FreshnessMode, request.freshness_mode().as_str())
    .with_output_digest(Output::DeliveryClass, request.delivery_class().as_str())
    .with_output_digest(Output::DenialCode, denial_code)
    .with_output_digest(Output::DenialDetail, denial_detail)
}

fn delivery_request(
    requested_resume: WorthServerQueryRequestedResume,
) -> WorthServerDirectDeliveryRequest {
    WorthServerDirectDeliveryRequest::new(
        WorthServerDirectFreshnessMode::LiveStrict,
        WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        requested_resume,
    )
}

fn runtime_backed_request(basis_digest: &str) -> WorthServerDirectDeliveryRequest {
    delivery_request(WorthServerQueryRequestedResume::runtime_backed(Some(
        basis_digest.to_string(),
    )))
}

fn resume_mode_label(request: &WorthServerDirectDeliveryRequest) -> &'static str {
    match request.requested_resume().kind() {
        worth_server::WorthServerQueryRequestedResumeKind::None => "None",
        worth_server::WorthServerQueryRequestedResumeKind::RuntimeBacked => "RuntimeBacked",
        worth_server::WorthServerQueryRequestedResumeKind::Durable => "Durable",
    }
}

fn compatibility_delivery_denied(
    server: &WorthServer,
    input: WorthServerQueryHandoffInput,
) -> WorthServerQueryHandoffDenial {
    match server.query_handoff().prepare(input) {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied compatibility downstream delivery, got {other:?}"),
    }
}
