use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::error::BridgeDeliveryErrorKind;
use crate::identity::{BridgeIdentity, SourceFailureRecordIdentityTag};
use crate::snapshot::BridgeTruthViewSelector;

use super::{BridgeSourceCapabilitySet, SourceDeclarationIdentity};

pub type SourceFailureRecordIdentity = BridgeIdentity<SourceFailureRecordIdentityTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFailureClass {
    UnsupportedSourceCapability,
    SourceContractMismatch,
    SourceContractVersionMismatch,
    TruthViewSelectionMismatch,
    HistoricalReadUnavailable,
    BranchReadUnavailable,
    FacetReadUnavailable,
    ReplaySourceRequestCoherenceFailure,
    SourceMaterializationRejected,
    AdapterCapabilityDrift,
    BuilderConfigurationConflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFailureRecord {
    failure_identity: SourceFailureRecordIdentity,
    declaration_identity: SourceDeclarationIdentity,
    selector_identity: Arc<str>,
    source_capability_digest: Arc<str>,
    failure_class: SourceFailureClass,
    delivery_error_kind: BridgeDeliveryErrorKind,
    detail: Arc<str>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl SourceFailureRecord {
    pub fn new(
        declaration_identity: SourceDeclarationIdentity,
        selector: &BridgeTruthViewSelector,
        required_capabilities: &BridgeSourceCapabilitySet,
        failure_class: SourceFailureClass,
        delivery_error_kind: BridgeDeliveryErrorKind,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        let detail = detail.into();
        let canonical_basis = Arc::<str>::from(format!(
            "source-failure-record|declaration={}|selector={}|capabilities={}|class:{failure_class:?}|delivery-kind:{delivery_error_kind:?}|detail={}",
            declaration_identity.as_str(),
            selector.selector_identity().as_str(),
            required_capabilities.digest(),
            detail.as_ref(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            failure_identity: SourceFailureRecordIdentity::admit_bridge_owned(format!(
                "source-failure-record:sha256:{digest:x}"
            )),
            declaration_identity,
            selector_identity: Arc::from(selector.selector_identity().as_str()),
            source_capability_digest: Arc::from(required_capabilities.digest()),
            failure_class,
            delivery_error_kind,
            detail,
            canonical_basis,
            digest: Arc::from(format!("source-failure-record:sha256:{digest:x}")),
        }
    }

    pub fn failure_identity(&self) -> &SourceFailureRecordIdentity {
        &self.failure_identity
    }

    pub fn declaration_identity(&self) -> &SourceDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn selector_identity(&self) -> &str {
        self.selector_identity.as_ref()
    }

    pub fn source_capability_digest(&self) -> &str {
        self.source_capability_digest.as_ref()
    }

    pub fn failure_class(&self) -> SourceFailureClass {
        self.failure_class
    }

    pub fn delivery_error_kind(&self) -> BridgeDeliveryErrorKind {
        self.delivery_error_kind.clone()
    }

    pub fn detail(&self) -> &str {
        self.detail.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
