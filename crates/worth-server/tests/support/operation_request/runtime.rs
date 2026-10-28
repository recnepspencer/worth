use worth_proof::{TransitionOutcome, TransitionReadiness};
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerCompatHttpRouteFamily, WorthServerCompatibilityPreparedRequest,
    WorthServerCompatibilityRequestInput, WorthServerConfig, WorthServerMiddlewareConfig,
    WorthServerOperationRegistration, WorthServerPipelineInput, WorthServerPipelineIntent,
    WorthServerRequestContextConfig, WorthServerRequestContextInput,
    WorthServerResolvedRequestContext, WorthServerSurfaceFamily, WorthServerTransportClass,
};

pub(crate) fn operation_request_test_server() -> WorthServer {
    operation_request_test_server_with_operations(
        WorthServerOperationRegistration::phase_two_defaults(),
    )
}

pub(crate) fn operation_request_test_server_with_operations(
    registrations: impl IntoIterator<Item = WorthServerOperationRegistration>,
) -> WorthServer {
    WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    WorthServerRequestContextConfig::builder()
                        .with_default_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                        .with_preview_targeting_enabled(true)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(registrations)
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("server should build")
}

pub(crate) fn compat_prepared_request(
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
        TransitionOutcome::Success(prepared_request) => prepared_request,
        other => panic!("expected prepared compatibility request, got {other:?}"),
    }
}

pub(crate) fn compat_request_input(
    route_family: WorthServerCompatHttpRouteFamily,
    method: &str,
    path: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(route_family)
        .with_method(method)
        .with_path(path)
}

pub(crate) fn worth_native_resolved_context(
    server: &WorthServer,
    diagnostics_profile: Option<DiagnosticRichnessProfile>,
) -> WorthServerResolvedRequestContext {
    worth_native_resolved_context_for_principal(server, diagnostics_profile, "principal-7")
}

pub(crate) fn worth_native_resolved_context_for_principal(
    server: &WorthServer,
    diagnostics_profile: Option<DiagnosticRichnessProfile>,
    principal_id: &str,
) -> WorthServerResolvedRequestContext {
    let mut builder = WorthServerRequestContextInput::builder()
        .with_surface_family(WorthServerSurfaceFamily::WorthNative)
        .with_transport_class(WorthServerTransportClass::WorthNativeInProcess)
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
        TransitionReadiness::Ready(resolved) => resolved,
        other => panic!("expected resolved request context, got {other:?}"),
    }
}

pub(crate) fn worth_native_admission(
    server: &WorthServer,
    diagnostics_profile: Option<DiagnosticRichnessProfile>,
    operation_name: &str,
) -> worth_server::WorthServerAdmission {
    match server.middleware().admit(WorthServerPipelineInput::new(
        worth_native_resolved_context(server, diagnostics_profile),
        WorthServerPipelineIntent::query_mutation(operation_name),
    )) {
        TransitionOutcome::Success(admission) => admission,
        other => panic!("expected admitted Worth-native operation, got {other:?}"),
    }
}
