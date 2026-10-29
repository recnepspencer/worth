pub use {std::sync::Arc, std::sync::Mutex};

pub use {serde_json::json, serde_json::Value};
pub use {worth_server::WorthServer, worth_server::WorthServerCompatibilityPreparedRequest, worth_server::WorthServerProductAdapterExecutionError, worth_server::WorthServerProductApplicationAdapter, worth_server::WorthServerProductApplicationAdapterRegistration, worth_server::WorthServerProductOperationAuthorizer, worth_server::WorthServerProductOperationDeclaration, worth_server::WorthServerProductOperationDenial, worth_server::WorthServerProductOperationErrorMaps, worth_server::WorthServerProductOperationInput, worth_server::WorthServerProductOperationPayload, worth_server::WorthServerProductOperationSuccess, worth_server::WorthServerProductOperationSupportSnapshot, worth_server::WorthServerProductSession, worth_server::WorthServerProductSessionCreationRequest, worth_server::WorthServerTransportCallerVerifier, worth_server::WorthServerWorthNativeSession, worth_server::WorthServerWorthNativeSessionInput};

#[allow(unused_imports)]
pub use {crate::shared_support_product_adapter_phase_nine_fixture as product_adapter_phase_nine_fixture};
pub use {crate::shared_support_product_result_schema_bound_json as schema_bound_json};

#[derive(Clone, Debug)]
pub struct StatefulEditorLikeBackend {
    pub state: Arc<Mutex<EditorLikeState>>,
}

impl Default for StatefulEditorLikeBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl StatefulEditorLikeBackend {
    pub fn new() -> Self {
        Self {
            state: std::sync::Arc::new(std::sync::Mutex::new(crate::shared_support_product_operation_phase_thirteen_fixture::EditorLikeState::default())),
        }
    }

    pub fn basis_digest(&self) -> String {
        self.state.lock().expect("editor state lock").basis_digest()
    }

    pub fn title(&self) -> String {
        self.state.lock().expect("editor state lock").title.clone()
    }

    pub fn registration(&self) -> WorthServerProductApplicationAdapterRegistration {
        worth_server::WorthServerProductApplicationAdapterRegistration::new(
            "stateful-editor-like",
            std::sync::Arc::new(self.clone()),
        )
        .with_operation(declared(
            worth_server::WorthServerProductOperationDeclaration::product_read(
                "product_editor.render",
                "product-editor.render.v1",
                crate::shared_support_product_adapter_phase_nine_fixture::result_contract(
                    "product-editor.render.result.v1",
                ),
                worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
                worth_server::WorthServerProductOperationSupportSnapshot::production_admitted("render-ready"),
            ),
        ))
        .with_operation(declared(
            worth_server::WorthServerProductOperationDeclaration::product_read(
                "product_editor.select",
                "product-editor.select.v1",
                crate::shared_support_product_adapter_phase_nine_fixture::result_contract(
                    "product-editor.select.result.v1",
                ),
                worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
                worth_server::WorthServerProductOperationSupportSnapshot::production_admitted("select-ready"),
            ),
        ))
        .with_operation(declared(
            worth_server::WorthServerProductOperationDeclaration::product_read(
                "product_editor.available_actions",
                "product-editor.actions.v1",
                crate::shared_support_product_adapter_phase_nine_fixture::result_contract(
                    "product-editor.actions.result.v1",
                ),
                worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
                worth_server::WorthServerProductOperationSupportSnapshot::production_admitted("actions-ready"),
            ),
        ))
        .with_operation(declared(
            worth_server::WorthServerProductOperationDeclaration::product_mutation(
                "product_editor.apply",
                "product-editor.apply.v1",
                crate::shared_support_product_adapter_phase_nine_fixture::result_contract(
                    "product-editor.apply.result.v1",
                ),
                worth_server::WorthServerProductOperationBasisKind::ProductSessionDerived,
                worth_server::WorthServerProductOperationSupportSnapshot::production_admitted("apply-ready"),
                "draft",
            ),
        ))
        .with_operation(declared(
            worth_server::WorthServerProductOperationDeclaration::product_mutation(
                "product_editor.finalize",
                "product-editor.finalize.v1",
                crate::shared_support_product_adapter_phase_nine_fixture::result_contract(
                    "product-editor.finalize.result.v1",
                ),
                worth_server::WorthServerProductOperationBasisKind::ProductSessionDerived,
                worth_server::WorthServerProductOperationSupportSnapshot::production_admitted("finalize-ready"),
                "draft",
            ),
        ))
    }
}

impl WorthServerProductApplicationAdapter for StatefulEditorLikeBackend {
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
        let mut state = self.state.lock().expect("editor state lock");
        let plan = operation.plan();
        let body = plan.payload().body();
        let request_basis = plan
            .operation_admission()
            .operation_request()
            .identity()
            .basis_digest()
            .unwrap_or("none");
        match plan.declaration().operation_name() {
            "product_editor.render" => product_success(
                plan,
                "product_editor.render",
                json!({
                    "request_basis": request_basis,
                    "observed_basis": state.basis_digest(),
                    "title": state.title,
                }),
            ),
            "product_editor.select" => {
                let node = body.get("node").and_then(serde_json::Value::as_str).unwrap_or("none");
                product_success(
                    plan,
                    "product_editor.select",
                    json!({
                        "node": node,
                        "title": state.title,
                        "basis": state.basis_digest(),
                    }),
                )
            }
            "product_editor.available_actions" => {
                let actions = if state.title == "Untitled" {
                    "apply"
                } else {
                    "apply,finalize"
                };
                product_success(
                    plan,
                    "product_editor.available_actions",
                    json!({
                        "actions": actions.split(',').collect::<Vec<_>>(),
                        "basis": state.basis_digest(),
                    }),
                )
            }
            "product_editor.apply" => {
                let title = body
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("Untitled");
                state.title = title.to_string();
                state.revision += 1;
                product_success(
                    plan,
                    "product_editor.apply",
                    json!({ "basis": state.basis_digest(), "title": state.title }),
                )
            }
            "product_editor.finalize" => {
                if body.get("confirm").and_then(serde_json::Value::as_bool) != Some(true) {
                    return Err(worth_server::WorthServerProductAdapterExecutionError::denied(
                        worth_server::WorthServerProductOperationDenial::new(
                            "product.finalize.confirm_required",
                            "finalize requires explicit confirmation",
                        ),
                    ));
                }
                state.revision += 1;
                product_success(
                    plan,
                    "product_editor.finalize",
                    json!({ "status": "finalized", "basis": state.basis_digest() }),
                )
            }
            operation_name => Err(worth_server::WorthServerProductAdapterExecutionError::failed(
                "unsupported_operation",
                format!("unsupported editor-like operation `{operation_name}`"),
            )),
        }
    }
}

pub fn build_server(backend: &StatefulEditorLikeBackend) -> WorthServer {
    crate::shared_support_product_adapter_phase_nine_fixture::build_server(vec![backend.registration()])
}

pub fn build_server_with_execution_authority(
    backend: &StatefulEditorLikeBackend,
    authority: Arc<worth_execution::ExecutionAuthority>,
) -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(crate::shared_support_product_adapter_phase_nine_fixture::base_config())
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .register_product_adapters(vec![backend.registration()])
        .with_execution_authority(authority)
        .build()
        .expect("phase thirteen server with execution authority")
}

pub fn build_server_with_operation_authority(
    backend: &StatefulEditorLikeBackend,
    verifier: Arc<dyn WorthServerTransportCallerVerifier>,
    authorizer: Arc<dyn WorthServerProductOperationAuthorizer>,
) -> WorthServer {
    crate::shared_support_product_adapter_phase_nine_fixture::build_server_with_operation_authority(
        vec![backend.registration()],
        verifier,
        authorizer,
    )
}

pub fn direct_session(server: &WorthServer) -> WorthServerWorthNativeSession {
    crate::shared_support_product_adapter_phase_nine_fixture::direct_session(server)
}

pub fn direct_session_with_proof(
    server: &WorthServer,
    proof_identity: &str,
) -> WorthServerWorthNativeSession {
    use {worth_proof::TransitionOutcome};

    match server.worth_native().session(
        worth_server::WorthServerWorthNativeSessionInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_application_authority_proof_identity(proof_identity)
            .build()
            .unwrap(),
    ) {
        TransitionOutcome::Success(session) => session,
        other => panic!("expected authorized Worth-native session, got {other:?}"),
    }
}

pub fn open_mutation_session(
    session: &WorthServerWorthNativeSession,
    basis_digest: &str,
    operation_name: &str,
) -> WorthServerProductSession {
    session
        .product_sessions()
        .open_mutation(
            worth_server::WorthServerProductSessionCreationRequest::for_operation(operation_name)
                .with_basis_digest(basis_digest)
                .with_expiry_seconds(300),
        )
        .expect("mutation session should open")
}

pub fn prepared_read_request(
    server: &WorthServer,
    operation_name: &str,
    basis_digest: Option<&str>,
) -> WorthServerCompatibilityPreparedRequest {
    crate::shared_support_product_adapter_phase_nine_fixture::prepared_read_request(server, operation_name, basis_digest)
}

pub fn prepared_mutation_request(
    server: &WorthServer,
    operation_name: &str,
    basis_digest: Option<&str>,
) -> WorthServerCompatibilityPreparedRequest {
    crate::shared_support_product_adapter_phase_nine_fixture::prepared_mutation_request(
        server,
        operation_name,
        basis_digest,
    )
}

pub fn render_payload() -> WorthServerProductOperationPayload {
    worth_server::WorthServerProductOperationPayload::json("product-editor.render.v1", json!({}))
}

pub fn select_payload(node: &str) -> WorthServerProductOperationPayload {
    worth_server::WorthServerProductOperationPayload::json("product-editor.select.v1", json!({ "node": node }))
}

pub fn actions_payload() -> WorthServerProductOperationPayload {
    worth_server::WorthServerProductOperationPayload::json("product-editor.actions.v1", json!({}))
}

pub fn apply_payload(title: &str) -> WorthServerProductOperationPayload {
    worth_server::WorthServerProductOperationPayload::json("product-editor.apply.v1", json!({ "title": title }))
}

pub fn finalize_payload(confirm: bool) -> WorthServerProductOperationPayload {
    worth_server::WorthServerProductOperationPayload::json(
        "product-editor.finalize.v1",
        json!({ "confirm": confirm }),
    )
}

pub fn direct_read(
    session: &WorthServerWorthNativeSession,
    operation_name: &str,
    payload: WorthServerProductOperationPayload,
    basis_digest: &str,
) -> worth_server::WorthServerCompletedProductOperation {
    session
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new(operation_name, payload)
                .with_basis_digest(basis_digest),
        )
        .expect("direct editor-like read should complete")
}

pub fn direct_mutation(
    session: &WorthServerWorthNativeSession,
    operation_name: &str,
    payload: WorthServerProductOperationPayload,
    basis_digest: &str,
    product_session: &WorthServerProductSession,
) -> Result<
    worth_server::WorthServerCompletedProductOperation,
    worth_server::WorthServerProductOperationSurfaceDenial,
> {
    session.product_operations().execute(
        worth_server::WorthServerProductOperationInput::new(operation_name, payload)
            .with_basis_digest(basis_digest)
            .with_product_session_identity(product_session.identity().as_str()),
    )
}

#[derive(Debug)]
pub struct EditorLikeState {
    pub title: String,
    pub revision: usize,
}

impl Default for EditorLikeState {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            revision: 0,
        }
    }
}

impl EditorLikeState {
    pub fn basis_digest(&self) -> String {
        format!("basis:r{}", self.revision)
    }
}

pub fn declared(
    declaration: WorthServerProductOperationDeclaration,
) -> WorthServerProductOperationDeclaration {
    declaration.with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough())
}

pub fn product_success(
    plan: &worth_server::WorthServerLoweredProductOperationPlan,
    result_key: &str,
    body: Value,
) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
    crate::shared_support_product_result_schema_bound_json::publish_schema_bound_json(
        result_key,
        plan.declaration().result_contract(),
        plan.declaration().result_contract().schema().identity(),
        body,
    )
}