use serde_json::Value;
use worth_server::{
    WorthServerCompletedProductOperation, WorthServerProductIdempotencyKey,
    WorthServerProductOperationInput, WorthServerProductOperationOutcome,
    WorthServerProductOperationPayload, WorthServerProductOperationSurfaceDenial,
    WorthServerWorthNativeSession,
};

pub fn execute(
    session: &WorthServerWorthNativeSession,
    operation_name: &str,
    payload_schema: &str,
    body: Value,
    basis: &str,
    idempotency_key: &str,
) -> Result<WorthServerCompletedProductOperation, WorthServerProductOperationSurfaceDenial> {
    session.product_operations().execute(
        WorthServerProductOperationInput::new(
            operation_name,
            WorthServerProductOperationPayload::json(payload_schema, body),
        )
        .with_basis_digest(basis)
        .with_idempotency_key(
            WorthServerProductIdempotencyKey::new(idempotency_key)
                .expect("test idempotency key should validate"),
        ),
    )
}

pub fn result_body(completed: &WorthServerCompletedProductOperation) -> &Value {
    match completed.outcome() {
        WorthServerProductOperationOutcome::Success(success) => {
            success.result_artifact().body().value()
        }
        other => panic!("expected success result, got {other:?}"),
    }
}
