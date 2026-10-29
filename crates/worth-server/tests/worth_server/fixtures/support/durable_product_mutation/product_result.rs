use serde::Serialize;
use serde_json::Value;
use worth_server::{WorthServerAdmittedDurableProductMutation, WorthServerProductResultValue};

#[derive(Serialize)]
pub(super) struct DurableMutationProductResult<'a> {
    operation: &'a str,
    resource: &'a Value,
    durable: bool,
    next_basis: &'a str,
    idempotency_retention: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    oversized_padding: Option<String>,
    #[serde(skip)]
    schema_identity: &'a str,
}

impl<'a> DurableMutationProductResult<'a> {
    pub(super) fn from_attempt(
        attempt: &'a WorthServerAdmittedDurableProductMutation,
        next_basis: &'a str,
    ) -> Self {
        Self {
            operation: attempt.operation_name(),
            resource: attempt.payload().body(),
            durable: true,
            next_basis,
            idempotency_retention: attempt
                .durable_contract()
                .idempotency_retention()
                .canonical_label(),
            oversized_padding: attempt
                .payload()
                .body()
                .get("oversized_result")
                .and_then(Value::as_bool)
                .filter(|oversized| *oversized)
                .map(|_| "x".repeat(20 * 1024)),
            schema_identity: attempt.result_contract().schema().identity(),
        }
    }
}

impl WorthServerProductResultValue for DurableMutationProductResult<'_> {
    fn result_schema_identity(&self) -> &str {
        self.schema_identity
    }

    fn result_schema_version(&self) -> u32 {
        1
    }
}
