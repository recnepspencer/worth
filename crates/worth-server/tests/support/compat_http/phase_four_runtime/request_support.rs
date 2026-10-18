use worth_proof::TransitionOutcome;
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerCompatHttpRouteFamily, WorthServerCompatibilityExecutionInput,
    WorthServerCompatibilityPreparedRequest, WorthServerCompatibilityRequestInput,
    WorthServerConfig, WorthServerQueryHandoffConfig, WorthServerQueryWorkspaceProvider,
    WorthServerRequestContextConfig, WorthServerStreamSelection, WorthServerStreamingResponse,
};

use crate::shared_support_query_handoff_runtime::TestWorkspaceProvider;

use super::StreamingDatasetWorkspaceProvider;

pub(crate) fn build_phase_four_server() -> WorthServer {
    build_phase_four_server_with_workspace_provider(TestWorkspaceProvider)
}

pub(crate) fn build_phase_four_server_with_workspace_provider(
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

pub(crate) fn prepared_stream_request(
    server: &WorthServer,
    request: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(request) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compatibility streaming request, got {other:?}"),
    }
}

pub(crate) fn compat_stream_input(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCompatibilityExecutionInput {
    WorthServerCompatibilityExecutionInput::new(
        prepared_stream_request(
            server,
            stream_input(operation_name)
                .build()
                .expect("compat stream input should validate structurally"),
        ),
        operation_name,
    )
}

pub(crate) fn compat_stream_head_input(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCompatibilityExecutionInput {
    WorthServerCompatibilityExecutionInput::new(
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

pub(crate) fn compat_read_input(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCompatibilityExecutionInput {
    WorthServerCompatibilityExecutionInput::new(
        prepared_stream_request(
            server,
            read_input(operation_name)
                .build()
                .expect("compat read input should validate structurally"),
        ),
        operation_name,
    )
}

pub(crate) fn read_input(
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    base_input(WorthServerCompatHttpRouteFamily::Read)
        .with_path(format!("/compat/reads/{operation_name}"))
}

pub(crate) fn stream_input(
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    base_input(WorthServerCompatHttpRouteFamily::Streaming)
        .with_path(format!("/compat/streams/{operation_name}"))
}

fn base_input(
    route_family: WorthServerCompatHttpRouteFamily,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(route_family)
        .with_method("GET")
        .with_header("accept", "application/json")
}

pub(crate) fn streaming_response_success(
    response: worth_server::WorthServerCompatibilityExecutionOutcome<WorthServerStreamingResponse>,
) -> WorthServerStreamingResponse {
    match response {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility streaming success, got {other:?}"),
    }
}

pub(crate) fn oversized_streaming_provider(
    row_count: usize,
    payload_width: usize,
) -> StreamingDatasetWorkspaceProvider {
    StreamingDatasetWorkspaceProvider::new(row_count, payload_width)
}

pub(crate) fn default_stream_selection() -> WorthServerStreamSelection {
    WorthServerStreamSelection::incremental().with_chunk_bytes(32)
}
