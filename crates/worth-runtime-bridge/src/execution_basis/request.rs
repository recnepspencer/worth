use std::sync::Arc;

use sha2::{Digest, Sha256};

/// Non-authoritative intent presented to the runtime bridge when Query needs
/// one managed execution attempt.
///
/// The strings are correlation inputs only. Operational authority begins with
/// the Bridge-minted execution basis returned by admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeManagedExecutionIntent {
    identity: BridgeManagedExecutionIntentIdentity,
    operation_binding_identity: Arc<str>,
    resource_attempt_identity: Arc<str>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BridgeManagedExecutionIntentIdentity(Arc<str>);

impl BridgeManagedExecutionIntent {
    pub fn new(
        operation_binding_identity: impl Into<Arc<str>>,
        resource_attempt_identity: impl Into<Arc<str>>,
    ) -> Self {
        let operation_binding_identity = operation_binding_identity.into();
        let resource_attempt_identity = resource_attempt_identity.into();
        let canonical_basis = format!(
            "bridge-managed-execution-intent-v1|operation={operation_binding_identity}|resource-attempt={resource_attempt_identity}"
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            identity: BridgeManagedExecutionIntentIdentity(Arc::from(format!(
                "bridge-managed-execution-intent:sha256:{digest:x}"
            ))),
            operation_binding_identity,
            resource_attempt_identity,
        }
    }

    pub fn identity(&self) -> &BridgeManagedExecutionIntentIdentity {
        &self.identity
    }

    pub fn operation_binding_identity(&self) -> &str {
        self.operation_binding_identity.as_ref()
    }

    pub fn resource_attempt_identity(&self) -> &str {
        self.resource_attempt_identity.as_ref()
    }
}

impl BridgeManagedExecutionIntentIdentity {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}
#[cfg(test)]
#[path = "request_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
