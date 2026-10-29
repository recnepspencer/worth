pub(crate) use {worth_proof::TransitionOutcome, worth_proof::TransitionReadiness};
pub(crate) use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::SyncSurface, worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerConfig, worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerPipelineInput, worth_server::WorthServerPipelineIntent, worth_server::WorthServerRequestContextConfig, worth_server::WorthServerRequestContextInput, worth_server::WorthServerResolvedRequestContext, worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass};

pub(crate) fn test_server_with_surfaces(registered_families: &[WorthServerSurfaceFamily]) -> WorthServer {
    let mut builder = worth_server::WorthServer::builder().with_config(
        worth_server::WorthServerConfig::builder()
            .with_bind_address(([127, 0, 0, 1], 8080).into())
            .with_request_context_config(
                worth_server::WorthServerRequestContextConfig::builder()
                    .with_preview_targeting_enabled(true)
                    .with_default_diagnostics_profile(worth_server::request_context::DiagnosticRichnessProfile::Standard)
                    .build()
                    .expect("request context config should validate"),
            )
            .with_middleware_config(
                worth_server::WorthServerMiddlewareConfig::builder()
                    .with_query_mutation_enabled(false)
                    .build()
                    .expect("middleware config should validate"),
            )
            .build()
            .expect("server config should validate"),
    );

    for family in registered_families {
        builder = match family {
            worth_server::WorthServerSurfaceFamily::WorthNative => {
                builder.register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
            }
            worth_server::WorthServerSurfaceFamily::CompatHttp => {
                builder.register_surface(worth_server::surfaces::CompatHttpSurface::disabled())
            }
            worth_server::WorthServerSurfaceFamily::Sync => builder.register_surface(worth_server::surfaces::SyncSurface::disabled()),
            unsupported => panic!("unexpected unsupported test family {unsupported:?}"),
        };
    }

    builder.build().expect("server should build")
}

pub(crate) fn resolve_worth_native_request_context(server: &WorthServer) -> WorthServerResolvedRequestContext {
    match server.request_contexts().resolve(
        worth_server::WorthServerRequestContextInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_surface_family(worth_server::WorthServerSurfaceFamily::WorthNative)
            .with_transport_class(worth_server::WorthServerTransportClass::WorthNativeInProcess)
            .build()
            .expect("request context input should validate"),
    ) {
        worth_proof::TransitionReadiness::Ready(resolved) => resolved,
        other => panic!("expected resolved request context, got {other:?}"),
    }
}