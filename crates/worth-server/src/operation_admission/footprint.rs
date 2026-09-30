use super::WorthServerOperationScope;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerOperationAuthorityKind {
    SharedReadOnly,
    DeterministicSubmission,
    ProductDraftMutation,
    DurableProductMutation,
    ProductSessionCoordination,
    BinaryStreaming,
    DiagnosticsOnly,
    LeaseCoordination,
}

impl WorthServerOperationAuthorityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SharedReadOnly => "shared-read-only",
            Self::DeterministicSubmission => "deterministic-submission",
            Self::ProductDraftMutation => "product-draft-mutation",
            Self::DurableProductMutation => "durable-product-mutation",
            Self::ProductSessionCoordination => "product-session-coordination",
            Self::BinaryStreaming => "binary-streaming",
            Self::DiagnosticsOnly => "diagnostics-only",
            Self::LeaseCoordination => "lease-coordination",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerOperationAuthorityFootprint {
    authority_kind: WorthServerOperationAuthorityKind,
    scope: WorthServerOperationScope,
    descriptor_digest: String,
    canonical_digest: String,
}

impl WorthServerOperationAuthorityFootprint {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::string;
        self.scope
            .owned_allocation_capacity_bytes()
            .saturating_add(string(&self.descriptor_digest))
            .saturating_add(string(&self.canonical_digest))
    }

    pub(crate) fn new(
        authority_kind: WorthServerOperationAuthorityKind,
        scope: WorthServerOperationScope,
        descriptor_digest: impl Into<String>,
    ) -> Self {
        let descriptor_digest = descriptor_digest.into();
        let scope_digest = scope.canonical_digest();
        let canonical_digest = format!(
            "worth-server-operation-authority-footprint-v1|kind={}|scope={scope_digest}|descriptor={descriptor_digest}",
            authority_kind.as_str(),
        );
        Self {
            authority_kind,
            scope,
            descriptor_digest,
            canonical_digest,
        }
    }

    pub fn authority_kind(&self) -> WorthServerOperationAuthorityKind {
        self.authority_kind
    }

    pub fn scope(&self) -> &WorthServerOperationScope {
        &self.scope
    }

    pub fn descriptor_digest(&self) -> &str {
        &self.descriptor_digest
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}
