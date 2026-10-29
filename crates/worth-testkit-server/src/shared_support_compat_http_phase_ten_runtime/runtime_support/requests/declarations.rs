pub use {worth_foundational::facade::DiagnosticRichnessProfile};
pub use {worth_proof::TransitionOutcome};
pub use {worth_server::WorthServer, worth_server::WorthServerBinaryDownloadExecutionInput, worth_server::WorthServerBinaryDownloadRequest, worth_server::WorthServerCompatHttpRouteFamily, worth_server::WorthServerCompatibilityExecutionInput, worth_server::WorthServerCompatibilityPreparedRequest, worth_server::WorthServerCompatibilityRequestInput, worth_server::WorthServerCompatibilityUploadExecutionInput, worth_server::WorthServerMultipartUpload};

pub fn prepared_request(
    server: &WorthServer,
    request: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(request) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compatibility request, got {other:?}"),
    }
}

pub fn compat_read_execution_input(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerCompatibilityExecutionInput {
    worth_server::WorthServerCompatibilityExecutionInput::new(
        prepared_request(
            server,
            worth_server::WorthServerCompatibilityRequestInput::builder()
                .with_authenticated_principal_id("principal-10")
                .with_tenant_id(tenant_id)
                .with_workspace_id(workspace_id)
                .with_branch_id(branch_id)
                .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
                .with_method("GET")
                .with_path(format!("/compat/reads/{operation_name}"))
                .with_header("accept", "application/json")
                .with_diagnostics_profile(diagnostics_profile)
                .build()
                .expect("phase ten read input should validate"),
        ),
        operation_name,
    )
}

pub fn compat_inspection_execution_input(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerCompatibilityExecutionInput {
    let input = worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-10")
        .with_tenant_id(tenant_id)
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
        .with_method("GET")
        .with_path(format!("/compat/reads/{operation_name}"))
        .with_header("accept", "application/json")
        .with_query_pair("inspect", "true")
        .with_diagnostics_profile(diagnostics_profile)
        .build()
        .expect("phase ten inspect input should validate");
    worth_server::WorthServerCompatibilityExecutionInput::new(prepared_request(server, input), operation_name)
}

pub fn compat_upload_execution_input(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
    upload: WorthServerMultipartUpload,
) -> WorthServerCompatibilityUploadExecutionInput {
    let input = worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-10")
        .with_tenant_id(tenant_id)
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Upload)
        .with_method("POST")
        .with_path(format!("/compat/uploads/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type("multipart/form-data; boundary=phase-ten")
        .with_body_present(true)
        .with_diagnostics_profile(diagnostics_profile)
        .build()
        .expect("phase ten upload input should validate");
    worth_server::WorthServerCompatibilityUploadExecutionInput::new(
        prepared_request(server, input),
        operation_name,
        upload,
    )
}

pub fn compat_download_execution_input(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
    request: WorthServerBinaryDownloadRequest,
) -> WorthServerBinaryDownloadExecutionInput {
    let input = worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-10")
        .with_tenant_id(tenant_id)
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Download)
        .with_method("GET")
        .with_path(format!("/compat/downloads/{operation_name}"))
        .with_header("accept", "application/octet-stream")
        .with_diagnostics_profile(diagnostics_profile)
        .build()
        .expect("phase ten download input should validate");
    worth_server::WorthServerBinaryDownloadExecutionInput::new(
        prepared_request(server, input),
        operation_name,
        request,
    )
}