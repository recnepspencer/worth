pub(crate) use {worth_proof::TransitionOutcome, worth_proof::TransitionReadiness};
pub(crate) use {
    worth_server::request_context::DiagnosticRichnessProfile, worth_server::WorthServer,
    worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerOperationAdmissionPosture,
    worth_server::WorthServerOperationAuthorityMetadata, worth_server::WorthServerOperationFamily,
    worth_server::WorthServerOperationRequestInput,
    worth_server::WorthServerOperatorEvidenceConfig, worth_server::WorthServerPipelineInput,
    worth_server::WorthServerPipelineIntent, worth_server::WorthServerQueryHandoffInput,
    worth_server::WorthServerQueryHandoffOperation, worth_server::WorthServerResponseConfig,
    worth_server::WorthServerResponseInput, worth_server::WorthServerResponseTransform,
    worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass,
};

pub(crate) use worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider;
pub(crate) use {
    worth_testkit_server::shared_support_response_fixture::operator_evidence_record,
    worth_testkit_server::shared_support_response_fixture::query_handoff_durable_denial,
    worth_testkit_server::shared_support_response_fixture::resolve_blank_principal_denial,
    worth_testkit_server::shared_support_response_fixture::resolve_preview_denial,
    worth_testkit_server::shared_support_response_fixture::test_server_with_request_context_and_operator_evidence_config,
};

pub(crate) use crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle;

pub(crate) fn certification_server(
    request_context_profile: DiagnosticRichnessProfile,
    response_profile: DiagnosticRichnessProfile,
    operator_profile: DiagnosticRichnessProfile,
) -> WorthServer {
    test_server_with_request_context_and_operator_evidence_config(
        TestWorkspaceProvider,
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
        worth_server::WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(request_context_profile)
            .with_maximum_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Forensic,
            )
            .with_preview_targeting_enabled(false)
            .build()
            .expect("request context config should validate"),
        worth_server::WorthServerResponseConfig::builder()
            .with_success_minimum_diagnostics_profile(response_profile)
            .with_denial_minimum_diagnostics_profile(response_profile)
            .build()
            .expect("response config should validate"),
        worth_server::WorthServerOperatorEvidenceConfig::builder()
            .with_minimum_diagnostics_profile(operator_profile)
            .build()
            .expect("operator evidence config should validate"),
    )
}

pub(crate) fn read_success_bundle(
    server: &WorthServer,
    surface_family: WorthServerSurfaceFamily,
    transport_class: WorthServerTransportClass,
    transform: WorthServerResponseTransform,
) -> WorthServerCertificationBundle {
    read_success_bundle_for_workspace(
        server,
        surface_family,
        transport_class,
        transform,
        "tenant-a",
        "workspace-42",
    )
}

pub(crate) fn read_success_bundle_for_workspace(
    server: &WorthServer,
    surface_family: WorthServerSurfaceFamily,
    transport_class: WorthServerTransportClass,
    transform: WorthServerResponseTransform,
    tenant_id: &str,
    workspace_id: &str,
) -> WorthServerCertificationBundle {
    let request_context = resolve_ready_request_context(
        server,
        surface_family,
        transport_class,
        tenant_id,
        workspace_id,
    );
    let admission = admit_query_read_posture(server, request_context);
    let response = server.responses().shape(
        worth_server::WorthServerResponseInput::query_handoff_success(prepare_query_handoff(
            server,
            admission,
            worth_server::WorthServerQueryHandoffOperation::query_read("users.profile"),
        )),
        transform,
    );
    crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest(
            "principal-7",
            tenant_id,
            workspace_id,
            "head",
            worth_server::request_context::DiagnosticRichnessProfile::Standard,
        ),
        response.clone(),
        operator_evidence_record(server, response),
    )
}

pub(crate) fn malformed_identity_bundle(server: &WorthServer) -> WorthServerCertificationBundle {
    let response = server.responses().shape(
        worth_server::WorthServerResponseInput::request_context_denied(
            resolve_blank_principal_denial(server),
        ),
        worth_server::WorthServerResponseTransform::compat_http(),
    );
    crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle::from_response_and_evidence(
        String::from("request_context:invalid_principal"),
        response.clone(),
        operator_evidence_record(server, response),
    )
}

pub(crate) fn preview_branch_denial_bundle(server: &WorthServer) -> WorthServerCertificationBundle {
    let response = server.responses().shape(
        worth_server::WorthServerResponseInput::request_context_denied(resolve_preview_denial(
            server,
        )),
        worth_server::WorthServerResponseTransform::compat_http(),
    );
    crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle::from_response_and_evidence(
        String::from("request_context:preview_disabled"),
        response.clone(),
        operator_evidence_record(server, response),
    )
}

pub(crate) fn durable_resume_denial_bundle(server: &WorthServer) -> WorthServerCertificationBundle {
    let response = server.responses().shape(
        worth_server::WorthServerResponseInput::query_handoff_denied(query_handoff_durable_denial(
            server,
        )),
        worth_server::WorthServerResponseTransform::compat_http(),
    );
    crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest(
            "principal-7",
            "tenant-a",
            "workspace-42",
            "head",
            worth_server::request_context::DiagnosticRichnessProfile::Standard,
        ),
        response.clone(),
        operator_evidence_record(server, response),
    )
}

pub(crate) fn middleware_preview_authorization_denial_bundle() -> WorthServerCertificationBundle {
    let server = test_server_with_request_context_and_operator_evidence_config(
        TestWorkspaceProvider,
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_preview_branch_authorization_enabled(false)
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
        worth_server::WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Standard,
            )
            .with_preview_targeting_enabled(true)
            .build()
            .expect("request context config should validate"),
        worth_server::WorthServerResponseConfig::builder()
            .build()
            .expect("response config should validate"),
        worth_server::WorthServerOperatorEvidenceConfig::builder()
            .build()
            .expect("operator evidence config should validate"),
    );

    let response = server.responses().shape(
        worth_server::WorthServerResponseInput::middleware_denied(preview_authorization_denial(
            &server,
        )),
        worth_server::WorthServerResponseTransform::compat_http(),
    );
    crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest(
            "principal-7",
            "tenant-a",
            "workspace-42",
            "preview-9",
            worth_server::request_context::DiagnosticRichnessProfile::Standard,
        ),
        response.clone(),
        operator_evidence_record(&server, response),
    )
}

pub(crate) fn request_context_digest(
    principal_id: &str,
    tenant_id: &str,
    workspace_id: &str,
    branch_target: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> String {
    format!(
        "principal={principal_id};tenant={tenant_id};workspace={workspace_id};branch={branch_target};diagnostics={diagnostics_profile:?}"
    )
}

pub(crate) fn resolve_ready_request_context(
    server: &WorthServer,
    surface_family: WorthServerSurfaceFamily,
    transport_class: WorthServerTransportClass,
    tenant_id: &str,
    workspace_id: &str,
) -> worth_server::WorthServerResolvedRequestContext {
    match server.request_contexts().resolve(
        worth_server::WorthServerRequestContextInput::builder()
            .with_surface_family(surface_family)
            .with_transport_class(transport_class)
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id(tenant_id)
            .with_workspace_id(workspace_id)
            .build()
            .expect("request input should validate"),
    ) {
        worth_proof::TransitionReadiness::Ready(resolved) => resolved,
        other => panic!("expected resolved request context, got {other:?}"),
    }
}

pub(crate) fn admit_query_read(
    server: &WorthServer,
    request_context: worth_server::WorthServerResolvedRequestContext,
) -> worth_server::WorthServerAdmission {
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            request_context,
            worth_server::WorthServerPipelineIntent::query_read("users.profile"),
        )) {
        worth_proof::TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted read pipeline, got {other:?}"),
    }
}

pub(crate) fn admit_query_read_posture(
    server: &WorthServer,
    request_context: worth_server::WorthServerResolvedRequestContext,
) -> WorthServerOperationAdmissionPosture {
    let admission = admit_query_read(server, request_context);
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            worth_server::WorthServerOperationRequestInput::builder()
                .with_operation_family(worth_server::WorthServerOperationFamily::QueryDirectRead)
                .with_operation_name("users.profile")
                .with_basis_digest("basis-users-profile")
                .build(),
        )
        .expect("read operation request should admit");
    server
        .operation_admissions()
        .admit(
            &admission,
            &operation_request,
            worth_server::WorthServerOperationAuthorityMetadata::shared_read(
                "query-shared-read-basis",
                "basis-users-profile",
                "users.profile",
            ),
        )
        .expect("read posture should admit")
}

pub(crate) fn prepare_query_handoff(
    server: &WorthServer,
    admission: WorthServerOperationAdmissionPosture,
    operation: WorthServerQueryHandoffOperation,
) -> worth_server::WorthServerQueryHandoff {
    match server
        .query_handoff()
        .prepare(worth_server::WorthServerQueryHandoffInput::new(
            admission, operation,
        )) {
        worth_proof::TransitionOutcome::Success(handoff) => handoff,
        other => panic!("expected query handoff success, got {other:?}"),
    }
}

pub(crate) fn preview_authorization_denial(
    server: &WorthServer,
) -> worth_server::WorthServerDenial {
    let request_context = match server.request_contexts().resolve(
        worth_server::WorthServerRequestContextInput::builder()
            .with_surface_family(worth_server::WorthServerSurfaceFamily::CompatHttp)
            .with_transport_class(worth_server::WorthServerTransportClass::CompatHttp)
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_preview_id("preview-9")
            .build()
            .expect("preview request should validate"),
    ) {
        worth_proof::TransitionReadiness::Ready(resolved) => resolved,
        other => panic!("expected preview request context, got {other:?}"),
    };
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            request_context,
            worth_server::WorthServerPipelineIntent::query_read("users.profile"),
        )) {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected preview authorization denial, got {other:?}"),
    }
}
