pub use {worth_proof::TransitionOutcome};
pub use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerCompatibilityExecutionInput, worth_server::WorthServerCompatibilityPreparedRequest, worth_server::WorthServerCompatibilityRequestInput, worth_server::WorthServerConfig, worth_server::WorthServerQueryHandoffConfig, worth_server::WorthServerQueryWorkspaceProvider, worth_server::WorthServerRequestContextConfig};

pub use {crate::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider};
pub use {crate::shared_support_worth_native_assertions::worth_native_session};
pub use {crate::shared_support_worth_native_runtime::worth_native_session_input_builder};

pub fn build_phase_two_server() -> WorthServer {
    build_phase_two_server_with_workspace_provider(TestWorkspaceProvider)
}

pub fn build_phase_two_server_with_workspace_provider(
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

pub fn prepared_read_request(
    server: &WorthServer,
    request: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(request) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compat request, got {other:?}"),
    }
}

pub fn read_input(
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
        .with_method("GET")
        .with_path(format!("/compat/reads/{operation_name}"))
        .with_header("accept", "application/json")
}

pub fn compat_execution_input(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCompatibilityExecutionInput {
    worth_server::WorthServerCompatibilityExecutionInput::new(
        prepared_read_request(
            server,
            read_input(operation_name)
                .build()
                .expect("compat read input should validate"),
        ),
        operation_name,
    )
}

pub fn branch_head_execution_input(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCompatibilityExecutionInput {
    worth_server::WorthServerCompatibilityExecutionInput::new(
        prepared_read_request(
            server,
            read_input(operation_name)
                .with_method("HEAD")
                .build()
                .expect("compat HEAD input should validate"),
        ),
        operation_name,
    )
}

pub fn worth_native_named_read(
    server: &WorthServer,
    operation_name: &str,
) -> (
    worth_server::WorthServerWorthNativeSession,
    worth_server::WorthServerAdmittedDirectDeclaration,
) {
    let session = worth_native_session(server);
    let declaration = session
        .declarations()
        .read(worth_server::WorthServerDirectDeclaration::named_read(
            operation_name,
        ))
        .expect("named read should prepare")
        .admit()
        .expect("named read should admit");
    (session, declaration)
}

#[allow(dead_code)]
pub fn worth_native_branch_session(
    server: &WorthServer,
    branch_id: &str,
) -> worth_server::WorthServerWorthNativeSession {
    match server.worth_native().session(
        worth_native_session_input_builder()
            .with_branch_id(branch_id)
            .build()
            .expect("branch session input should validate"),
    ) {
        worth_proof::TransitionOutcome::Success(session) => session,
        other => panic!("expected Worth-native branch session, got {other:?}"),
    }
}