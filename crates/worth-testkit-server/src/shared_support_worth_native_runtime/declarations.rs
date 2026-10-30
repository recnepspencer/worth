pub use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
pub use {std::sync::atomic::AtomicUsize, std::sync::Arc, std::sync::Mutex};
pub use {worth_proof::TransitionOutcome, worth_proof::TransitionReadiness};
pub use {
    worth_server::request_context::DiagnosticRichnessProfile,
    worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface,
    worth_server::WorthServer, worth_server::WorthServerConfig,
    worth_server::WorthServerDenialCode, worth_server::WorthServerMiddlewareConfig,
    worth_server::WorthServerQueryHandoffConfig,
    worth_server::WorthServerQueryWorkspaceBindingError,
    worth_server::WorthServerQueryWorkspaceBindingRequest,
    worth_server::WorthServerQueryWorkspaceBindingTarget,
    worth_server::WorthServerQueryWorkspaceProvider, worth_server::WorthServerRequestContextConfig,
    worth_server::WorthServerRequestContextInput, worth_server::WorthServerResolvedRequestContext,
    worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass,
    worth_server::WorthServerWorthNativeSessionDenial,
    worth_server::WorthServerWorthNativeSessionInput,
};

pub use {
    crate::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider,
    crate::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider,
    crate::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider,
};

pub fn build_server(register_worth_native: bool) -> WorthServer {
    let _ = crate::setup_observation::record("root-default-query-provider-server", &|inputs| {
        inputs.unsigned("recipe_revision", 1);
        inputs.boolean("register_worth_native", register_worth_native);
    });
    build_server_with_workspace_provider(TestWorkspaceProvider, register_worth_native)
}

pub fn build_server_with_disabled_worth_native() -> WorthServer {
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
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(TestWorkspaceProvider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

pub fn build_server_with_preview_denial() -> WorthServer {
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
                        .with_preview_branch_authorization_enabled(false)
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(TestWorkspaceProvider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

pub fn build_server_with_profiled_workspace(
    support_profile: WorthQueryRuntimeSupportProfile,
) -> WorthServer {
    build_server_with_workspace_provider(crate::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(support_profile), true)
}

pub fn build_server_with_profiled_counting_workspace(
    support_profile: WorthQueryRuntimeSupportProfile,
) -> (WorthServer, Arc<AtomicUsize>) {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_server_with_workspace_provider(
        crate::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider::new(support_profile, attempted_writes.clone()),
        true,
    );
    (server, attempted_writes)
}

pub fn build_server_with_capturing_workspace_provider() -> (WorthServer, CapturingWorkspaceProvider)
{
    let workspace_provider =
        crate::shared_support_worth_native_runtime::CapturingWorkspaceProvider::default();
    let server = build_server_with_workspace_provider(workspace_provider.clone(), true);
    (server, workspace_provider)
}

pub fn build_server_with_failing_workspace_provider(
    stage: &'static str,
    message: &'static str,
) -> WorthServer {
    build_server_with_workspace_provider(FailingWorkspaceProvider { stage, message }, true)
}

pub fn server_with_request_context_default(
    default_diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(default_diagnostics_profile)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    worth_server::WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(TestWorkspaceProvider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

pub fn worth_native_session_input_builder(
) -> worth_server::WorthServerWorthNativeSessionInputBuilder {
    worth_server::WorthServerWorthNativeSessionInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
}

pub fn resolve_worth_native_request_context(
    server: &WorthServer,
) -> WorthServerResolvedRequestContext {
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

pub fn denied_prepared_session(
    outcome: worth_proof::TransitionOutcome<
        worth_server::WorthServerWorthNativePreparedSession,
        WorthServerWorthNativeSessionDenial,
        worth_server::WorthServerWorthNativeDeferred,
        worth_server::WorthServerWorthNativeStale,
        worth_server::WorthServerWorthNativeRebindRequired,
        worth_server::WorthServerWorthNativeFailure,
    >,
) -> WorthServerWorthNativeSessionDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected Worth-native denial, got {other:?}"),
    }
}

pub fn denied_session(
    outcome: worth_proof::TransitionOutcome<
        worth_server::WorthServerWorthNativeSession,
        WorthServerWorthNativeSessionDenial,
        worth_server::WorthServerWorthNativeDeferred,
        worth_server::WorthServerWorthNativeStale,
        worth_server::WorthServerWorthNativeRebindRequired,
        worth_server::WorthServerWorthNativeFailure,
    >,
) -> WorthServerWorthNativeSessionDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected Worth-native denial, got {other:?}"),
    }
}

pub fn expect_preview_access_denial(denial: &WorthServerWorthNativeSessionDenial) {
    assert_eq!(
        denial
            .middleware_denial()
            .expect("middleware denial should be preserved")
            .code(),
        worth_server::WorthServerDenialCode::PreviewBranchAccessDenied
    );
}

pub fn build_server_with_workspace_provider(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    register_worth_native: bool,
) -> WorthServer {
    let mut builder = worth_server::WorthServer::builder().with_config(
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
    );
    builder = builder
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults());

    if register_worth_native {
        builder = builder.register_surface(worth_server::surfaces::WorthNativeSurface::enabled());
    }

    builder
        .register_surface(worth_server::surfaces::CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

#[derive(Clone, Debug, Default)]
pub struct CapturingWorkspaceProvider {
    pub captured_targets: Arc<Mutex<Vec<WorthServerQueryWorkspaceBindingTarget>>>,
}

impl CapturingWorkspaceProvider {
    pub fn take_captured_targets(&self) -> Vec<WorthServerQueryWorkspaceBindingTarget> {
        let mut captured_targets = self
            .captured_targets
            .lock()
            .expect("capturing workspace provider mutex should not be poisoned");
        std::mem::take(&mut *captured_targets)
    }
}

impl WorthServerQueryWorkspaceProvider for CapturingWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "capturing-test-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<
        worth_query::facade::runtime::WorthQueryWorkspace,
        WorthServerQueryWorkspaceBindingError,
    > {
        self.captured_targets
            .lock()
            .expect("capturing workspace provider mutex should not be poisoned")
            .push(request.target().clone());

        TestWorkspaceProvider.bind_workspace(request)
    }
}

#[derive(Clone, Debug)]
pub struct FailingWorkspaceProvider {
    pub stage: &'static str,
    pub message: &'static str,
}

impl WorthServerQueryWorkspaceProvider for FailingWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "failing-test-workspace-provider"
    }

    fn bind_workspace(
        &self,
        _request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<
        worth_query::facade::runtime::WorthQueryWorkspace,
        WorthServerQueryWorkspaceBindingError,
    > {
        Err(worth_server::WorthServerQueryWorkspaceBindingError::new(
            self.stage,
            self.message,
        ))
    }
}
