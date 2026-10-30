pub use {worth_proof::TransitionOutcome, worth_proof::TransitionReadiness};
pub use {
    worth_server::request_context::DiagnosticRichnessProfile,
    worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::SyncSurface,
    worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer,
    worth_server::WorthServerAdmission, worth_server::WorthServerConfig,
    worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerOperationAdmissionPosture,
    worth_server::WorthServerOperationFamily, worth_server::WorthServerOperationRequestInput,
    worth_server::WorthServerQueryHandoff, worth_server::WorthServerQueryHandoffConfig,
    worth_server::WorthServerQueryHandoffDenial, worth_server::WorthServerQueryHandoffOutcome,
    worth_server::WorthServerQueryWorkspaceProvider, worth_server::WorthServerRequestContextConfig,
    worth_server::WorthServerRequestContextInput, worth_server::WorthServerResolvedRequestContext,
    worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass,
};

pub fn test_server(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    include_sync_surface: bool,
) -> WorthServer {
    test_server_with_middleware(
        workspace_provider,
        include_sync_surface,
        worth_server::WorthServerMiddlewareConfig::builder()
            .build()
            .expect("middleware config should validate"),
    )
}

pub fn test_server_with_middleware(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    include_sync_surface: bool,
    middleware_config: WorthServerMiddlewareConfig,
) -> WorthServer {
    let mut builder = worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8090).into())
                .with_middleware_config(middleware_config)
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_default_diagnostics_profile(
                            worth_server::request_context::DiagnosticRichnessProfile::Standard,
                        )
                        .build()
                        .expect("request context config should validate"),
                )
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(workspace_provider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::disabled());
    if include_sync_surface {
        builder = builder.register_surface(worth_server::surfaces::SyncSurface::disabled());
    }
    builder.build().expect("server should build")
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
        .expect("request context input should validate")
}

pub fn resolve_request_context(
    server: &WorthServer,
    input: WorthServerRequestContextInput,
) -> WorthServerResolvedRequestContext {
    match server.request_contexts().resolve(input) {
        worth_proof::TransitionReadiness::Ready(resolved) => resolved,
        other => panic!("expected resolved request context, got {other:?}"),
    }
}

pub fn admit_read(
    server: &WorthServer,
    resolved: WorthServerResolvedRequestContext,
) -> WorthServerAdmission {
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            resolved,
            worth_server::WorthServerPipelineIntent::query_read("users.profile"),
        )) {
        worth_proof::TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted pipeline result, got {other:?}"),
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
        .admit_declared(&admission, &operation_request)
        .expect("read operation admission should admit")
}

pub fn admit_mutation(
    server: &WorthServer,
    resolved: WorthServerResolvedRequestContext,
) -> WorthServerAdmission {
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            resolved,
            worth_server::WorthServerPipelineIntent::query_mutation("users.rename"),
        )) {
        worth_proof::TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted mutation pipeline result, got {other:?}"),
    }
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
                .with_operation_family(
                    worth_server::WorthServerOperationFamily::QueryDirectSubmission,
                )
                .with_operation_name("users.rename")
                .with_idempotency_key("idem-7")
                .build(),
        )
        .expect("mutation operation request should admit");
    server
        .operation_admissions()
        .admit_declared(&admission, &operation_request)
        .expect("mutation operation admission should admit")
}

pub fn admit_delivery_posture(
    server: &WorthServer,
    resolved: WorthServerResolvedRequestContext,
    basis_digest: &str,
) -> WorthServerOperationAdmissionPosture {
    let admission = admit_read(server, resolved);
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            worth_server::WorthServerOperationRequestInput::builder()
                .with_operation_family(worth_server::WorthServerOperationFamily::SyncLease)
                .with_operation_name("users.profile")
                .with_basis_digest(basis_digest)
                .build(),
        )
        .expect("delivery operation request should admit");
    server
        .operation_admissions()
        .admit_declared(&admission, &operation_request)
        .expect("delivery operation admission should admit")
}

pub fn success(outcome: WorthServerQueryHandoffOutcome) -> WorthServerQueryHandoff {
    match outcome {
        worth_proof::TransitionOutcome::Success(handoff) => handoff,
        other => panic!("expected successful query handoff, got {other:?}"),
    }
}

#[allow(dead_code)]
pub fn denied(outcome: WorthServerQueryHandoffOutcome) -> WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied query handoff, got {other:?}"),
    }
}
