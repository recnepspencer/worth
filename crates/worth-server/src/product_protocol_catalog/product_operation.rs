use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WorthServerProductOperationProtocol {
    operation_name: String,
    operation_family: String,
    method: String,
    route: String,
    request_schema_identity: String,
    result_schema_identity: String,
    result_schema_version: u32,
    result_contract_digest: String,
    result_encoding: String,
    result_canonicalization: String,
    result_max_inline_bytes: usize,
    basis_kind: String,
    requires_basis: bool,
    requires_product_session: bool,
    requires_idempotency_key: bool,
    #[serde(skip_serializing_if = "is_false")]
    requires_server_instance: bool,
}

pub(crate) struct WorthServerProductOperationProtocolParts {
    pub operation_name: String,
    pub operation_family: String,
    pub method: String,
    pub route: String,
    pub request_schema_identity: String,
    pub result_schema_identity: String,
    pub result_schema_version: u32,
    pub result_contract_digest: String,
    pub result_encoding: String,
    pub result_canonicalization: String,
    pub result_max_inline_bytes: usize,
    pub basis_kind: String,
    pub requires_product_session: bool,
    pub requires_idempotency_key: bool,
    pub requires_server_instance: bool,
}

impl WorthServerProductOperationProtocol {
    pub(crate) fn from_parts(parts: WorthServerProductOperationProtocolParts) -> Self {
        Self {
            operation_name: parts.operation_name,
            operation_family: parts.operation_family,
            method: parts.method,
            route: parts.route,
            request_schema_identity: parts.request_schema_identity,
            result_schema_identity: parts.result_schema_identity,
            result_schema_version: parts.result_schema_version,
            result_contract_digest: parts.result_contract_digest,
            result_encoding: parts.result_encoding,
            result_canonicalization: parts.result_canonicalization,
            result_max_inline_bytes: parts.result_max_inline_bytes,
            basis_kind: parts.basis_kind,
            requires_basis: true,
            requires_product_session: parts.requires_product_session,
            requires_idempotency_key: parts.requires_idempotency_key,
            requires_server_instance: parts.requires_server_instance,
        }
    }

    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }

    pub fn operation_family(&self) -> &str {
        &self.operation_family
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn route(&self) -> &str {
        &self.route
    }

    pub fn request_schema_identity(&self) -> &str {
        &self.request_schema_identity
    }

    pub fn result_schema_identity(&self) -> &str {
        &self.result_schema_identity
    }

    pub fn result_schema_version(&self) -> u32 {
        self.result_schema_version
    }

    pub fn result_contract_digest(&self) -> &str {
        &self.result_contract_digest
    }

    pub fn result_encoding(&self) -> &str {
        &self.result_encoding
    }

    pub fn result_canonicalization(&self) -> &str {
        &self.result_canonicalization
    }

    pub fn result_max_inline_bytes(&self) -> usize {
        self.result_max_inline_bytes
    }

    pub fn basis_kind(&self) -> &str {
        &self.basis_kind
    }

    pub fn requires_basis(&self) -> bool {
        self.requires_basis
    }

    pub fn requires_product_session(&self) -> bool {
        self.requires_product_session
    }

    pub fn requires_idempotency_key(&self) -> bool {
        self.requires_idempotency_key
    }

    pub fn requires_server_instance(&self) -> bool {
        self.requires_server_instance
    }
}

fn is_false(value: &bool) -> bool {
    !value
}
