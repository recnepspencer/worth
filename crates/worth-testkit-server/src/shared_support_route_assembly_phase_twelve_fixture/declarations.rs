pub use serde_json::json;
pub use {
    worth_server::WorthServer, worth_server::WorthServerProductOperationInput,
    worth_server::WorthServerProductOperationPayload,
};

#[allow(unused_imports)]
pub use crate::shared_support_product_adapter_phase_nine_fixture as product_fixture;

pub fn build_server() -> WorthServer {
    let structured_read =
        worth_server::WorthServerProductOperationDeclaration::product_structured_read(
            "product_editor.inspect",
            "product-editor.inspect.v1",
            crate::shared_support_product_adapter_phase_nine_fixture::result_contract(
                "product-editor.inspect.result.v1",
            ),
            worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
            worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                "inspect-supported",
            ),
        )
        .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough());
    crate::shared_support_product_adapter_phase_nine_fixture::build_server(vec![
        crate::shared_support_product_adapter_phase_nine_fixture::editor_registration(None, None)
            .with_operation(structured_read),
    ])
}

pub fn base_config() -> worth_server::WorthServerConfig {
    crate::shared_support_product_adapter_phase_nine_fixture::base_config()
}

pub fn editor_registration() -> worth_server::WorthServerProductApplicationAdapterRegistration {
    crate::shared_support_product_adapter_phase_nine_fixture::editor_registration(None, None)
}

pub fn direct_read(server: &WorthServer) -> worth_server::WorthServerCompletedProductOperation {
    crate::shared_support_product_adapter_phase_nine_fixture::completed(
        crate::shared_support_product_adapter_phase_nine_fixture::direct_session(server)
            .product_operations()
            .execute(
                worth_server::WorthServerProductOperationInput::new(
                    "product_editor.render",
                    worth_server::WorthServerProductOperationPayload::json(
                        "product-editor.render.v1",
                        json!({}),
                    ),
                )
                .with_basis_digest("basis:7"),
            ),
    )
}

pub fn direct_mutation(
    server: &WorthServer,
) -> (
    worth_server::WorthServerProductSession,
    worth_server::WorthServerCompletedProductOperation,
) {
    let session = crate::shared_support_product_adapter_phase_nine_fixture::direct_session(server);
    let product_session =
        crate::shared_support_product_adapter_phase_nine_fixture::open_mutation_product_session(
            &session,
            "product_editor.apply",
            "basis:7",
        );
    let completed = crate::shared_support_product_adapter_phase_nine_fixture::completed(
        session.product_operations().execute(
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.apply",
                worth_server::WorthServerProductOperationPayload::json(
                    "product-editor.apply.v1",
                    json!({ "title": "Rename" }),
                ),
            )
            .with_basis_digest("basis:7")
            .with_product_session_identity(product_session.identity().as_str()),
        ),
    );
    (product_session, completed)
}
