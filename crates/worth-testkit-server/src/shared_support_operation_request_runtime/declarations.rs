pub use {worth_proof::TransitionOutcome, worth_proof::TransitionReadiness};
pub use {
    worth_server::request_context::DiagnosticRichnessProfile,
    worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface,
    worth_server::WorthServer, worth_server::WorthServerCompatHttpRouteFamily,
    worth_server::WorthServerCompatibilityPreparedRequest,
    worth_server::WorthServerCompatibilityRequestInput, worth_server::WorthServerConfig,
    worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerOperationRegistration,
    worth_server::WorthServerPipelineInput, worth_server::WorthServerPipelineIntent,
    worth_server::WorthServerRequestContextConfig, worth_server::WorthServerRequestContextInput,
    worth_server::WorthServerResolvedRequestContext, worth_server::WorthServerSurfaceFamily,
    worth_server::WorthServerTransportClass,
};

pub fn operation_request_test_server() -> WorthServer {
    operation_request_test_server_with_operations(
        worth_server::WorthServerOperationRegistration::phase_two_defaults(),
    )
}

pub fn operation_request_test_server_with_operations(
    registrations: impl IntoIterator<Item = WorthServerOperationRegistration>,
) -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_default_diagnostics_profile(
                            worth_server::request_context::DiagnosticRichnessProfile::Standard,
                        )
                        .with_preview_targeting_enabled(true)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    worth_server::WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(registrations)
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("server should build")
}

pub fn compat_prepared_request(
    server: &WorthServer,
    diagnostics_profile: Option<DiagnosticRichnessProfile>,
    route_family: WorthServerCompatHttpRouteFamily,
    method: &str,
    path: &str,
    basis_digest: Option<&str>,
    idempotency_key: Option<&str>,
) -> WorthServerCompatibilityPreparedRequest {
    let mut builder = compat_request_input(route_family, method, path);
    if let Some(diagnostics_profile) = diagnostics_profile {
        builder = builder.with_diagnostics_profile(diagnostics_profile);
    }
    if let Some(basis_digest) = basis_digest {
        builder = builder.with_query_pair("basis", basis_digest);
    }
    if let Some(idempotency_key) = idempotency_key {
        builder = builder.with_header("idempotency-key", idempotency_key);
    }
    match server.compat_http().prepare_request(
        builder
            .build()
            .expect("compat request should validate structurally"),
    ) {
        worth_proof::TransitionOutcome::Success(prepared_request) => prepared_request,
        other => panic!("expected prepared compatibility request, got {other:?}"),
    }
}

pub fn compat_request_input(
    route_family: WorthServerCompatHttpRouteFamily,
    method: &str,
    path: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(route_family)
        .with_method(method)
        .with_path(path)
}

pub fn worth_native_resolved_context(
    server: &WorthServer,
    diagnostics_profile: Option<DiagnosticRichnessProfile>,
) -> WorthServerResolvedRequestContext {
    worth_native_resolved_context_for_principal(server, diagnostics_profile, "principal-7")
}

pub fn worth_native_resolved_context_for_principal(
    server: &WorthServer,
    diagnostics_profile: Option<DiagnosticRichnessProfile>,
    principal_id: &str,
) -> WorthServerResolvedRequestContext {
    let mut builder = worth_server::WorthServerRequestContextInput::builder()
        .with_surface_family(worth_server::WorthServerSurfaceFamily::WorthNative)
        .with_transport_class(worth_server::WorthServerTransportClass::WorthNativeInProcess)
        .with_authenticated_principal_id(principal_id)
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9");
    if let Some(diagnostics_profile) = diagnostics_profile {
        builder = builder.with_diagnostics_profile(diagnostics_profile);
    }
    match server.request_contexts().resolve(
        builder
            .build()
            .expect("Worth-native request context should validate"),
    ) {
        worth_proof::TransitionReadiness::Ready(resolved) => resolved,
        other => panic!("expected resolved request context, got {other:?}"),
    }
}

pub fn worth_native_admission(
    server: &WorthServer,
    diagnostics_profile: Option<DiagnosticRichnessProfile>,
    operation_name: &str,
) -> worth_server::WorthServerAdmission {
    match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            worth_native_resolved_context(server, diagnostics_profile),
            worth_server::WorthServerPipelineIntent::query_mutation(operation_name),
        )) {
        worth_proof::TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted Worth-native operation, got {other:?}"),
    }
}
