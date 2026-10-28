use worth_proof::TransitionOutcome;
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerCompatibilityDenial, WorthServerCompatibilityRequest,
    WorthServerCompatibilityRequestInput, WorthServerConfig, WorthServerMiddlewareConfig,
    WorthServerRequestContextConfig,
};

pub(crate) fn compat_http_admission_test_server() -> WorthServer {
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
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("server should build")
}

pub(crate) fn compat_http_denial(
    server: &WorthServer,
    input: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityDenial {
    match server.compat_http().request(input) {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied compat request, got {other:?}"),
    }
}

pub(crate) fn compat_http_request(
    server: &WorthServer,
    input: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityRequest {
    match server.compat_http().request(input) {
        TransitionOutcome::Success(request) => request,
        other => panic!("expected successful compat request, got {other:?}"),
    }
}
