pub(crate) use {worth_proof::TransitionReadiness};
pub(crate) use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerBranchTarget, worth_server::WorthServerConfig, worth_server::WorthServerRequestContext, worth_server::WorthServerRequestContextConfig, worth_server::WorthServerRequestContextDenial, worth_server::WorthServerRequestContextDenialCode, worth_server::WorthServerRequestContextInput, worth_server::WorthServerResolvedRequestContext, worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass};

pub(crate) fn test_server(request_context: WorthServerRequestContextConfig) -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(request_context)
                .build()
                .expect("server config should validate"),
        )
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

pub(crate) fn input_builder() -> worth_server::WorthServerRequestContextInputBuilder {
    worth_server::WorthServerRequestContextInput::builder()
        .with_surface_family(worth_server::WorthServerSurfaceFamily::WorthNative)
        .with_transport_class(worth_server::WorthServerTransportClass::WorthNativeInProcess)
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
}

pub(crate) fn denied(
    resolution: TransitionReadiness<
        WorthServerResolvedRequestContext,
        WorthServerRequestContextDenial,
        worth_server::WorthServerRequestContextDeferred,
        worth_server::WorthServerRequestContextStale,
        worth_server::WorthServerRequestContextRebindRequired,
        worth_server::WorthServerRequestContextFailure,
    >,
) -> WorthServerRequestContextDenial {
    match resolution {
        worth_proof::TransitionReadiness::Denied(denial) => denial,
        other => panic!("expected denial, got {other:?}"),
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
        worth_proof::TransitionReadiness::Ready(context) => context,
        other => panic!("expected ready context, got {other:?}"),
    }
}