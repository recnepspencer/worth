use worth_proof::TransitionOutcome;
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerBinaryDownload, WorthServerBinaryDownloadExecutionInput,
    WorthServerBinaryDownloadOutcome, WorthServerBinaryDownloadRequest,
    WorthServerBinaryEgressSession, WorthServerCompatHttpRouteFamily,
    WorthServerCompatibilityPreparedRequest, WorthServerCompatibilityRequestInput,
    WorthServerConfig, WorthServerMiddlewareConfig, WorthServerQueryHandoffConfig,
    WorthServerQueryHandoffDenial, WorthServerQueryWorkspaceProvider,
    WorthServerRequestContextConfig,
};

pub(crate) fn build_phase_eight_server_with_workspace_provider(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
) -> WorthServer {
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
                        .with_workspace_provider(workspace_provider)
                        .build()
                        .expect("query handoff config should validate"),
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

pub(crate) fn download_input(
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-8")
        .with_tenant_id(tenant_id)
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(WorthServerCompatHttpRouteFamily::Download)
        .with_method("GET")
        .with_path(format!("/compat/downloads/{operation_name}"))
        .with_header("accept", "application/octet-stream")
}

pub(crate) fn prepared_request(
    server: &WorthServer,
    request: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(request) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compatibility request, got {other:?}"),
    }
}

pub(crate) fn compat_download_execution_input(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    request: WorthServerBinaryDownloadRequest,
) -> WorthServerBinaryDownloadExecutionInput {
    WorthServerBinaryDownloadExecutionInput::new(
        prepared_request(
            server,
            download_input(tenant_id, workspace_id, branch_id, operation_name)
                .build()
                .expect("download input should validate structurally"),
        ),
        operation_name,
        request,
    )
}

pub(crate) fn compat_download_success(
    outcome: WorthServerBinaryDownloadOutcome<WorthServerBinaryDownload>,
) -> WorthServerBinaryDownload {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected binary download success, got {other:?}"),
    }
}

pub(crate) fn compat_download_denied(
    outcome: WorthServerBinaryDownloadOutcome<WorthServerBinaryDownload>,
) -> WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(value) => value,
        other => panic!("expected binary download denial, got {other:?}"),
    }
}

pub(crate) fn compat_resume_success(
    outcome: WorthServerBinaryDownloadOutcome<worth_server::WorthServerBinarySessionResume>,
) -> worth_server::WorthServerBinarySessionResume {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected binary session resume, got {other:?}"),
    }
}

pub(crate) fn plan_integrity(
    server: &WorthServer,
    session: &WorthServerBinaryEgressSession,
) -> worth_server::WorthServerBinaryIntegrityDigest {
    server.compat_http().plan_binary_integrity(session)
}
