pub use {worth_proof::TransitionOutcome, worth_proof::TransitionReadiness};
pub use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerConfig, worth_server::WorthServerDirectDeliveryClass, worth_server::WorthServerDirectFreshnessMode, worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerOperationAdmissionPosture, worth_server::WorthServerOperationAuthorityMetadata, worth_server::WorthServerOperationFamily, worth_server::WorthServerOperationRequestInput, worth_server::WorthServerOperatorEvidenceConfig, worth_server::WorthServerQueryHandoffConfig, worth_server::WorthServerQueryHandoffDenial, worth_server::WorthServerQueryHandoffInput, worth_server::WorthServerQueryHandoffOperation, worth_server::WorthServerQueryHandoffOutcome, worth_server::WorthServerQueryWorkspaceProvider, worth_server::WorthServerRequestContextConfig, worth_server::WorthServerRequestContextDenial, worth_server::WorthServerRequestContextInput, worth_server::WorthServerResolvedRequestContext, worth_server::WorthServerResponseConfig, worth_server::WorthServerResponseInput, worth_server::WorthServerResponseTransform, worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass};

pub fn test_server(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    middleware_config: WorthServerMiddlewareConfig,
) -> WorthServer {
    test_server_with_response_config(
        workspace_provider,
        middleware_config,
        worth_server::WorthServerResponseConfig::builder()
            .build()
            .expect("response config should validate"),
    )
}

pub fn test_server_with_response_config(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    middleware_config: WorthServerMiddlewareConfig,
    response_config: WorthServerResponseConfig,
) -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8091).into())
                .with_middleware_config(middleware_config)
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_default_diagnostics_profile(worth_server::request_context::DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(workspace_provider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .with_response_config(response_config)
                .with_operator_evidence_config(
                    worth_server::WorthServerOperatorEvidenceConfig::builder()
                        .build()
                        .expect("operator evidence config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

pub fn test_server_with_response_and_operator_evidence_config(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    middleware_config: WorthServerMiddlewareConfig,
    response_config: WorthServerResponseConfig,
    operator_evidence_config: WorthServerOperatorEvidenceConfig,
) -> WorthServer {
    test_server_with_request_context_and_operator_evidence_config(
        workspace_provider,
        middleware_config,
        worth_server::WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(worth_server::request_context::DiagnosticRichnessProfile::Standard)
            .build()
            .expect("request context config should validate"),
        response_config,
        operator_evidence_config,
    )
}

pub fn test_server_with_request_context_and_operator_evidence_config(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    middleware_config: WorthServerMiddlewareConfig,
    request_context_config: WorthServerRequestContextConfig,
    response_config: WorthServerResponseConfig,
    operator_evidence_config: WorthServerOperatorEvidenceConfig,
) -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8091).into())
                .with_middleware_config(middleware_config)
                .with_request_context_config(request_context_config)
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(workspace_provider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .with_response_config(response_config)
                .with_operator_evidence_config(operator_evidence_config)
                .build()
                .expect("server config should validate"),
        )
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

pub fn request_input(
    surface_family: WorthServerSurfaceFamily,
    transport_class: WorthServerTransportClass,
) -> WorthServerRequestContextInput {
    worth_server::WorthServerRequestContextInput::builder()
        .with_surface_family(surface_family)
        .with_transport_class(transport_class)
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .build()
        .expect("request input should validate")
}

pub fn resolve_ready(
    server: &WorthServer,
    input: WorthServerRequestContextInput,
) -> WorthServerResolvedRequestContext {
    match server.request_contexts().resolve(input) {
        worth_proof::TransitionReadiness::Ready(resolved) => resolved,
        other => panic!("expected resolved request context, got {other:?}"),
    }
}

pub fn resolve_preview_denial(server: &WorthServer) -> WorthServerRequestContextDenial {
    let input = worth_server::WorthServerRequestContextInput::builder()
        .with_surface_family(worth_server::WorthServerSurfaceFamily::CompatHttp)
        .with_transport_class(worth_server::WorthServerTransportClass::CompatHttp)
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_preview_id("preview-9")
        .build()
        .expect("preview request should validate");
    match server.request_contexts().resolve(input) {
        worth_proof::TransitionReadiness::Denied(denial) => denial,
        other => panic!("expected request context denial, got {other:?}"),
    }
}

pub fn resolve_blank_principal_denial(
    server: &WorthServer,
) -> WorthServerRequestContextDenial {
    let input = worth_server::WorthServerRequestContextInput::builder()
        .with_surface_family(worth_server::WorthServerSurfaceFamily::CompatHttp)
        .with_transport_class(worth_server::WorthServerTransportClass::CompatHttp)
        .with_authenticated_principal_id("   ")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .build()
        .expect("blank principal request should validate");
    match server.request_contexts().resolve(input) {
        worth_proof::TransitionReadiness::Denied(denial) => denial,
        other => panic!("expected request context denial, got {other:?}"),
    }
}

pub fn resolve_blank_workspace_denial(
    server: &WorthServer,
) -> WorthServerRequestContextDenial {
    let input = worth_server::WorthServerRequestContextInput::builder()
        .with_surface_family(worth_server::WorthServerSurfaceFamily::CompatHttp)
        .with_transport_class(worth_server::WorthServerTransportClass::CompatHttp)
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("   ")
        .build()
        .expect("blank workspace request should validate");
    match server.request_contexts().resolve(input) {
        worth_proof::TransitionReadiness::Denied(denial) => denial,
        other => panic!("expected request context denial, got {other:?}"),
    }
}

pub fn admit_read(
    server: &WorthServer,
    resolved: WorthServerResolvedRequestContext,
) -> worth_server::WorthServerAdmission {
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            resolved,
            worth_server::WorthServerPipelineIntent::query_read("users.profile"),
        )) {
        worth_proof::TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted read, got {other:?}"),
    }
}

pub fn admit_mutation(
    server: &WorthServer,
    resolved: WorthServerResolvedRequestContext,
) -> worth_server::WorthServerAdmission {
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            resolved,
            worth_server::WorthServerPipelineIntent::query_mutation("users.rename"),
        )) {
        worth_proof::TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted mutation, got {other:?}"),
    }
}

pub fn admit_read_posture(
    server: &WorthServer,
    resolved: WorthServerResolvedRequestContext,
) -> WorthServerOperationAdmissionPosture {
    let admission = admit_read(server, resolved);
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
        .expect("read operation admission should admit")
}

pub fn admit_mutation_posture(
    server: &WorthServer,
    resolved: WorthServerResolvedRequestContext,
) -> WorthServerOperationAdmissionPosture {
    let admission = admit_mutation(server, resolved);
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            worth_server::WorthServerOperationRequestInput::builder()
                .with_operation_family(worth_server::WorthServerOperationFamily::QueryDirectSubmission)
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
            worth_server::WorthServerOperationAuthorityMetadata::deterministic_submission(
                "query-write",
                "query-write-review",
                "caller-basis-unbound",
                "idempotent",
            ),
        )
        .expect("mutation operation admission should admit")
}

pub fn middleware_mutation_denial(server: &WorthServer) -> worth_server::WorthServerDenial {
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            resolve_ready(
                server,
                request_input(
                    worth_server::WorthServerSurfaceFamily::CompatHttp,
                    worth_server::WorthServerTransportClass::CompatHttp,
                ),
            ),
            worth_server::WorthServerPipelineIntent::query_mutation("users.rename"),
        )) {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected middleware denial, got {other:?}"),
    }
}

pub fn query_handoff_success(
    server: &WorthServer,
    surface_family: WorthServerSurfaceFamily,
    transport_class: WorthServerTransportClass,
    operation: WorthServerQueryHandoffOperation,
) -> worth_server::WorthServerQueryHandoff {
    let resolved = resolve_ready(server, request_input(surface_family, transport_class));
    let admission = match &operation {
        worth_server::WorthServerQueryHandoffOperation::QueryRead { .. }
        | worth_server::WorthServerQueryHandoffOperation::DownstreamDelivery { .. } => {
            admit_read_posture(server, resolved)
        }
        worth_server::WorthServerQueryHandoffOperation::QueryMutation { .. } => {
            admit_mutation_posture(server, resolved)
        }
        worth_server::WorthServerQueryHandoffOperation::DirectRead { .. }
        | worth_server::WorthServerQueryHandoffOperation::DirectState { .. }
        | worth_server::WorthServerQueryHandoffOperation::DirectInspection { .. }
        | worth_server::WorthServerQueryHandoffOperation::DirectProjection { .. }
        | worth_server::WorthServerQueryHandoffOperation::DirectMutation { .. } => {
            panic!("response fixture does not construct direct Worth-native query handoffs")
        }
    };
    match server
        .query_handoff()
        .prepare(worth_server::WorthServerQueryHandoffInput::new(admission, operation))
    {
        worth_proof::TransitionOutcome::Success(handoff) => handoff,
        other => panic!("expected query handoff success, got {other:?}"),
    }
}

pub fn query_handoff_durable_denial(server: &WorthServer) -> WorthServerQueryHandoffDenial {
    let outcome: WorthServerQueryHandoffOutcome =
        server
            .query_handoff()
            .prepare(worth_server::WorthServerQueryHandoffInput::new(
                admit_read_posture(
                    server,
                    resolve_ready(
                        server,
                        request_input(
                            worth_server::WorthServerSurfaceFamily::CompatHttp,
                            worth_server::WorthServerTransportClass::CompatHttp,
                        ),
                    ),
                ),
                worth_server::WorthServerQueryHandoffOperation::downstream_delivery(
                    "users.profile",
                    worth_server::WorthServerDirectFreshnessMode::LiveStrict,
                    worth_server::WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                    worth_server::WorthServerQueryRequestedResume::durable(),
                ),
            ));
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected durable resume denial, got {other:?}"),
    }
}

pub fn shape_success(
    server: &WorthServer,
    handoff: worth_server::WorthServerQueryHandoff,
    transform: WorthServerResponseTransform,
) -> worth_server::WorthServerResponseEnvelope {
    server.responses().shape(
        worth_server::WorthServerResponseInput::query_handoff_success(handoff),
        transform,
    )
}

pub fn operator_evidence_record(
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