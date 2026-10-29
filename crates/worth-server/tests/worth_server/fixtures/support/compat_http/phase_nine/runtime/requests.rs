use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServer, WorthServerBinaryDownloadExecutionInput, WorthServerBinaryDownloadRequest,
    WorthServerCompatHttpRouteFamily, WorthServerCompatibilityExecutionInput,
    WorthServerCompatibilityPreparedRequest, WorthServerCompatibilityRequestInput,
    WorthServerCompatibilityUploadExecutionInput, WorthServerMultipartUpload,
};

pub(crate) fn read_input(
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerCompatibilityRequestInput {
    WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-9")
        .with_tenant_id(tenant_id)
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(WorthServerCompatHttpRouteFamily::Read)
        .with_method("GET")
        .with_path(format!("/compat/reads/{operation_name}"))
        .with_header("accept", "application/json")
        .with_diagnostics_profile(diagnostics_profile)
        .build()
        .expect("phase nine read input should validate")
}

pub(crate) fn inspect_input(
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerCompatibilityRequestInput {
    WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-9")
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
        .expect("phase nine inspection input should validate")
}

pub(crate) fn upload_input(
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerCompatibilityRequestInput {
    WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-9")
        .with_tenant_id(tenant_id)
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(WorthServerCompatHttpRouteFamily::Upload)
        .with_method("POST")
        .with_path(format!("/compat/uploads/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type("multipart/form-data; boundary=phase-nine")
        .with_body_present(true)
        .with_diagnostics_profile(diagnostics_profile)
        .build()
        .expect("phase nine upload input should validate")
}

pub(crate) fn download_input(
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerCompatibilityRequestInput {
    WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-9")
        .with_tenant_id(tenant_id)
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(WorthServerCompatHttpRouteFamily::Download)
        .with_method("GET")
        .with_path(format!("/compat/downloads/{operation_name}"))
        .with_header("accept", "application/octet-stream")
        .with_diagnostics_profile(diagnostics_profile)
        .build()
        .expect("phase nine download input should validate")
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
            read_input(
                tenant_id,
                workspace_id,
                branch_id,
                operation_name,
                diagnostics_profile,
            ),
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
    WorthServerCompatibilityExecutionInput::new(
        prepared_request(
            server,
            inspect_input(
                tenant_id,
                workspace_id,
                branch_id,
                operation_name,
                diagnostics_profile,
            ),
        ),
        operation_name,
    )
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
    WorthServerCompatibilityUploadExecutionInput::new(
        prepared_request(
            server,
            upload_input(
                tenant_id,
                workspace_id,
                branch_id,
                operation_name,
                diagnostics_profile,
            ),
        ),
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
    WorthServerBinaryDownloadExecutionInput::new(
        prepared_request(
            server,
            download_input(
                tenant_id,
                workspace_id,
                branch_id,
                operation_name,
                diagnostics_profile,
            ),
        ),
        operation_name,
        request,
    )
}
