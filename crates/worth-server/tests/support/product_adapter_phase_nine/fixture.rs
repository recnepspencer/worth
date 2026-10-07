#![allow(dead_code)]

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use serde_json::{json, Value};
use worth_proof::TransitionOutcome;
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerCompatHttpRouteFamily, WorthServerCompatibilityPreparedRequest,
    WorthServerCompatibilityRequestInput, WorthServerConfig, WorthServerMiddlewareConfig,
    WorthServerProductAdapterExecutionError, WorthServerProductApplicationAdapter,
    WorthServerProductApplicationAdapterRegistration, WorthServerProductOperationAuthorizer,
    WorthServerProductOperationBasisKind, WorthServerProductOperationDeclaration,
    WorthServerProductOperationDenial, WorthServerProductOperationErrorMaps,
    WorthServerProductOperationPayload, WorthServerProductOperationSuccess,
    WorthServerProductOperationSupportSnapshot, WorthServerProductPayloadSchemaValidator,
    WorthServerProductResultContract, WorthServerProductSession,
    WorthServerProductSessionCreationRequest, WorthServerQueryHandoffConfig,
    WorthServerQueryWorkspaceProvider, WorthServerRequestContextConfig,
    WorthServerTransportCallerVerifier, WorthServerWorthNativeSession,
    WorthServerWorthNativeSessionInput,
};

#[path = "../product_result/schema_bound_json.rs"]
pub(crate) mod schema_bound_json;

#[path = "../query_handoff/runtime.rs"]
mod query_handoff_runtime;

pub fn build_server(
    registrations: Vec<WorthServerProductApplicationAdapterRegistration>,
) -> WorthServer {
    build_server_with_query_workspace_provider(
        registrations,
        Arc::new(query_handoff_runtime::TestWorkspaceProvider),
    )
}

pub fn build_server_with_operation_authority(
    registrations: Vec<WorthServerProductApplicationAdapterRegistration>,
    verifier: Arc<dyn WorthServerTransportCallerVerifier>,
    authorizer: Arc<dyn WorthServerProductOperationAuthorizer>,
) -> WorthServer {
    WorthServer::builder()
        .with_config(base_config_with_workspace_provider(Arc::new(
            query_handoff_runtime::TestWorkspaceProvider,
        )))
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .register_product_adapters(registrations)
        .with_transport_caller_verifier(verifier)
        .with_product_operation_authorizer(authorizer)
        .build()
        .expect("authorized product server should build")
}

pub fn build_server_with_query_workspace_provider(
    registrations: Vec<WorthServerProductApplicationAdapterRegistration>,
    workspace_provider: Arc<dyn WorthServerQueryWorkspaceProvider>,
) -> WorthServer {
    WorthServer::builder()
        .with_config(base_config_with_workspace_provider(workspace_provider))
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .register_product_adapters(registrations)
        .build()
        .expect("phase nine product server should build")
}

pub fn direct_session(server: &WorthServer) -> WorthServerWorthNativeSession {
    match server.worth_native().session(
        WorthServerWorthNativeSessionInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .build()
            .expect("Worth-native session input should validate"),
    ) {
        TransitionOutcome::Success(session) => session,
        other => panic!("expected Worth-native session, got {other:?}"),
    }
}

pub fn prepared_read_request(
    server: &WorthServer,
    operation_name: &str,
    basis_digest: Option<&str>,
) -> WorthServerCompatibilityPreparedRequest {
    let mut builder = WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(WorthServerCompatHttpRouteFamily::Read)
        .with_method("GET")
        .with_path(format!("/compat/reads/{operation_name}"))
        .with_header("accept", "application/json");
    if let Some(basis_digest) = basis_digest {
        builder = builder.with_query_pair("basis", basis_digest);
    }
    match server.compat_http().prepare_request(
        builder
            .build()
            .expect("compat read request should validate"),
    ) {
        TransitionOutcome::Success(request) => request,
        other => panic!("expected prepared read request, got {other:?}"),
    }
}

pub fn prepared_mutation_request(
    server: &WorthServer,
    operation_name: &str,
    basis_digest: Option<&str>,
) -> WorthServerCompatibilityPreparedRequest {
    let mut builder = WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(WorthServerCompatHttpRouteFamily::Mutation)
        .with_method("POST")
        .with_path(format!("/compat/mutations/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type("application/json")
        .with_body_present(true);
    if let Some(basis_digest) = basis_digest {
        builder = builder.with_query_pair("basis", basis_digest);
    }
    match server.compat_http().prepare_request(
        builder
            .build()
            .expect("compat mutation request should validate"),
    ) {
        TransitionOutcome::Success(request) => request,
        other => panic!("expected prepared mutation request, got {other:?}"),
    }
}

pub fn completed(
    result: Result<
        worth_server::WorthServerCompletedProductOperation,
        worth_server::WorthServerProductOperationSurfaceDenial,
    >,
) -> worth_server::WorthServerCompletedProductOperation {
    result.expect("product operation should complete through the server boundary")
}

pub fn open_mutation_product_session(
    session: &WorthServerWorthNativeSession,
    operation_name: &str,
    basis_digest: &str,
) -> WorthServerProductSession {
    session
        .product_sessions()
        .open_mutation(
            WorthServerProductSessionCreationRequest::for_operation(operation_name)
                .with_basis_digest(basis_digest)
                .with_expiry_seconds(300),
        )
        .expect("mutation product session should open through the server boundary")
}

pub fn editor_registration(
    calls: Option<Arc<AtomicUsize>>,
    validator: Option<Arc<dyn WorthServerProductPayloadSchemaValidator>>,
) -> WorthServerProductApplicationAdapterRegistration {
    let adapter = Arc::new(EditorAdapter {
        calls: calls.unwrap_or_else(|| Arc::new(AtomicUsize::new(0))),
    });
    let render = declared(WorthServerProductOperationDeclaration::product_read(
        "product_editor.render",
        "product-editor.render.v1",
        result_contract("product-editor.render.result.v1"),
        WorthServerProductOperationBasisKind::DurableProductDerived,
        WorthServerProductOperationSupportSnapshot::production_admitted("render-supported"),
    ));
    let apply = declared(WorthServerProductOperationDeclaration::product_mutation(
        "product_editor.apply",
        "product-editor.apply.v1",
        result_contract("product-editor.apply.result.v1"),
        WorthServerProductOperationBasisKind::DurableProductDerived,
        WorthServerProductOperationSupportSnapshot::production_admitted("apply-supported"),
        "draft",
    ));
    let apply = match validator {
        Some(validator) => apply.with_payload_validator(validator),
        None => apply,
    };
    WorthServerProductApplicationAdapterRegistration::new("editor-adapter", adapter)
        .with_operation(render)
        .with_operation(declared(
            WorthServerProductOperationDeclaration::product_read(
                "product_editor.select",
                "product-editor.select.v1",
                result_contract("product-editor.select.result.v1"),
                WorthServerProductOperationBasisKind::DurableProductDerived,
                WorthServerProductOperationSupportSnapshot::production_admitted("select-supported"),
            ),
        ))
        .with_operation(declared(
            WorthServerProductOperationDeclaration::product_read(
                "product_editor.available_actions",
                "product-editor.actions.v1",
                result_contract("product-editor.actions.result.v1"),
                WorthServerProductOperationBasisKind::DurableProductDerived,
                WorthServerProductOperationSupportSnapshot::production_admitted(
                    "actions-supported",
                ),
            ),
        ))
        .with_operation(apply)
        .with_operation(declared(
            WorthServerProductOperationDeclaration::product_mutation(
                "product_editor.finalize",
                "product-editor.finalize.v1",
                result_contract("product-editor.finalize.result.v1"),
                WorthServerProductOperationBasisKind::DurableProductDerived,
                WorthServerProductOperationSupportSnapshot::production_admitted(
                    "finalize-supported",
                ),
                "draft",
            ),
        ))
}

pub fn query_derived_editor_registration(
    calls: Option<Arc<AtomicUsize>>,
) -> WorthServerProductApplicationAdapterRegistration {
    let adapter = Arc::new(EditorAdapter {
        calls: calls.unwrap_or_else(|| Arc::new(AtomicUsize::new(0))),
    });
    WorthServerProductApplicationAdapterRegistration::new("query-derived-editor", adapter)
        .with_operation(declared(
            WorthServerProductOperationDeclaration::product_read(
                "product_editor.render",
                "product-editor.render.v1",
                result_contract("product-editor.render.result.v1"),
                WorthServerProductOperationBasisKind::QueryDerived,
                WorthServerProductOperationSupportSnapshot::production_admitted(
                    "render-query-derived",
                ),
            ),
        ))
        .with_operation(declared(
            WorthServerProductOperationDeclaration::product_mutation(
                "product_editor.apply",
                "product-editor.apply.v1",
                result_contract("product-editor.apply.result.v1"),
                WorthServerProductOperationBasisKind::QueryDerived,
                WorthServerProductOperationSupportSnapshot::production_admitted(
                    "apply-query-derived",
                ),
                "draft",
            ),
        ))
}

pub fn base_config() -> WorthServerConfig {
    base_config_with_workspace_provider(Arc::new(query_handoff_runtime::TestWorkspaceProvider))
}

pub fn base_config_with_workspace_provider(
    workspace_provider: Arc<dyn WorthServerQueryWorkspaceProvider>,
) -> WorthServerConfig {
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
                .with_workspace_provider_arc(workspace_provider)
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

pub fn result_contract(schema_identity: &str) -> WorthServerProductResultContract {
    WorthServerProductResultContract::canonical_json(schema_identity, 1, 16 * 1024)
        .expect("test product result contract should validate")
}

#[derive(Debug, Default)]
pub struct EditorAdapter {
    calls: Arc<AtomicUsize>,
}

impl WorthServerProductApplicationAdapter for EditorAdapter {
    fn execute_with_lease(
        &self,
        operation: &worth_server::WorthServerScheduledProductOperation,
        _lease: &worth_execution::ExecutionResourceLease<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<
        Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError>,
        worth_execution::MapKernelStop,
    > {
        context.checkpoint(0)?;
        Ok(self.execute(operation))
    }

    fn execute(
        &self,
        operation: &worth_server::WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        let plan = operation.plan();
        if let Some(reason_key) = plan
            .payload()
            .body()
            .get("deny_reason")
            .and_then(Value::as_str)
        {
            let denial =
                WorthServerProductOperationDenial::new(reason_key, "product-owned refusal");
            let denial = if reason_key == "product_editor.stale_basis" {
                denial.with_basis_mismatch("basis:6", "basis:7")
            } else {
                denial
            };
            return Err(WorthServerProductAdapterExecutionError::denied(denial));
        }
        if let Some(reason_key) = plan
            .payload()
            .body()
            .get("failure_reason")
            .and_then(Value::as_str)
        {
            return Err(WorthServerProductAdapterExecutionError::failed(
                reason_key,
                "product execution failed",
            ));
        }
        schema_bound_json::publish_schema_bound_json(
            plan.declaration().operation_name(),
            plan.declaration().result_contract(),
            plan.declaration().result_contract().schema().identity(),
            json!({
                "operation": plan.declaration().operation_name(),
                "scheduler_admission": operation.scheduler_admission().canonical_digest(),
                "basis": plan.operation_admission()
                    .operation_request()
                    .identity()
                    .basis_digest()
                    .unwrap_or("none"),
            }),
        )
    }
}

#[derive(Debug)]
pub struct RequireTitleValidator;

impl WorthServerProductPayloadSchemaValidator for RequireTitleValidator {
    fn validate(
        &self,
        payload: &WorthServerProductOperationPayload,
    ) -> Result<(), WorthServerProductOperationDenial> {
        if payload
            .body()
            .get("title")
            .and_then(Value::as_str)
            .is_some()
        {
            Ok(())
        } else {
            Err(WorthServerProductOperationDenial::new(
                "missing_title",
                "payload must include a title string",
            ))
        }
    }
}
