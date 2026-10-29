pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerCompatibilityDenial, worth_server::WorthServerCompatibilityRequest, worth_server::WorthServerCompatibilityRequestInput, worth_server::WorthServerConfig, worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerRequestContextConfig};

pub(crate) fn compat_http_admission_test_server() -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_default_diagnostics_profile(worth_server::request_context::DiagnosticRichnessProfile::Standard)
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
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("server should build")
}

pub(crate) fn compat_http_denial(
    server: &WorthServer,
    input: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityDenial {
    match server.compat_http().request(input) {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied compat request, got {other:?}"),
    }
}

pub(crate) fn compat_http_request(
    server: &WorthServer,
    input: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityRequest {
    match server.compat_http().request(input) {
        worth_proof::TransitionOutcome::Success(request) => request,
        other => panic!("expected successful compat request, got {other:?}"),
    }
}