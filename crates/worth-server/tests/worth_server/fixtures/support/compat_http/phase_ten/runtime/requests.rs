use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServer, WorthServerBinaryDownloadExecutionInput, WorthServerBinaryDownloadRequest,
    WorthServerCompatHttpRouteFamily, WorthServerCompatibilityExecutionInput,
    WorthServerCompatibilityPreparedRequest, WorthServerCompatibilityRequestInput,
    WorthServerCompatibilityUploadExecutionInput, WorthServerMultipartUpload,
};

pub(crate) fn prepared_request(
    server: &WorthServer,
    request: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(request) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compatibility request, got {other:?}"),
    }
}

pub(crate) fn compat_read_execution_input(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerCompatibilityExecutionInput {
    WorthServerCompatibilityExecutionInput::new(
        prepared_request(
            server,
            WorthServerCompatibilityRequestInput::builder()
                .with_authenticated_principal_id("principal-10")
                .with_tenant_id(tenant_id)
                .with_workspace_id(workspace_id)
                .with_branch_id(branch_id)
                .with_route_family(WorthServerCompatHttpRouteFamily::Read)
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

pub(crate) fn compat_inspection_execution_input(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerCompatibilityExecutionInput {
    let input = WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-10")
        .with_tenant_id(tenant_id)
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(WorthServerCompatHttpRouteFamily::Read)
        .with_method("GET")
        .with_path(format!("/compat/reads/{operation_name}"))
        .with_header("accept", "application/json")
        .with_query_pair("inspect", "true")
        .with_diagnostics_profile(diagnostics_profile)
        .build()
        .expect("phase ten inspect input should validate");
    WorthServerCompatibilityExecutionInput::new(prepared_request(server, input), operation_name)
}

pub(crate) fn compat_upload_execution_input(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
    upload: WorthServerMultipartUpload,
) -> WorthServerCompatibilityUploadExecutionInput {
    let input = WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-10")
        .with_tenant_id(tenant_id)
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(WorthServerCompatHttpRouteFamily::Upload)
        .with_method("POST")
        .with_path(format!("/compat/uploads/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type("multipart/form-data; boundary=phase-ten")
        .with_body_present(true)
        .with_diagnostics_profile(diagnostics_profile)
        .build()
        .expect("phase ten upload input should validate");
    WorthServerCompatibilityUploadExecutionInput::new(
        prepared_request(server, input),
        operation_name,
        upload,
    )
}

pub(crate) fn compat_download_execution_input(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
    request: WorthServerBinaryDownloadRequest,
) -> WorthServerBinaryDownloadExecutionInput {
    let input = WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-10")
        .with_tenant_id(tenant_id)
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(WorthServerCompatHttpRouteFamily::Download)
        .with_method("GET")
        .with_path(format!("/compat/downloads/{operation_name}"))
        .with_header("accept", "application/octet-stream")
        .with_diagnostics_profile(diagnostics_profile)
        .build()
        .expect("phase ten download input should validate");
    WorthServerBinaryDownloadExecutionInput::new(
        prepared_request(server, input),
        operation_name,
        request,
    )
}
