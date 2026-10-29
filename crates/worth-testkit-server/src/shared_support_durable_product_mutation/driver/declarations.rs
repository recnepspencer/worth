pub use {serde_json::Value};
pub use {worth_server::WorthServerCompletedProductOperation, worth_server::WorthServerProductIdempotencyKey, worth_server::WorthServerProductOperationInput, worth_server::WorthServerProductOperationOutcome, worth_server::WorthServerProductOperationPayload, worth_server::WorthServerProductOperationSurfaceDenial, worth_server::WorthServerWorthNativeSession};

pub fn execute(
    session: &WorthServerWorthNativeSession,
    operation_name: &str,
    payload_schema: &str,
    body: Value,
    basis: &str,
    idempotency_key: &str,
) -> Result<WorthServerCompletedProductOperation, WorthServerProductOperationSurfaceDenial> {
    session.product_operations().execute(
        worth_server::WorthServerProductOperationInput::new(
            operation_name,
            worth_server::WorthServerProductOperationPayload::json(payload_schema, body),
        )
        .with_basis_digest(basis)
        .with_idempotency_key(
            worth_server::WorthServerProductIdempotencyKey::new(idempotency_key)
                .expect("test idempotency key should validate"),
        ),
    )
}

pub fn result_body(completed: &WorthServerCompletedProductOperation) -> &Value {
    match completed.outcome() {
        worth_server::WorthServerProductOperationOutcome::Success(success) => {
            success.result_artifact().body().value()
        }
        other => panic!("expected success result, got {other:?}"),
    }
}