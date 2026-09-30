use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use serde_json::json;
use worth_proof::TransitionOutcome;
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerCompatHttpRouteFamily, WorthServerCompatibilityPreparedRequest,
    WorthServerCompatibilityRequestInput, WorthServerConfig, WorthServerMiddlewareConfig,
    WorthServerProductApplicationAdapter, WorthServerProductApplicationAdapterRegistration,
    WorthServerProductOperationBasisKind, WorthServerProductOperationDeclaration,
    WorthServerProductOperationErrorMaps, WorthServerProductOperationPayload,
    WorthServerProductOperationSuccess, WorthServerProductOperationSupportSnapshot,
    WorthServerProductSessionClock, WorthServerProductSessionTerminationObserver,
    WorthServerQueryHandoffConfig, WorthServerRequestContextConfig, WorthServerWorthNativeSession,
    WorthServerWorthNativeSessionInput,
};

#[path = "../product_result/schema_bound_json.rs"]
pub(crate) mod schema_bound_json;

mod manual_clock;

pub use manual_clock::ManualProductSessionClock;

pub fn build_server(
    registrations: Vec<WorthServerProductApplicationAdapterRegistration>,
) -> WorthServer {
    build_server_with_clock_and_observers(registrations, None, Vec::new())
}

pub fn build_server_with_clock(
    registrations: Vec<WorthServerProductApplicationAdapterRegistration>,
    product_session_clock: Option<Arc<dyn WorthServerProductSessionClock>>,
) -> WorthServer {
    build_server_with_clock_and_observers(registrations, product_session_clock, Vec::new())
}

pub fn build_server_with_clock_and_observers(
    registrations: Vec<WorthServerProductApplicationAdapterRegistration>,
    product_session_clock: Option<Arc<dyn WorthServerProductSessionClock>>,
    termination_observers: Vec<Arc<dyn WorthServerProductSessionTerminationObserver>>,
) -> WorthServer {
    let mut builder = WorthServer::builder()
        .with_config(base_config())
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .register_product_adapters(registrations);
    if let Some(product_session_clock) = product_session_clock {
        builder = builder.with_product_session_clock(product_session_clock);
    }
    for observer in termination_observers {
        builder = builder.observe_product_session_termination(observer);
    }
    builder
        .build()
        .expect("phase ten product-session server should build")
}

pub fn prepared_session_request(
    server: &WorthServer,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
) -> WorthServerCompatibilityPreparedRequest {
    prepared_request(
        server,
        workspace_id,
        branch_id,
        WorthServerCompatHttpRouteFamily::Mutation,
        "POST",
        &format!("/compat/mutations/{operation_name}"),
        None,
    )
}

pub fn prepared_product_read_request(
    server: &WorthServer,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    basis_digest: Option<&str>,
) -> WorthServerCompatibilityPreparedRequest {
    prepared_request(
        server,
        workspace_id,
        branch_id,
        WorthServerCompatHttpRouteFamily::Read,
        "GET",
        &format!("/compat/reads/{operation_name}"),
        basis_digest,
    )
}

pub fn prepared_product_mutation_request(
    server: &WorthServer,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    basis_digest: Option<&str>,
) -> WorthServerCompatibilityPreparedRequest {
    prepared_request(
        server,
        workspace_id,
        branch_id,
        WorthServerCompatHttpRouteFamily::Mutation,
        "POST",
        &format!("/compat/mutations/{operation_name}"),
        basis_digest,
    )
}

fn prepared_request(
    server: &WorthServer,
    workspace_id: &str,
    branch_id: &str,
    route_family: WorthServerCompatHttpRouteFamily,
    method: &str,
    path: &str,
    basis_digest: Option<&str>,
) -> WorthServerCompatibilityPreparedRequest {
    let mut builder = WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(route_family)
        .with_method(method)
        .with_path(path)
        .with_header("accept", "application/json");
    if let Some(basis_digest) = basis_digest {
        builder = builder.with_query_pair("basis", basis_digest);
    }
    match server.compat_http().prepare_request(
        builder
            .build()
            .expect("compatibility request input should validate"),
    ) {
        TransitionOutcome::Success(prepared_request) => prepared_request,
        other => panic!("expected prepared compatibility request, got {other:?}"),
    }
}

pub fn direct_session(
    server: &WorthServer,
    workspace_id: &str,
    branch_id: &str,
) -> WorthServerWorthNativeSession {
    match server.worth_native().session(
        WorthServerWorthNativeSessionInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id(workspace_id)
            .with_branch_id(branch_id)
            .build()
            .expect("Worth-native session input should validate"),
    ) {
        TransitionOutcome::Success(session) => session,
        other => panic!("expected Worth-native session, got {other:?}"),
    }
}

pub fn preview_payload() -> WorthServerProductOperationPayload {
    WorthServerProductOperationPayload::json(
        "product-editor.render-preview.v1",
        json!({ "document": "doc-7" }),
    )
}

pub fn apply_payload() -> WorthServerProductOperationPayload {
    apply_payload_with_title("Rename")
}

pub fn apply_payload_with_title(title: &str) -> WorthServerProductOperationPayload {
    WorthServerProductOperationPayload::json("product-editor.apply.v1", json!({ "title": title }))
}

pub fn prepared_product_mutation_request_with_basis_and_header(
    server: &WorthServer,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    basis_digest: &str,
    header_name: &str,
    header_value: &str,
) -> WorthServerCompatibilityPreparedRequest {
    let builder = WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id(workspace_id)
        .with_branch_id(branch_id)
        .with_route_family(WorthServerCompatHttpRouteFamily::Mutation)
        .with_method("POST")
        .with_path(format!("/compat/mutations/{operation_name}"))
        .with_query_pair("basis", basis_digest)
        .with_header("accept", "application/json")
        .with_header(header_name, header_value);
    match server.compat_http().prepare_request(
        builder
            .build()
            .expect("compatibility request input should validate"),
    ) {
        TransitionOutcome::Success(prepared_request) => prepared_request,
        other => panic!("expected prepared compatibility request, got {other:?}"),
    }
}

pub fn session_backed_editor_registration(
    calls: Arc<AtomicUsize>,
) -> WorthServerProductApplicationAdapterRegistration {
    WorthServerProductApplicationAdapterRegistration::new(
        "session-backed-editor",
        Arc::new(SessionBackedEditorAdapter { calls }),
    )
    .with_operation(declared(
        WorthServerProductOperationDeclaration::product_read(
            "product_editor.render_preview",
            "product-editor.render-preview.v1",
            result_contract("product-editor.render-preview.result.v1"),
            WorthServerProductOperationBasisKind::ProductSessionDerived,
            WorthServerProductOperationSupportSnapshot::production_admitted("preview-session"),
        ),
    ))
    .with_operation(declared(
        WorthServerProductOperationDeclaration::product_mutation(
            "product_editor.apply",
            "product-editor.apply.v1",
            result_contract("product-editor.apply.result.v1"),
            WorthServerProductOperationBasisKind::ProductSessionDerived,
            WorthServerProductOperationSupportSnapshot::production_admitted("apply-session"),
            "draft",
        ),
    ))
}

fn base_config() -> WorthServerConfig {
    WorthServerConfig::builder()
        .with_bind_address(([127, 0, 0, 1], 8080).into())
        .with_request_context_config(
            WorthServerRequestContextConfig::builder()
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
                .build()
                .expect("query handoff config should validate"),
        )
        .build()
        .expect("server config should validate")
}

fn declared(
    declaration: WorthServerProductOperationDeclaration,
) -> WorthServerProductOperationDeclaration {
    declaration.with_error_map(WorthServerProductOperationErrorMaps::passthrough())
}

#[derive(Debug)]
struct SessionBackedEditorAdapter {
    calls: Arc<AtomicUsize>,
}

impl WorthServerProductApplicationAdapter for SessionBackedEditorAdapter {
    fn execute_with_lease(
        &self,
        operation: &worth_server::WorthServerScheduledProductOperation,
        _lease: &worth_execution::ExecutionResourceLease<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<
        Result<
            WorthServerProductOperationSuccess,
            worth_server::WorthServerProductAdapterExecutionError,
        >,
        worth_execution::MapKernelStop,
    > {
        context.checkpoint(0)?;
        Ok(self.execute(operation))
    }

    fn execute(
        &self,
        operation: &worth_server::WorthServerScheduledProductOperation,
    ) -> Result<
        WorthServerProductOperationSuccess,
        worth_server::WorthServerProductAdapterExecutionError,
    > {
        self.calls.fetch_add(1, Ordering::Relaxed);
        schema_bound_json::publish_schema_bound_json(
            operation.plan().declaration().operation_name(),
            operation.plan().declaration().result_contract(),
            operation
                .plan()
                .declaration()
                .result_contract()
                .schema()
                .identity(),
            json!({
                "operation": operation.plan().declaration().operation_name(),
                "product_session": operation
                    .plan()
                    .operation_admission()
                    .operation_request()
                    .identity()
                    .product_session_identity()
                    .unwrap_or("none"),
            }),
        )
    }
}

fn result_contract(schema_identity: &str) -> worth_server::WorthServerProductResultContract {
    worth_server::WorthServerProductResultContract::canonical_json(schema_identity, 1, 16 * 1024)
        .expect("session-backed result contract should validate")
}
