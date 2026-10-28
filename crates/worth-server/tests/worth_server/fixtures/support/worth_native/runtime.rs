use std::sync::{atomic::AtomicUsize, Arc, Mutex};
use worth_proof::{TransitionOutcome, TransitionReadiness};
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerConfig, WorthServerDenialCode, WorthServerMiddlewareConfig,
    WorthServerQueryHandoffConfig, WorthServerQueryWorkspaceBindingError,
    WorthServerQueryWorkspaceBindingRequest, WorthServerQueryWorkspaceBindingTarget,
    WorthServerQueryWorkspaceProvider, WorthServerRequestContextConfig,
    WorthServerRequestContextInput, WorthServerResolvedRequestContext, WorthServerSurfaceFamily,
    WorthServerTransportClass, WorthServerWorthNativeSessionDenial,
    WorthServerWorthNativeSessionInput,
};

use crate::shared_support_query_handoff_runtime::{
    ProfiledCountingTestWorkspaceProvider, ProfiledTestWorkspaceProvider, TestWorkspaceProvider,
};

pub(crate) fn build_server(register_worth_native: bool) -> WorthServer {
    build_server_with_workspace_provider(TestWorkspaceProvider, register_worth_native)
}

pub(crate) fn build_server_with_disabled_worth_native() -> WorthServer {
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
                .with_middleware_config(
                    WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(TestWorkspaceProvider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(WorthNativeSurface::disabled())
        .register_surface(CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

pub(crate) fn build_server_with_preview_denial() -> WorthServer {
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
                .with_middleware_config(
                    WorthServerMiddlewareConfig::builder()
                        .with_preview_branch_authorization_enabled(false)
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(TestWorkspaceProvider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

pub(crate) fn build_server_with_profiled_workspace(
    support_profile: WorthQueryRuntimeSupportProfile,
) -> WorthServer {
    build_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(support_profile), true)
}

pub(crate) fn build_server_with_profiled_counting_workspace(
    support_profile: WorthQueryRuntimeSupportProfile,
) -> (WorthServer, Arc<AtomicUsize>) {
    let attempted_writes = Arc::new(AtomicUsize::new(0));
    let server = build_server_with_workspace_provider(
        ProfiledCountingTestWorkspaceProvider::new(support_profile, attempted_writes.clone()),
        true,
    );
    (server, attempted_writes)
}

pub(crate) fn build_server_with_capturing_workspace_provider(
) -> (WorthServer, CapturingWorkspaceProvider) {
    let workspace_provider = CapturingWorkspaceProvider::default();
    let server = build_server_with_workspace_provider(workspace_provider.clone(), true);
    (server, workspace_provider)
}

pub(crate) fn build_server_with_failing_workspace_provider(
    stage: &'static str,
    message: &'static str,
) -> WorthServer {
    build_server_with_workspace_provider(FailingWorkspaceProvider { stage, message }, true)
}

pub(crate) fn server_with_request_context_default(
    default_diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServer {
    WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(default_diagnostics_profile)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(TestWorkspaceProvider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

pub(crate) fn worth_native_session_input_builder(
) -> worth_server::WorthServerWorthNativeSessionInputBuilder {
    WorthServerWorthNativeSessionInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
}

pub(crate) fn resolve_worth_native_request_context(
    server: &WorthServer,
) -> WorthServerResolvedRequestContext {
    match server.request_contexts().resolve(
        WorthServerRequestContextInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_surface_family(WorthServerSurfaceFamily::WorthNative)
            .with_transport_class(WorthServerTransportClass::WorthNativeInProcess)
            .build()
            .expect("request context input should validate"),
    ) {
        TransitionReadiness::Ready(resolved) => resolved,
        other => panic!("expected resolved request context, got {other:?}"),
    }
}

pub(crate) fn denied_prepared_session(
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
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected Worth-native denial, got {other:?}"),
    }
}

pub(crate) fn denied_session(
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
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected Worth-native denial, got {other:?}"),
    }
}

pub(crate) fn expect_preview_access_denial(denial: &WorthServerWorthNativeSessionDenial) {
    assert_eq!(
        denial
            .middleware_denial()
            .expect("middleware denial should be preserved")
            .code(),
        WorthServerDenialCode::PreviewBranchAccessDenied
    );
}

pub(crate) fn build_server_with_workspace_provider(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    register_worth_native: bool,
) -> WorthServer {
    let mut builder = WorthServer::builder().with_config(
        WorthServerConfig::builder()
            .with_bind_address(([127, 0, 0, 1], 8080).into())
            .with_request_context_config(
                WorthServerRequestContextConfig::builder()
                    .with_preview_targeting_enabled(true)
                    .with_default_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                    .build()
                    .expect("request context config should validate"),
            )
            .with_middleware_config(
                WorthServerMiddlewareConfig::builder()
                    .with_query_mutation_enabled(true)
                    .build()
                    .expect("middleware config should validate"),
            )
            .with_query_handoff_config(
                WorthServerQueryHandoffConfig::builder()
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
        builder = builder.register_surface(WorthNativeSurface::enabled());
    }

    builder
        .register_surface(CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CapturingWorkspaceProvider {
    captured_targets: Arc<Mutex<Vec<WorthServerQueryWorkspaceBindingTarget>>>,
}

impl CapturingWorkspaceProvider {
    pub(crate) fn take_captured_targets(&self) -> Vec<WorthServerQueryWorkspaceBindingTarget> {
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
struct FailingWorkspaceProvider {
    stage: &'static str,
    message: &'static str,
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
        Err(WorthServerQueryWorkspaceBindingError::new(
            self.stage,
            self.message,
        ))
    }
}
