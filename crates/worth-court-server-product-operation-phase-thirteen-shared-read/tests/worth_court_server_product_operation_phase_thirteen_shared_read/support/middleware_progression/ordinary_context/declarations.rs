pub(crate) use {worth_proof::TransitionOutcome, worth_proof::TransitionReadiness};
pub(crate) use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerAdmission, worth_server::WorthServerConfig, worth_server::WorthServerDenial, worth_server::WorthServerDenialCode, worth_server::WorthServerDenialPriority, worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerPipelineInput, worth_server::WorthServerPipelineIntent, worth_server::WorthServerPipelineStep, worth_server::WorthServerPreparedQueryHandoffKind, worth_server::WorthServerRequestContextConfig, worth_server::WorthServerRequestContextInput, worth_server::WorthServerResolvedRequestContext, worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass};

pub(crate) fn test_server(
    request_context: WorthServerRequestContextConfig,
    middleware: WorthServerMiddlewareConfig,
) -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(request_context)
                .with_middleware_config(middleware)
                .build()
                .expect("server config should validate"),
        )
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

pub(crate) fn request_input_builder() -> worth_server::WorthServerRequestContextInputBuilder {
    worth_server::WorthServerRequestContextInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
}

pub(crate) fn resolve_request_context(
    server: &WorthServer,
    input: WorthServerRequestContextInput,
) -> WorthServerResolvedRequestContext {
    match server.request_contexts().resolve(input) {
        worth_proof::TransitionReadiness::Ready(resolved) => resolved,
        other => panic!("expected resolved request context, got {other:?}"),
    }
}

pub(crate) fn denied(
    outcome: TransitionOutcome<
        WorthServerAdmission,
        WorthServerDenial,
        worth_server::WorthServerMiddlewareDeferred,
        worth_server::WorthServerMiddlewareStale,
        worth_server::WorthServerMiddlewareRebindRequired,
        worth_server::WorthServerMiddlewareFailure,
    >,
) -> WorthServerDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected middleware denial, got {other:?}"),
    }
}