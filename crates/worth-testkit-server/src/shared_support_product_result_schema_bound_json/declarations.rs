pub use serde_json::Value;
pub use {serde::Serialize, serde::Serializer};
pub use {
    worth_server::WorthServerProductAdapterExecutionError,
    worth_server::WorthServerProductOperationSuccess,
    worth_server::WorthServerProductResultContract, worth_server::WorthServerProductResultValue,
};

pub struct SchemaBoundJsonResult {
    pub schema_identity: String,
    pub schema_version: u32,
    pub body: Value,
}

impl SchemaBoundJsonResult {
    pub fn v1(schema_identity: impl Into<String>, body: Value) -> Self {
        Self {
            schema_identity: schema_identity.into(),
            schema_version: 1,
            body,
        }
    }
}

impl Serialize for SchemaBoundJsonResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.body.serialize(serializer)
    }
}

impl WorthServerProductResultValue for SchemaBoundJsonResult {
    fn result_schema_identity(&self) -> &str {
        &self.schema_identity
    }

    fn result_schema_version(&self) -> u32 {
        self.schema_version
    }
}

#[allow(dead_code)]
pub fn publish_schema_bound_json(
    result_key: impl Into<String>,
    contract: &WorthServerProductResultContract,
    schema_identity: impl Into<String>,
    body: Value,
) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
    worth_server::WorthServerProductOperationSuccess::publish_json(
        result_key,
        contract,
        &crate::shared_support_product_result_schema_bound_json::SchemaBoundJsonResult::v1(
            schema_identity,
            body,
        ),
    )
    .map_err(worth_server::WorthServerProductAdapterExecutionError::invalid_result_artifact)
}
