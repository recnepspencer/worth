use worth_proof::{TransitionOutcome, TransitionReadiness};
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerCompatibilityRequestInput, WorthServerConfig,
    WorthServerRequestContextConfig, WorthServerRequestContextDenial,
    WorthServerRequestContextInput, WorthServerResolvedRequestContext, WorthServerSurfaceFamily,
    WorthServerTransportClass,
};

pub(crate) fn compat_http_entry_test_server() -> WorthServer {
    WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("request context config should validate"),
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

pub(crate) fn compat_http_prepared_request(
    server: &WorthServer,
    input: WorthServerCompatibilityRequestInput,
) -> worth_server::WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(input) {
        TransitionOutcome::Success(request) => request,
        other => panic!("expected admitted compat request, got {other:?}"),
    }
}

pub(crate) fn ready_context(
    resolution: TransitionReadiness<
        WorthServerResolvedRequestContext,
        WorthServerRequestContextDenial,
        worth_server::WorthServerRequestContextDeferred,
        worth_server::WorthServerRequestContextStale,
        worth_server::WorthServerRequestContextRebindRequired,
        worth_server::WorthServerRequestContextFailure,
    >,
) -> WorthServerResolvedRequestContext {
    match resolution {
        TransitionReadiness::Ready(context) => context,
        other => panic!("expected ready request context, got {other:?}"),
    }
}

pub(crate) fn worth_native_request_context_input() -> WorthServerRequestContextInput {
    WorthServerRequestContextInput::builder()
        .with_surface_family(WorthServerSurfaceFamily::WorthNative)
        .with_transport_class(WorthServerTransportClass::WorthNativeInProcess)
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .build()
        .expect("Worth-native input should validate")
}
