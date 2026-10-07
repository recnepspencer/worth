use crate::WorthServerOperationFamily;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerOperationIdentity {
    operation_family: WorthServerOperationFamily,
    tenant_id: String,
    workspace_id: String,
    target_identity: String,
    operation_name: String,
    basis_digest: Option<String>,
    idempotency_key: Option<String>,
    product_session_identity: Option<String>,
    payload_identity: Option<String>,
    canonical_digest: String,
}

pub(crate) struct WorthServerOperationIdentityParts {
    pub(crate) operation_family: WorthServerOperationFamily,
    pub(crate) tenant_id: String,
    pub(crate) workspace_id: String,
    pub(crate) target_identity: String,
    pub(crate) operation_name: String,
    pub(crate) basis_digest: Option<String>,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) product_session_identity: Option<String>,
    pub(crate) payload_identity: Option<String>,
}

impl WorthServerOperationIdentity {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::{
            option_string, string,
        };
        string(&self.tenant_id)
            .saturating_add(string(&self.workspace_id))
            .saturating_add(string(&self.target_identity))
            .saturating_add(string(&self.operation_name))
            .saturating_add(option_string(&self.basis_digest))
            .saturating_add(option_string(&self.idempotency_key))
            .saturating_add(option_string(&self.product_session_identity))
            .saturating_add(option_string(&self.payload_identity))
            .saturating_add(string(&self.canonical_digest))
    }

    pub(crate) fn new(parts: WorthServerOperationIdentityParts) -> Self {
        let WorthServerOperationIdentityParts {
            operation_family,
            tenant_id,
            workspace_id,
            target_identity,
            operation_name,
            basis_digest,
            idempotency_key,
            product_session_identity,
            payload_identity,
        } = parts;
        let canonical_digest = format!(
            "worth-server-operation-identity-v1|family={}|tenant={tenant_id}|workspace={workspace_id}|target={target_identity}|operation={operation_name}|basis={}|idempotency={}|product_session={}|payload={}",
            operation_family.as_str(),
            basis_digest.as_deref().unwrap_or("none"),
            idempotency_key.as_deref().unwrap_or("none"),
            product_session_identity.as_deref().unwrap_or("none"),
            payload_identity.as_deref().unwrap_or("none"),
        );
        Self {
            operation_family,
            tenant_id,
            workspace_id,
            target_identity,
            operation_name,
            basis_digest,
            idempotency_key,
            product_session_identity,
            payload_identity,
            canonical_digest,
        }
    }

    pub fn operation_family(&self) -> WorthServerOperationFamily {
        self.operation_family
    }

    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }

    pub fn basis_digest(&self) -> Option<&str> {
        self.basis_digest.as_deref()
    }

    pub fn idempotency_key(&self) -> Option<&str> {
        self.idempotency_key.as_deref()
    }

    pub fn product_session_identity(&self) -> Option<&str> {
        self.product_session_identity.as_deref()
    }

    pub fn payload_identity(&self) -> Option<&str> {
        self.payload_identity.as_deref()
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }

    pub(crate) fn with_basis_digest(&self, basis_digest: Option<&str>) -> Self {
        Self::new(WorthServerOperationIdentityParts {
            operation_family: self.operation_family,
            tenant_id: self.tenant_id.clone(),
            workspace_id: self.workspace_id.clone(),
            target_identity: self.target_identity.clone(),
            operation_name: self.operation_name.clone(),
            basis_digest: basis_digest.map(str::to_string),
            idempotency_key: self.idempotency_key.clone(),
            product_session_identity: self.product_session_identity.clone(),
            payload_identity: self.payload_identity.clone(),
        })
    }
}
