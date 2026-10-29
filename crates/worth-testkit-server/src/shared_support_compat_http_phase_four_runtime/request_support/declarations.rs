pub use {worth_proof::TransitionOutcome};
pub use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerCompatHttpRouteFamily, worth_server::WorthServerCompatibilityExecutionInput, worth_server::WorthServerCompatibilityPreparedRequest, worth_server::WorthServerCompatibilityRequestInput, worth_server::WorthServerConfig, worth_server::WorthServerQueryHandoffConfig, worth_server::WorthServerQueryWorkspaceProvider, worth_server::WorthServerRequestContextConfig, worth_server::WorthServerStreamSelection, worth_server::WorthServerStreamingResponse};

pub use {crate::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider};

pub use {crate::shared_support_compat_http_phase_four_runtime::StreamingDatasetWorkspaceProvider};

pub fn build_phase_four_server() -> WorthServer {
    build_phase_four_server_with_workspace_provider(TestWorkspaceProvider)
}

pub fn build_phase_four_server_with_workspace_provider(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
) -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(worth_server::request_context::DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("request context config should validate"),
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

pub fn prepared_stream_request(
    server: &WorthServer,
    request: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(request) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compatibility streaming request, got {other:?}"),
    }
}

pub fn compat_stream_input(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCompatibilityExecutionInput {
    worth_server::WorthServerCompatibilityExecutionInput::new(
        prepared_stream_request(
            server,
            stream_input(operation_name)
                .build()
                .expect("compat stream input should validate structurally"),
        ),
        operation_name,
    )
}

pub fn compat_stream_head_input(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCompatibilityExecutionInput {
    worth_server::WorthServerCompatibilityExecutionInput::new(
        prepared_stream_request(
            server,
            stream_input(operation_name)
                .with_method("HEAD")
                .build()
                .expect("compat stream head input should validate structurally"),
        ),
        operation_name,
    )
}

pub fn compat_read_input(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCompatibilityExecutionInput {
    worth_server::WorthServerCompatibilityExecutionInput::new(
        prepared_stream_request(
            server,
            read_input(operation_name)
                .build()
                .expect("compat read input should validate structurally"),
        ),
        operation_name,
    )
}

pub fn read_input(
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    base_input(worth_server::WorthServerCompatHttpRouteFamily::Read)
        .with_path(format!("/compat/reads/{operation_name}"))
}

pub fn stream_input(
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    base_input(worth_server::WorthServerCompatHttpRouteFamily::Streaming)
        .with_path(format!("/compat/streams/{operation_name}"))
}

pub fn base_input(
    route_family: WorthServerCompatHttpRouteFamily,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(route_family)
        .with_method("GET")
        .with_header("accept", "application/json")
}

pub fn streaming_response_success(
    response: worth_server::WorthServerCompatibilityExecutionOutcome<WorthServerStreamingResponse>,
) -> WorthServerStreamingResponse {
    match response {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility streaming success, got {other:?}"),
    }
}

pub fn oversized_streaming_provider(
    row_count: usize,
    payload_width: usize,
) -> StreamingDatasetWorkspaceProvider {
    crate::shared_support_compat_http_phase_four_runtime::StreamingDatasetWorkspaceProvider::new(row_count, payload_width)
}

pub fn default_stream_selection() -> WorthServerStreamSelection {
    worth_server::WorthServerStreamSelection::incremental().with_chunk_bytes(32)
}