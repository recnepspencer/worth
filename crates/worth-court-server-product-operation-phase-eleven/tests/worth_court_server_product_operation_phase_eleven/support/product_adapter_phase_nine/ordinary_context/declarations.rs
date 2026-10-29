#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

pub(crate) use {serde_json::json};
pub(crate) use {worth_server::WorthServerOperationReadinessDenialCode, worth_server::WorthServerProductApplicationAdapterRegistration, worth_server::WorthServerProductOperationBasisKind, worth_server::WorthServerProductOperationDeclaration, worth_server::WorthServerProductOperationDenialCode, worth_server::WorthServerProductOperationErrorMaps, worth_server::WorthServerProductOperationInput, worth_server::WorthServerProductOperationOutcome, worth_server::WorthServerProductOperationPayload, worth_server::WorthServerProductOperationSupportSnapshot, worth_server::WorthServerProductOperationSurfaceDenialCode};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_product_adapter_phase_nine_fixture as fixture};

pub(crate) use {worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::build_server, worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::completed, worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::direct_session, worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::editor_registration, worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::open_mutation_product_session, worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::prepared_mutation_request, worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::prepared_read_request, worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::result_contract, worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::EditorAdapter, worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::RequireTitleValidator};

pub(crate) fn editor_execution_cases() -> [(&'static str, WorthServerProductOperationPayload); 5] {
    [
        (
            "product_editor.render",
            worth_server::WorthServerProductOperationPayload::json(
                "product-editor.render.v1",
                json!({ "document": "doc-1" }),
            ),
        ),
        (
            "product_editor.select",
            worth_server::WorthServerProductOperationPayload::json(
                "product-editor.select.v1",
                json!({ "node": "node-1" }),
            ),
        ),
        (
            "product_editor.available_actions",
            worth_server::WorthServerProductOperationPayload::json(
                "product-editor.actions.v1",
                json!({ "node": "node-1" }),
            ),
        ),
        (
            "product_editor.apply",
            worth_server::WorthServerProductOperationPayload::json(
                "product-editor.apply.v1",
                json!({ "title": "Rename" }),
            ),
        ),
        (
            "product_editor.finalize",
            worth_server::WorthServerProductOperationPayload::json(
                "product-editor.finalize.v1",
                json!({ "title": "Finalize" }),
            ),
        ),
    ]
}

pub(crate) fn expected_scheduler_lane(operation_name: &str, product_session_identity: &str) -> String {
    if operation_name.contains("apply") || operation_name.contains("finalize") {
        format!("product-draft:{product_session_identity}:draft")
    } else {
        "shared-read".to_string()
    }
}

pub(crate) fn unsupported_render_registration() -> WorthServerProductApplicationAdapterRegistration {
    worth_server::WorthServerProductApplicationAdapterRegistration::new(
        "unsupported-render",
        std::sync::Arc::new(worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::EditorAdapter::default()),
    )
    .with_operation(
        worth_server::WorthServerProductOperationDeclaration::product_read(
            "product_editor.render",
            "product-editor.render.v1",
            result_contract("product-editor.render.result.v1"),
            worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
            worth_server::WorthServerProductOperationSupportSnapshot::unsupported("render-unsupported"),
        )
        .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough()),
    )
}