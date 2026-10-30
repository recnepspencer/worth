pub use {
    worth_server::request_context::DiagnosticRichnessProfile,
    worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface,
    worth_server::WorthServer, worth_server::WorthServerConfig,
    worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerQueryHandoffConfig,
    worth_server::WorthServerQueryWorkspaceProvider, worth_server::WorthServerRequestContextConfig,
};

pub fn build_phase_ten_server_with_workspace_provider(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
) -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(
                            worth_server::request_context::DiagnosticRichnessProfile::Standard,
                        )
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    worth_server::WorthServerMiddlewareConfig::builder()
                        .with_compat_http_maximum_diagnostics_profile(
                            worth_server::request_context::DiagnosticRichnessProfile::Forensic,
                        )
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
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
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("server should build")
}
