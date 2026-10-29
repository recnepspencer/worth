pub use {serde::Serialize};
pub use {serde_json::Value};
pub use {worth_server::WorthServerAdmittedDurableProductMutation, worth_server::WorthServerProductResultValue};

#[derive(Serialize)]
pub struct DurableMutationProductResult<'a> {
    pub operation: &'a str,
    pub resource: &'a Value,
    pub durable: bool,
    pub next_basis: &'a str,
    pub idempotency_retention: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oversized_padding: Option<String>,
    #[serde(skip)]
    pub schema_identity: &'a str,
}

impl<'a> DurableMutationProductResult<'a> {
    pub fn from_attempt(
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
                .and_then(serde_json::Value::as_bool)
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