use super::*;

pub fn editor_registration(
    calls: Option<Arc<AtomicUsize>>,
    validator: Option<Arc<dyn WorthServerProductPayloadSchemaValidator>>,
) -> WorthServerProductApplicationAdapterRegistration {
    let adapter = std::sync::Arc::new(EditorAdapter {
        calls: calls.unwrap_or_else(|| std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0))),
    });
    let render = declared(
        worth_server::WorthServerProductOperationDeclaration::product_read(
            "product_editor.render",
            "product-editor.render.v1",
            result_contract("product-editor.render.result.v1"),
            worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
            worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                "render-supported",
            ),
        ),
    );
    let apply = declared(
        worth_server::WorthServerProductOperationDeclaration::product_mutation(
            "product_editor.apply",
            "product-editor.apply.v1",
            result_contract("product-editor.apply.result.v1"),
            worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
            worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                "apply-supported",
            ),
            "draft",
        ),
    );
    let apply = match validator {
        Some(validator) => apply.with_payload_validator(validator),
        None => apply,
    };
    worth_server::WorthServerProductApplicationAdapterRegistration::new("editor-adapter", adapter)
        .with_operation(render)
        .with_operation(declared(
            worth_server::WorthServerProductOperationDeclaration::product_read(
                "product_editor.select",
                "product-editor.select.v1",
                result_contract("product-editor.select.result.v1"),
                worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
                worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                    "select-supported",
                ),
            ),
        ))
        .with_operation(declared(
            worth_server::WorthServerProductOperationDeclaration::product_read(
                "product_editor.available_actions",
                "product-editor.actions.v1",
                result_contract("product-editor.actions.result.v1"),
                worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
                worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                    "actions-supported",
                ),
            ),
        ))
        .with_operation(apply)
        .with_operation(declared(
            worth_server::WorthServerProductOperationDeclaration::product_mutation(
                "product_editor.finalize",
                "product-editor.finalize.v1",
                result_contract("product-editor.finalize.result.v1"),
                worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
                worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                    "finalize-supported",
                ),
                "draft",
            ),
        ))
}

pub fn query_derived_editor_registration(
    calls: Option<Arc<AtomicUsize>>,
) -> WorthServerProductApplicationAdapterRegistration {
    let adapter = std::sync::Arc::new(EditorAdapter {
        calls: calls.unwrap_or_else(|| std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0))),
    });
    worth_server::WorthServerProductApplicationAdapterRegistration::new(
        "query-derived-editor",
        adapter,
    )
    .with_operation(declared(
        worth_server::WorthServerProductOperationDeclaration::product_read(
            "product_editor.render",
            "product-editor.render.v1",
            result_contract("product-editor.render.result.v1"),
            worth_server::WorthServerProductOperationBasisKind::QueryDerived,
            worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                "render-query-derived",
            ),
        ),
    ))
    .with_operation(declared(
        worth_server::WorthServerProductOperationDeclaration::product_mutation(
            "product_editor.apply",
            "product-editor.apply.v1",
            result_contract("product-editor.apply.result.v1"),
            worth_server::WorthServerProductOperationBasisKind::QueryDerived,
            worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                "apply-query-derived",
            ),
            "draft",
        ),
    ))
}
