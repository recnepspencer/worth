use super::*;

pub fn prepared_read_request(
    server: &WorthServer,
    operation_name: &str,
    basis_digest: Option<&str>,
) -> WorthServerCompatibilityPreparedRequest {
    crate::shared_support_product_adapter_phase_nine_fixture::prepared_read_request(
        server,
        operation_name,
        basis_digest,
    )
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
    worth_server::WorthServerProductOperationPayload::json(
        "product-editor.select.v1",
        json!({ "node": node }),
    )
}

pub fn actions_payload() -> WorthServerProductOperationPayload {
    worth_server::WorthServerProductOperationPayload::json("product-editor.actions.v1", json!({}))
}

pub fn apply_payload(title: &str) -> WorthServerProductOperationPayload {
    worth_server::WorthServerProductOperationPayload::json(
        "product-editor.apply.v1",
        json!({ "title": title }),
    )
}

pub fn finalize_payload(confirm: bool) -> WorthServerProductOperationPayload {
    worth_server::WorthServerProductOperationPayload::json(
        "product-editor.finalize.v1",
        json!({ "confirm": confirm }),
    )
}
