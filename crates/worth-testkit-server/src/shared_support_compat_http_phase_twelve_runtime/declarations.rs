pub use {worth_foundational::facade::DiagnosticRichnessProfile};
pub use {worth_proof::TransitionOutcome};
pub use {worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerBinaryDownloadExecutionInput, worth_server::WorthServerBinaryDownloadRequest, worth_server::WorthServerCompatHttpRouteFamily, worth_server::WorthServerCompatibilityPreparedRequest, worth_server::WorthServerCompatibilityRequestInput, worth_server::WorthServerCompatibilityUploadExecutionInput, worth_server::WorthServerConfig, worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerMultipartUpload, worth_server::WorthServerQueryHandoffConfig, worth_server::WorthServerRequestContextConfig};

pub use {crate::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider};

pub fn build_phase_twelve_server() -> WorthServer {
    build_server_with_max_diagnostics(worth_foundational::facade::DiagnosticRichnessProfile::Forensic)
}

pub fn build_phase_twelve_budget_limited_server() -> WorthServer {
    build_server_with_max_diagnostics(worth_foundational::facade::DiagnosticRichnessProfile::Standard)
}

pub fn build_server_with_max_diagnostics(
    maximum_diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(worth_foundational::facade::DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    worth_server::WorthServerMiddlewareConfig::builder()
                        .with_compat_http_maximum_diagnostics_profile(maximum_diagnostics_profile)
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
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("server should build")
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

pub fn read_request(
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerCompatibilityRequestInput {
    worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-12")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
        .with_method("GET")
        .with_path("/compat/reads/tasks")
        .with_header("accept", "application/json")
        .with_diagnostics_profile(diagnostics_profile)
        .build()
        .expect("phase twelve read input should validate")
}

pub fn head_read_request() -> WorthServerCompatibilityRequestInput {
    worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-12")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
        .with_method("HEAD")
        .with_path("/compat/reads/tasks")
        .with_header("accept", "application/json")
        .build()
        .expect("phase twelve head read input should validate")
}

pub fn upload_input(
    server: &WorthServer,
    upload: WorthServerMultipartUpload,
) -> WorthServerCompatibilityUploadExecutionInput {
    let request = worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-12")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Upload)
        .with_method("POST")
        .with_path("/compat/uploads/blob")
        .with_header("accept", "application/json")
        .with_body_content_type("multipart/form-data; boundary=phase-twelve")
        .with_body_present(true)
        .build()
        .expect("phase twelve upload input should validate");
    worth_server::WorthServerCompatibilityUploadExecutionInput::new(
        prepared_request(server, request),
        "blob",
        upload,
    )
}

pub fn download_input(
    server: &WorthServer,
    request: WorthServerBinaryDownloadRequest,
) -> WorthServerBinaryDownloadExecutionInput {
    let compat_request = worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-12")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Download)
        .with_method("GET")
        .with_path("/compat/downloads/blob")
        .with_header("accept", "application/octet-stream")
        .build()
        .expect("phase twelve download input should validate");
    worth_server::WorthServerBinaryDownloadExecutionInput::new(
        prepared_request(server, compat_request),
        "blob",
        request,
    )
}

pub fn drip_fed_upload() -> WorthServerMultipartUpload {
    use {serde_json::json};
    use {worth_server::WorthServerUploadChunk, worth_server::WorthServerUploadManifest, worth_server::WorthServerUploadPart, worth_server::WorthServerUploadTransferMode};

    let part = (0..33).fold(
        WorthServerUploadPart::file("blob")
            .with_content_type("application/octet-stream")
            .with_declared_length(33)
            .with_transfer_mode(WorthServerUploadTransferMode::UnknownLength)
            .with_body_bytes(vec![b'a'; 33]),
        |part, _| part.with_wire_chunk(WorthServerUploadChunk::new(vec![1])),
    );
    worth_server::WorthServerMultipartUpload::new(
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