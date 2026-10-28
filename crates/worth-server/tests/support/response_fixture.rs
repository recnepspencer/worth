use worth_proof::{TransitionOutcome, TransitionReadiness};
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerConfig, WorthServerDirectDeliveryClass, WorthServerDirectFreshnessMode,
    WorthServerMiddlewareConfig, WorthServerOperationAdmissionPosture,
    WorthServerOperationAuthorityMetadata, WorthServerOperationFamily,
    WorthServerOperationRequestInput, WorthServerOperatorEvidenceConfig,
    WorthServerQueryHandoffConfig, WorthServerQueryHandoffDenial, WorthServerQueryHandoffInput,
    WorthServerQueryHandoffOperation, WorthServerQueryHandoffOutcome,
    WorthServerQueryWorkspaceProvider, WorthServerRequestContextConfig,
    WorthServerRequestContextDenial, WorthServerRequestContextInput,
    WorthServerResolvedRequestContext, WorthServerResponseConfig, WorthServerResponseInput,
    WorthServerResponseTransform, WorthServerSurfaceFamily, WorthServerTransportClass,
};

pub(crate) fn test_server(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    middleware_config: WorthServerMiddlewareConfig,
) -> WorthServer {
    test_server_with_response_config(
        workspace_provider,
        middleware_config,
        WorthServerResponseConfig::builder()
            .build()
            .expect("response config should validate"),
    )
}

pub(crate) fn test_server_with_response_config(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    middleware_config: WorthServerMiddlewareConfig,
    response_config: WorthServerResponseConfig,
) -> WorthServer {
    WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8091).into())
                .with_middleware_config(middleware_config)
                .with_request_context_config(
                    WorthServerRequestContextConfig::builder()
                        .with_default_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_query_handoff_config(
                    WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(workspace_provider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .with_response_config(response_config)
                .with_operator_evidence_config(
                    WorthServerOperatorEvidenceConfig::builder()
                        .build()
                        .expect("operator evidence config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_surface(WorthNativeSurface::disabled())
        .register_surface(CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

pub(crate) fn test_server_with_response_and_operator_evidence_config(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    middleware_config: WorthServerMiddlewareConfig,
    response_config: WorthServerResponseConfig,
    operator_evidence_config: WorthServerOperatorEvidenceConfig,
) -> WorthServer {
    test_server_with_request_context_and_operator_evidence_config(
        workspace_provider,
        middleware_config,
        WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(DiagnosticRichnessProfile::Standard)
            .build()
            .expect("request context config should validate"),
        response_config,
        operator_evidence_config,
    )
}

pub(crate) fn test_server_with_request_context_and_operator_evidence_config(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    middleware_config: WorthServerMiddlewareConfig,
    request_context_config: WorthServerRequestContextConfig,
    response_config: WorthServerResponseConfig,
    operator_evidence_config: WorthServerOperatorEvidenceConfig,
) -> WorthServer {
    WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8091).into())
                .with_middleware_config(middleware_config)
                .with_request_context_config(request_context_config)
                .with_query_handoff_config(
                    WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(workspace_provider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .with_response_config(response_config)
                .with_operator_evidence_config(operator_evidence_config)
                .build()
                .expect("server config should validate"),
        )
        .register_surface(WorthNativeSurface::disabled())
        .register_surface(CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

pub(crate) fn request_input(
    surface_family: WorthServerSurfaceFamily,
    transport_class: WorthServerTransportClass,
) -> WorthServerRequestContextInput {
    WorthServerRequestContextInput::builder()
        .with_surface_family(surface_family)
        .with_transport_class(transport_class)
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .build()
        .expect("request input should validate")
}

pub(crate) fn resolve_ready(
    server: &WorthServer,
    input: WorthServerRequestContextInput,
) -> WorthServerResolvedRequestContext {
    match server.request_contexts().resolve(input) {
        TransitionReadiness::Ready(resolved) => resolved,
        other => panic!("expected resolved request context, got {other:?}"),
    }
}

pub(crate) fn resolve_preview_denial(server: &WorthServer) -> WorthServerRequestContextDenial {
    let input = WorthServerRequestContextInput::builder()
        .with_surface_family(WorthServerSurfaceFamily::CompatHttp)
        .with_transport_class(WorthServerTransportClass::CompatHttp)
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_preview_id("preview-9")
        .build()
        .expect("preview request should validate");
    match server.request_contexts().resolve(input) {
        TransitionReadiness::Denied(denial) => denial,
        other => panic!("expected request context denial, got {other:?}"),
    }
}

pub(crate) fn resolve_blank_principal_denial(
    server: &WorthServer,
) -> WorthServerRequestContextDenial {
    let input = WorthServerRequestContextInput::builder()
        .with_surface_family(WorthServerSurfaceFamily::CompatHttp)
        .with_transport_class(WorthServerTransportClass::CompatHttp)
        .with_authenticated_principal_id("   ")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .build()
        .expect("blank principal request should validate");
    match server.request_contexts().resolve(input) {
        TransitionReadiness::Denied(denial) => denial,
        other => panic!("expected request context denial, got {other:?}"),
    }
}

pub(crate) fn resolve_blank_workspace_denial(
    server: &WorthServer,
) -> WorthServerRequestContextDenial {
    let input = WorthServerRequestContextInput::builder()
        .with_surface_family(WorthServerSurfaceFamily::CompatHttp)
        .with_transport_class(WorthServerTransportClass::CompatHttp)
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("   ")
        .build()
        .expect("blank workspace request should validate");
    match server.request_contexts().resolve(input) {
        TransitionReadiness::Denied(denial) => denial,
        other => panic!("expected request context denial, got {other:?}"),
    }
}

pub(crate) fn admit_read(
    server: &WorthServer,
    resolved: WorthServerResolvedRequestContext,
) -> worth_server::WorthServerAdmission {
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            resolved,
            worth_server::WorthServerPipelineIntent::query_read("users.profile"),
        )) {
        TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted read, got {other:?}"),
    }
}

pub(crate) fn admit_mutation(
    server: &WorthServer,
    resolved: WorthServerResolvedRequestContext,
) -> worth_server::WorthServerAdmission {
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            resolved,
            worth_server::WorthServerPipelineIntent::query_mutation("users.rename"),
        )) {
        TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted mutation, got {other:?}"),
    }
}

pub(crate) fn admit_read_posture(
    server: &WorthServer,
    resolved: WorthServerResolvedRequestContext,
) -> WorthServerOperationAdmissionPosture {
    let admission = admit_read(server, resolved);
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectRead)
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
            WorthServerOperationAuthorityMetadata::shared_read(
                "query-shared-read-basis",
                "basis-users-profile",
                "users.profile",
            ),
        )
        .expect("read operation admission should admit")
}

pub(crate) fn admit_mutation_posture(
    server: &WorthServer,
    resolved: WorthServerResolvedRequestContext,
) -> WorthServerOperationAdmissionPosture {
    let admission = admit_mutation(server, resolved);
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectSubmission)
                .with_operation_name("users.rename")
                .with_idempotency_key("idem-7")
                .build(),
        )
        .expect("mutation operation request should admit");
    server
        .operation_admissions()
        .admit(
            &admission,
            &operation_request,
            WorthServerOperationAuthorityMetadata::deterministic_submission(
                "query-write",
                "query-write-review",
                "caller-basis-unbound",
                "idempotent",
            ),
        )
        .expect("mutation operation admission should admit")
}

pub(crate) fn middleware_mutation_denial(server: &WorthServer) -> worth_server::WorthServerDenial {
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            resolve_ready(
                server,
                request_input(
                    WorthServerSurfaceFamily::CompatHttp,
                    WorthServerTransportClass::CompatHttp,
                ),
            ),
            worth_server::WorthServerPipelineIntent::query_mutation("users.rename"),
        )) {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected middleware denial, got {other:?}"),
    }
}

pub(crate) fn query_handoff_success(
    server: &WorthServer,
    surface_family: WorthServerSurfaceFamily,
    transport_class: WorthServerTransportClass,
    operation: WorthServerQueryHandoffOperation,
) -> worth_server::WorthServerQueryHandoff {
    let resolved = resolve_ready(server, request_input(surface_family, transport_class));
    let admission = match &operation {
        WorthServerQueryHandoffOperation::QueryRead { .. }
        | WorthServerQueryHandoffOperation::DownstreamDelivery { .. } => {
            admit_read_posture(server, resolved)
        }
        WorthServerQueryHandoffOperation::QueryMutation { .. } => {
            admit_mutation_posture(server, resolved)
        }
        WorthServerQueryHandoffOperation::DirectRead { .. }
        | WorthServerQueryHandoffOperation::DirectState { .. }
        | WorthServerQueryHandoffOperation::DirectInspection { .. }
        | WorthServerQueryHandoffOperation::DirectProjection { .. }
        | WorthServerQueryHandoffOperation::DirectMutation { .. } => {
            panic!("response fixture does not construct direct Worth-native query handoffs")
        }
    };
    match server
        .query_handoff()
        .prepare(WorthServerQueryHandoffInput::new(admission, operation))
    {
        TransitionOutcome::Success(handoff) => handoff,
        other => panic!("expected query handoff success, got {other:?}"),
    }
}

pub(crate) fn query_handoff_durable_denial(server: &WorthServer) -> WorthServerQueryHandoffDenial {
    let outcome: WorthServerQueryHandoffOutcome =
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admit_read_posture(
                    server,
                    resolve_ready(
                        server,
                        request_input(
                            WorthServerSurfaceFamily::CompatHttp,
                            WorthServerTransportClass::CompatHttp,
                        ),
                    ),
                ),
                WorthServerQueryHandoffOperation::downstream_delivery(
                    "users.profile",
                    WorthServerDirectFreshnessMode::LiveStrict,
                    WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                    worth_server::WorthServerQueryRequestedResume::durable(),
                ),
            ));
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected durable resume denial, got {other:?}"),
    }
}

pub(crate) fn shape_success(
    server: &WorthServer,
    handoff: worth_server::WorthServerQueryHandoff,
    transform: WorthServerResponseTransform,
) -> worth_server::WorthServerResponseEnvelope {
    server.responses().shape(
        WorthServerResponseInput::query_handoff_success(handoff),
        transform,
    )
}

pub(crate) fn operator_evidence_record(
    server: &WorthServer,
    response: worth_server::WorthServerResponseEnvelope,
) -> worth_server::WorthServerOperatorEvidenceRecord {
    server
        .operator_evidence()
        .record(
            worth_server::WorthServerEvidenceInput::response_envelope(response),
            worth_server::WorthServerEvidenceTransform::operator_default(),
        )
        .expect("operator evidence record should materialize")
}
