pub use worth_proof::TransitionOutcome;
pub use {
    worth_server::request_context::DiagnosticRichnessProfile,
    worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface,
    worth_server::WorthServer, worth_server::WorthServerCompatHttpRouteFamily,
    worth_server::WorthServerCompatibilityPreparedRequest,
    worth_server::WorthServerCompatibilityRequestInput,
    worth_server::WorthServerCompatibilityUpload,
    worth_server::WorthServerCompatibilityUploadExecutionInput,
    worth_server::WorthServerCompatibilityUploadOutcome, worth_server::WorthServerConfig,
    worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerMultipartUpload,
    worth_server::WorthServerPreparedMultipartUpload, worth_server::WorthServerQueryHandoffConfig,
    worth_server::WorthServerQueryHandoffDenial, worth_server::WorthServerQueryWorkspaceProvider,
    worth_server::WorthServerRequestContextConfig,
};

pub fn build_phase_six_server_with_workspace_provider(
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

pub fn upload_input(
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    boundary: &str,
) -> WorthServerCompatibilityRequestInput {
    worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id(tenant_id)
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Upload)
        .with_method("POST")
        .with_path(format!("/compat/uploads/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type(format!("multipart/form-data; boundary={boundary}"))
        .with_body_present(true)
        .build()
        .expect("phase six upload input should validate structurally")
}

pub fn prepared_request(
    server: &WorthServer,
    request: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(request) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compatibility request, got {other:?}"),
    }
}

pub fn prepared_upload(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    boundary: &str,
    upload: WorthServerMultipartUpload,
) -> WorthServerPreparedMultipartUpload {
    match server.compat_http().prepare_upload(
        worth_server::WorthServerCompatibilityUploadExecutionInput::new(
            prepared_request(
                server,
                upload_input(tenant_id, workspace_id, branch_id, operation_name, boundary),
            ),
            operation_name,
            upload,
        ),
    ) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared upload, got {other:?}"),
    }
}

pub fn compat_upload_execution_input(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    boundary: &str,
    upload: WorthServerMultipartUpload,
) -> WorthServerCompatibilityUploadExecutionInput {
    worth_server::WorthServerCompatibilityUploadExecutionInput::new(
        prepared_request(
            server,
            upload_input(tenant_id, workspace_id, branch_id, operation_name, boundary),
        ),
        operation_name,
        upload,
    )
}

pub fn compat_upload_success(
    outcome: WorthServerCompatibilityUploadOutcome<WorthServerCompatibilityUpload>,
) -> WorthServerCompatibilityUpload {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility upload success, got {other:?}"),
    }
}

pub fn compat_upload_denied(
    outcome: WorthServerCompatibilityUploadOutcome<WorthServerCompatibilityUpload>,
) -> WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected compatibility upload denial, got {other:?}"),
    }
}
