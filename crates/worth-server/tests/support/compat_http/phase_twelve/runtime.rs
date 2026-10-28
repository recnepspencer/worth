use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_proof::TransitionOutcome;
use worth_server::{
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerBinaryDownloadExecutionInput, WorthServerBinaryDownloadRequest,
    WorthServerCompatHttpRouteFamily, WorthServerCompatibilityPreparedRequest,
    WorthServerCompatibilityRequestInput, WorthServerCompatibilityUploadExecutionInput,
    WorthServerConfig, WorthServerMiddlewareConfig, WorthServerMultipartUpload,
    WorthServerQueryHandoffConfig, WorthServerRequestContextConfig,
};

use crate::shared_support_query_handoff_runtime::TestWorkspaceProvider;

pub(crate) fn build_phase_twelve_server() -> WorthServer {
    build_server_with_max_diagnostics(DiagnosticRichnessProfile::Forensic)
}

pub(crate) fn build_phase_twelve_budget_limited_server() -> WorthServer {
    build_server_with_max_diagnostics(DiagnosticRichnessProfile::Standard)
}

fn build_server_with_max_diagnostics(
    maximum_diagnostics_profile: DiagnosticRichnessProfile,
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
                        .with_compat_http_maximum_diagnostics_profile(maximum_diagnostics_profile)
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
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("server should build")
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

pub(crate) fn read_request(
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerCompatibilityRequestInput {
    WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-12")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(WorthServerCompatHttpRouteFamily::Read)
        .with_method("GET")
        .with_path("/compat/reads/tasks")
        .with_header("accept", "application/json")
        .with_diagnostics_profile(diagnostics_profile)
        .build()
        .expect("phase twelve read input should validate")
}

pub(crate) fn head_read_request() -> WorthServerCompatibilityRequestInput {
    WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-12")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(WorthServerCompatHttpRouteFamily::Read)
        .with_method("HEAD")
        .with_path("/compat/reads/tasks")
        .with_header("accept", "application/json")
        .build()
        .expect("phase twelve head read input should validate")
}

pub(crate) fn upload_input(
    server: &WorthServer,
    upload: WorthServerMultipartUpload,
) -> WorthServerCompatibilityUploadExecutionInput {
    let request = WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-12")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(WorthServerCompatHttpRouteFamily::Upload)
        .with_method("POST")
        .with_path("/compat/uploads/blob")
        .with_header("accept", "application/json")
        .with_body_content_type("multipart/form-data; boundary=phase-twelve")
        .with_body_present(true)
        .build()
        .expect("phase twelve upload input should validate");
    WorthServerCompatibilityUploadExecutionInput::new(
        prepared_request(server, request),
        "blob",
        upload,
    )
}

pub(crate) fn download_input(
    server: &WorthServer,
    request: WorthServerBinaryDownloadRequest,
) -> WorthServerBinaryDownloadExecutionInput {
    let compat_request = WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-12")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(WorthServerCompatHttpRouteFamily::Download)
        .with_method("GET")
        .with_path("/compat/downloads/blob")
        .with_header("accept", "application/octet-stream")
        .build()
        .expect("phase twelve download input should validate");
    WorthServerBinaryDownloadExecutionInput::new(
        prepared_request(server, compat_request),
        "blob",
        request,
    )
}

pub(crate) fn drip_fed_upload() -> WorthServerMultipartUpload {
    use serde_json::json;
    use worth_server::{
        WorthServerUploadChunk, WorthServerUploadManifest, WorthServerUploadPart,
        WorthServerUploadTransferMode,
    };

    let part = (0..33).fold(
        WorthServerUploadPart::file("blob")
            .with_content_type("application/octet-stream")
            .with_declared_length(33)
            .with_transfer_mode(WorthServerUploadTransferMode::UnknownLength)
            .with_body_bytes(vec![b'a'; 33]),
        |part, _| part.with_wire_chunk(WorthServerUploadChunk::new(vec![1])),
    );
    WorthServerMultipartUpload::new(
        WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": "Task",
                "aspects": {
                    "identity.id": "drip",
                    "title.value": "Drip Feed"
                }
            }
        }))
        .with_file_part("blob"),
    )
    .with_part(part)
}
