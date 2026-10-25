use worth_proof::TransitionOutcome;
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerCompatHttpRouteFamily, WorthServerCompatibilityPreparedRequest,
    WorthServerCompatibilityRequestInput, WorthServerCompatibilityUpload,
    WorthServerCompatibilityUploadExecutionInput, WorthServerCompatibilityUploadOutcome,
    WorthServerConfig, WorthServerMiddlewareConfig, WorthServerMultipartUpload,
    WorthServerPreparedMultipartUpload, WorthServerQueryHandoffConfig,
    WorthServerQueryHandoffDenial, WorthServerQueryWorkspaceProvider,
    WorthServerRequestContextConfig,
};

pub(crate) fn build_phase_six_server_with_workspace_provider(
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

pub(crate) fn upload_input(
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    boundary: &str,
) -> WorthServerCompatibilityRequestInput {
    WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id(tenant_id)
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(WorthServerCompatHttpRouteFamily::Upload)
        .with_method("POST")
        .with_path(format!("/compat/uploads/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type(format!("multipart/form-data; boundary={boundary}"))
        .with_body_present(true)
        .build()
        .expect("phase six upload input should validate structurally")
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

pub(crate) fn prepared_upload(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    boundary: &str,
    upload: WorthServerMultipartUpload,
) -> WorthServerPreparedMultipartUpload {
    match server
        .compat_http()
        .prepare_upload(WorthServerCompatibilityUploadExecutionInput::new(
            prepared_request(
                server,
                upload_input(tenant_id, workspace_id, branch_id, operation_name, boundary),
            ),
            operation_name,
            upload,
        )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared upload, got {other:?}"),
    }
}

pub(crate) fn compat_upload_execution_input(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    boundary: &str,
    upload: WorthServerMultipartUpload,
) -> WorthServerCompatibilityUploadExecutionInput {
    WorthServerCompatibilityUploadExecutionInput::new(
        prepared_request(
            server,
            upload_input(tenant_id, workspace_id, branch_id, operation_name, boundary),
        ),
        operation_name,
        upload,
    )
}

pub(crate) fn compat_upload_success(
    outcome: WorthServerCompatibilityUploadOutcome<WorthServerCompatibilityUpload>,
) -> WorthServerCompatibilityUpload {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility upload success, got {other:?}"),
    }
}

pub(crate) fn compat_upload_denied(
    outcome: WorthServerCompatibilityUploadOutcome<WorthServerCompatibilityUpload>,
) -> WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(value) => value,
        other => panic!("expected compatibility upload denial, got {other:?}"),
    }
}
