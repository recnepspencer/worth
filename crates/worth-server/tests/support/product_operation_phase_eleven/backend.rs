use std::sync::{Arc, Mutex};

use serde_json::json;
use worth_server::{
    WorthServerProductAdapterExecutionError, WorthServerProductApplicationAdapter,
    WorthServerProductApplicationAdapterRegistration, WorthServerProductOperationBasisKind,
    WorthServerProductOperationDeclaration, WorthServerProductOperationErrorMaps,
    WorthServerProductOperationPayload, WorthServerProductOperationSuccess,
    WorthServerProductOperationSupportSnapshot,
};

use crate::fixture::schema_bound_json;

#[derive(Clone, Debug)]
pub struct StatefulProductEditorBackend {
    state: Arc<Mutex<ProductEditorState>>,
}

impl StatefulProductEditorBackend {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ProductEditorState::default())),
        }
    }

    pub fn basis_digest(&self) -> String {
        self.state
            .lock()
            .expect("stateful editor backend should not poison")
            .basis_digest()
    }

    pub fn title(&self) -> String {
        self.state
            .lock()
            .expect("stateful editor backend should not poison")
            .title
            .clone()
    }

    pub fn revision(&self) -> usize {
        self.state
            .lock()
            .expect("stateful editor backend should not poison")
            .revision
    }
}

pub fn stateful_editor_registration(
    backend: StatefulProductEditorBackend,
) -> WorthServerProductApplicationAdapterRegistration {
    WorthServerProductApplicationAdapterRegistration::new("stateful-editor", Arc::new(backend))
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

pub fn controlled_apply_payload(title: &str, fail: bool) -> WorthServerProductOperationPayload {
    WorthServerProductOperationPayload::json(
        "product-editor.apply.v1",
        json!({ "title": title, "fail": fail }),
    )
}

impl WorthServerProductApplicationAdapter for StatefulProductEditorBackend {
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
        let mut state = self
            .state
            .lock()
            .expect("stateful editor backend should not poison");
        match operation.plan().declaration().operation_name() {
            "product_editor.render_preview" => product_success(
                operation,
                "product_editor.render_preview",
                json!({ "basis": state.basis_digest(), "title": state.title }),
            ),
            "product_editor.apply" => {
                let body = operation.plan().payload().body();
                if body.get("fail").and_then(|value| value.as_bool()) == Some(true) {
                    return Err(WorthServerProductAdapterExecutionError::failed(
                        "adapter_failed",
                        format!("controlled failure at {}", state.basis_digest()),
                    ));
                }
                let title = body
                    .get("title")
                    .and_then(|value| value.as_str())
                    .unwrap_or("Untitled");
                state.title = title.to_string();
                state.revision += 1;
                product_success(
                    operation,
                    "product_editor.apply",
                    json!({ "basis": state.basis_digest(), "title": state.title }),
                )
            }
            operation_name => Err(WorthServerProductAdapterExecutionError::failed(
                "unsupported_operation",
                format!("stateful editor backend does not support `{operation_name}`"),
            )),
        }
    }
}

#[derive(Debug)]
struct ProductEditorState {
    title: String,
    revision: usize,
}

impl Default for ProductEditorState {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            revision: 0,
        }
    }
}

impl ProductEditorState {
    fn basis_digest(&self) -> String {
        format!("basis:r{}", self.revision)
    }
}

fn declared(
    declaration: WorthServerProductOperationDeclaration,
) -> WorthServerProductOperationDeclaration {
    declaration.with_error_map(WorthServerProductOperationErrorMaps::passthrough())
}

impl Default for StatefulProductEditorBackend {
    fn default() -> Self {
        Self::new()
    }
}

fn result_contract(schema_identity: &str) -> worth_server::WorthServerProductResultContract {
    worth_server::WorthServerProductResultContract::canonical_json(schema_identity, 1, 16 * 1024)
        .expect("stateful editor result contract should validate")
}

fn product_success(
    operation: &worth_server::WorthServerScheduledProductOperation,
    result_key: &str,
    body: serde_json::Value,
) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
    schema_bound_json::publish_schema_bound_json(
        result_key,
        operation.plan().declaration().result_contract(),
        operation
            .plan()
            .declaration()
            .result_contract()
            .schema()
            .identity(),
        body,
    )
}
