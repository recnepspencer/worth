pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_server::WorthServer, worth_server::WorthServerDirectDeliveryClass, worth_server::WorthServerDirectDeliveryRequest, worth_server::WorthServerDirectFreshnessMode, worth_server::WorthServerQueryHandoffDenial, worth_server::WorthServerQueryHandoffInput, worth_server::WorthServerQueryHandoffOperation, worth_server::WorthServerQueryRequestedResume, worth_server::WorthServerResponseInput, worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass};

pub(crate) use {crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle, crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest as Output, worth_testkit_server::shared_support_query_handoff_fixture::admit_delivery_posture, worth_testkit_server::shared_support_query_handoff_fixture::request_input, worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context, worth_testkit_server::shared_support_query_handoff_fixture::success, worth_testkit_server::shared_support_worth_native_assertions::admitted_named_read, worth_testkit_server::shared_support_worth_native_assertions::operator_evidence_record};

pub(crate) use {crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::direct_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::direct_delivery_denied, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::direct_delivery_success, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::direct_lease_success, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::request_context_digest, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::support_posture_digest, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::worth_native_session_for_branch, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::worth_native_session_for_target};

pub(crate) fn runtime_backed_delivery_bundle(
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
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract,
        delivery
            .downstream_delivery_contract()
            .contract_for_reporting(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration, lease.declaration_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff, delivery.handoff_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture,
        support_posture_digest(delivery.support_posture()),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch, delivery.direct_context().branch_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace,
        delivery.direct_context().workspace_digest(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis, lease.resume_basis_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeliveryRequest, delivery.request().request_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::ResumeMode, resume_mode_label(delivery.request()))
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::FreshnessMode,
        delivery.request().freshness_mode().as_str(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeliveryClass,
        delivery.request().delivery_class().as_str(),
    )
}

pub(crate) fn compatibility_runtime_backed_delivery_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let requested_resume =
        worth_server::WorthServerQueryRequestedResume::runtime_backed(Some(lease.resume_basis_digest()));
    let request = delivery_request(requested_resume.clone());
    let request_context = resolve_request_context(
        server,
        request_input(
            worth_server::WorthServerSurfaceFamily::CompatHttp,
            worth_server::WorthServerTransportClass::CompatHttp,
        ),
    );
    let request_context_digest_value = request_context_digest(request_context.request_context());
    let handoff = success(
        server
            .query_handoff()
            .prepare(worth_server::WorthServerQueryHandoffInput::new(
                admit_delivery_posture(server, request_context, lease.resume_basis_digest()),
                worth_server::WorthServerQueryHandoffOperation::downstream_delivery(
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
        .shape_with_defaults(worth_server::WorthServerResponseInput::query_handoff_success(handoff));
    crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest_value,
        response.clone(),
        operator_evidence_record(server, response),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract, surface_contract)
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration, declaration.declaration_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff, handoff_digest)
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture, support_posture)
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch, branch_digest)
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace, workspace_digest)
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis, lease.resume_basis_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeliveryRequest, request.request_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::ResumeMode, resume_mode_label(&request))
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::FreshnessMode, request.freshness_mode().as_str())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeliveryClass, request.delivery_class().as_str())
}

pub(crate) fn durable_delivery_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = delivery_request(worth_server::WorthServerQueryRequestedResume::durable());
    let denial = direct_delivery_denied(session.direct().negotiate_delivery(&lease, &request));
    denial_bundle(
        server,
        session.resolved_request_context().request_context(),
        denial,
        &lease,
        &request,
    )
}

pub(crate) fn compatibility_durable_delivery_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = delivery_request(worth_server::WorthServerQueryRequestedResume::durable());
    let request_context = resolve_request_context(
        server,
        request_input(
            worth_server::WorthServerSurfaceFamily::CompatHttp,
            worth_server::WorthServerTransportClass::CompatHttp,
        ),
    );
    let durable_request_context = request_context.request_context().clone();
    let denial = compatibility_delivery_denied(
        server,
        worth_server::WorthServerQueryHandoffInput::new(
            admit_delivery_posture(server, request_context, lease.resume_basis_digest()),
            worth_server::WorthServerQueryHandoffOperation::downstream_delivery(
                operation_name,
                request.freshness_mode(),
                request.delivery_class(),
                worth_server::WorthServerQueryRequestedResume::durable(),
            ),
        ),
    );
    denial_bundle(server, &durable_request_context, denial, &lease, &request)
}

pub(crate) fn runtime_backed_missing_basis_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = delivery_request(worth_server::WorthServerQueryRequestedResume::runtime_backed(
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

pub(crate) fn runtime_backed_stale_basis_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = delivery_request(worth_server::WorthServerQueryRequestedResume::runtime_backed(Some(
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

pub(crate) fn cross_workspace_lease_reuse_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let main_session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&main_session, operation_name);
    let lease = direct_lease_success(main_session.direct().declare_lease(&declaration));
    let hostile_session = worth_native_session_for_target(server, Some("workspace-84"), None);
    let request = delivery_request(worth_server::WorthServerQueryRequestedResume::none());
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

pub(crate) fn cross_branch_lease_reuse_denial_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let main_session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&main_session, operation_name);
    let lease = direct_lease_success(main_session.direct().declare_lease(&declaration));
    let hostile_session = worth_native_session_for_target(server, None, Some("branch-9"));
    let request = delivery_request(worth_server::WorthServerQueryRequestedResume::none());
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

pub(crate) fn denial_bundle(
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
        .shape_with_defaults(worth_server::WorthServerResponseInput::query_handoff_denied(denial));
    crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest(request_context),
        response.clone(),
        operator_evidence_record(server, response),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration, lease.declaration_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis, lease.resume_basis_digest())
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch,
        request_context.branch_target().branch_digest(),
    )
    .with_output_digest(
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace,
        request_context.workspace_target().workspace_digest(),
    )
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeliveryRequest, request.request_digest())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::ResumeMode, resume_mode_label(request))
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::FreshnessMode, request.freshness_mode().as_str())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeliveryClass, request.delivery_class().as_str())
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialCode, denial_code)
    .with_output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialDetail, denial_detail)
}

pub(crate) fn delivery_request(
    requested_resume: WorthServerQueryRequestedResume,
) -> WorthServerDirectDeliveryRequest {
    worth_server::WorthServerDirectDeliveryRequest::new(
        worth_server::WorthServerDirectFreshnessMode::LiveStrict,
        worth_server::WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        requested_resume,
    )
}

pub(crate) fn runtime_backed_request(basis_digest: &str) -> WorthServerDirectDeliveryRequest {
    delivery_request(worth_server::WorthServerQueryRequestedResume::runtime_backed(Some(
        basis_digest.to_string(),
    )))
}

pub(crate) fn resume_mode_label(request: &WorthServerDirectDeliveryRequest) -> &'static str {
    match request.requested_resume().kind() {
        worth_server::WorthServerQueryRequestedResumeKind::None => "None",
        worth_server::WorthServerQueryRequestedResumeKind::RuntimeBacked => "RuntimeBacked",
        worth_server::WorthServerQueryRequestedResumeKind::Durable => "Durable",
    }
}

pub(crate) fn compatibility_delivery_denied(
    server: &WorthServer,
    input: WorthServerQueryHandoffInput,
) -> WorthServerQueryHandoffDenial {
    match server.query_handoff().prepare(input) {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied compatibility downstream delivery, got {other:?}"),
    }
}