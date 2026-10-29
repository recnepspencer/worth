pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerBuildError, worth_server::WorthServerCompatHttpRouteFamily, worth_server::WorthServerCompatibilityExecutionInput, worth_server::WorthServerCompatibilityRequestInput, worth_server::WorthServerConfig, worth_server::WorthServerOperationDenial, worth_server::WorthServerOperationFamily, worth_server::WorthServerOperationRegistration, worth_server::WorthServerQueryHandoffConfig, worth_server::WorthServerQueryWorkspaceBindingError, worth_server::WorthServerQueryWorkspaceBindingRequest, worth_server::WorthServerQueryWorkspaceProvider, worth_server::WorthServerRequestContextConfig};

pub(crate) fn build_server(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    operations: Vec<WorthServerOperationRegistration>,
) -> Result<WorthServer, WorthServerBuildError> {
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
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(workspace_provider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(operations)
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .build()
}

pub(crate) fn prepared_read_request(
    server: &WorthServer,
) -> worth_server::WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/compat/reads/users.profile")
            .with_header("accept", "application/json")
            .build()
            .expect("compat request should validate"),
    ) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compatibility request, got {other:?}"),
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CountingWorkspaceProvider {
    pub(crate) bind_count: Arc<AtomicUsize>,
}

impl CountingWorkspaceProvider {
    pub(crate) fn bind_count(&self) -> Arc<AtomicUsize> {
        self.bind_count.clone()
    }
}

impl WorthServerQueryWorkspaceProvider for CountingWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "counting-workspace-provider"
    }

    fn bind_workspace(
        &self,
        _request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<
        worth_query::facade::runtime::WorthQueryWorkspace,
        WorthServerQueryWorkspaceBindingError,
    > {
        self.bind_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Err(worth_server::WorthServerQueryWorkspaceBindingError::new(
            "bind_workspace",
            "operation-family guard should deny before workspace binding",
        ))
    }
}