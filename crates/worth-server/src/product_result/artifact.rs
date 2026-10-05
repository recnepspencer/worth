use serde_json::Value;

use super::{WorthServerProductResultBody, WorthServerProductResultContract};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerProductResultArtifactErrorCode {
    SerializationFailed,
    SchemaContractMismatch,
    InlineBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductResultArtifactError {
    code: WorthServerProductResultArtifactErrorCode,
    detail: String,
}

impl WorthServerProductResultArtifactError {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        crate::product_adapter::execution_pipeline::read_batch_accounting::string(&self.detail)
    }

    fn serialization_failed(error: serde_json::Error) -> Self {
        Self {
            code: WorthServerProductResultArtifactErrorCode::SerializationFailed,
            detail: format!("product result JSON serialization failed: {error}"),
        }
    }

    fn inline_budget_exceeded(max_inline_bytes: usize) -> Self {
        Self {
            code: WorthServerProductResultArtifactErrorCode::InlineBudgetExceeded,
            detail: format!(
                "product result body exceeded the declared {max_inline_bytes}-byte inline budget"
            ),
        }
    }

    fn schema_contract_mismatch(
        actual_identity: &str,
        actual_version: u32,
        contract: &WorthServerProductResultContract,
    ) -> Self {
        Self {
            code: WorthServerProductResultArtifactErrorCode::SchemaContractMismatch,
            detail: format!(
                "typed product result schema `{actual_identity}` v{actual_version} does not match declared schema `{}` v{}",
                contract.schema().identity(),
                contract.schema().version(),
            ),
        }
    }

    pub fn code(&self) -> WorthServerProductResultArtifactErrorCode {
        self.code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductResultArtifact {
    contract: WorthServerProductResultContract,
    body: WorthServerProductResultBody,
    body_digest: String,
    artifact_digest: String,
}

impl WorthServerProductResultArtifact {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::string;
        self.contract
            .owned_allocation_capacity_bytes()
            .saturating_add(self.body.owned_allocation_capacity_bytes())
            .saturating_add(string(&self.body_digest))
            .saturating_add(string(&self.artifact_digest))
    }

    pub fn publish_json<T>(
        contract: &WorthServerProductResultContract,
        value: &T,
    ) -> Result<Self, WorthServerProductResultArtifactError>
    where
        T: super::WorthServerProductResultValue,
    {
        if value.result_schema_identity() != contract.schema().identity()
            || value.result_schema_version() != contract.schema().version()
        {
            return Err(
                WorthServerProductResultArtifactError::schema_contract_mismatch(
                    value.result_schema_identity(),
                    value.result_schema_version(),
                    contract,
                ),
            );
        }
        let serialized = match super::bounded_serialization::serialize_with_inline_budget(
            value,
            contract.max_inline_bytes(),
        ) {
            Ok(serialized) => serialized,
            Err(super::bounded_serialization::WorthServerBoundedJsonSerializationError::Serialization(error)) => {
                return Err(WorthServerProductResultArtifactError::serialization_failed(error));
            }
            Err(super::bounded_serialization::WorthServerBoundedJsonSerializationError::InlineBudgetExceeded) => {
                return Err(WorthServerProductResultArtifactError::inline_budget_exceeded(
                    contract.max_inline_bytes(),
                ));
            }
        };
        let value = serde_json::from_slice(&serialized)
            .map_err(WorthServerProductResultArtifactError::serialization_failed)?;
        Self::canonical_json(contract, value)
    }

    pub(crate) fn canonical_json(
        contract: &WorthServerProductResultContract,
        value: Value,
    ) -> Result<Self, WorthServerProductResultArtifactError> {
        let body = WorthServerProductResultBody::canonical_json(value)
            .map_err(WorthServerProductResultArtifactError::serialization_failed)?;
        if body.byte_len() > contract.max_inline_bytes() {
            return Err(
                WorthServerProductResultArtifactError::inline_budget_exceeded(
                    contract.max_inline_bytes(),
                ),
            );
        }
        let body_digest = super::sha256_hex(body.canonical_bytes());
        let body_byte_len = body.byte_len().to_string();
        let artifact_digest = crate::canonical_digest::WorthServerCanonicalDigestBuilder::new(
            "worth-server-product-result-artifact-v2",
        )
        .field("contract", contract.canonical_digest())
        .field("body", &body_digest)
        .field("bytes", &body_byte_len)
        .finish();
        Ok(Self {
            contract: contract.clone(),
            body,
            body_digest,
            artifact_digest,
        })
    }

    pub fn contract(&self) -> &WorthServerProductResultContract {
        &self.contract
    }

    pub fn body(&self) -> &WorthServerProductResultBody {
        &self.body
    }

    pub fn body_digest(&self) -> &str {
        &self.body_digest
    }

    pub fn artifact_digest(&self) -> &str {
        &self.artifact_digest
    }
}
