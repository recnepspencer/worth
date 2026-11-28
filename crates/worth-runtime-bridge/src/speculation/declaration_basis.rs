use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::snapshot::{BridgeTruthViewSelector, SnapshotReadContract};
use crate::source::BridgeSourceCapabilitySet;

use super::BridgeRequestKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgePreviewRetainedArtifactSchema {
    PreviewLifecycleArtifactsV1,
}

impl BridgePreviewRetainedArtifactSchema {
    pub fn canonical_basis(self) -> &'static str {
        match self {
            Self::PreviewLifecycleArtifactsV1 => {
                "preview-retained-artifact-schema|kind:PreviewLifecycleArtifactsV1"
            }
        }
    }

    pub fn digest(self) -> Arc<str> {
        let digest = Sha256::digest(self.canonical_basis().as_bytes());
        Arc::from(format!(
            "preview-retained-artifact-schema:sha256:{digest:x}"
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePreviewSessionBasis {
    truth_view_selector: BridgeTruthViewSelector,
    source_capabilities: BridgeSourceCapabilitySet,
    retained_artifact_schema: BridgePreviewRetainedArtifactSchema,
    truth_view_basis_digest: Arc<str>,
    source_capability_digest: Arc<str>,
    retained_artifact_schema_digest: Arc<str>,
}

impl BridgePreviewSessionBasis {
    pub fn new(
        truth_view_selector: BridgeTruthViewSelector,
        source_capabilities: BridgeSourceCapabilitySet,
        retained_artifact_schema: BridgePreviewRetainedArtifactSchema,
    ) -> Self {
        let truth_view_basis_digest = Arc::from(truth_view_selector.digest());
        let source_capability_digest = Arc::from(source_capabilities.digest());
        let retained_artifact_schema_digest = retained_artifact_schema.digest();

        Self {
            truth_view_selector,
            source_capabilities,
            retained_artifact_schema,
            truth_view_basis_digest,
            source_capability_digest,
            retained_artifact_schema_digest,
        }
    }

    pub fn truth_view_selector(&self) -> &BridgeTruthViewSelector {
        &self.truth_view_selector
    }

    pub fn source_capabilities(&self) -> &BridgeSourceCapabilitySet {
        &self.source_capabilities
    }

    pub fn retained_artifact_schema(&self) -> BridgePreviewRetainedArtifactSchema {
        self.retained_artifact_schema
    }

    pub fn truth_view_basis_digest(&self) -> &str {
        self.truth_view_basis_digest.as_ref()
    }

    pub fn source_capability_digest(&self) -> &str {
        self.source_capability_digest.as_ref()
    }

    pub fn retained_artifact_schema_digest(&self) -> &str {
        self.retained_artifact_schema_digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePreviewRequestShapeBasis {
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgePreviewRequestShapeBasis {
    pub fn from_request_kind(request_kind: BridgeRequestKind) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "preview-request-shape|request-kind:{request_kind:?}"
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            canonical_basis,
            digest: Arc::from(format!("preview-request-shape:sha256:{digest:x}")),
        }
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePreviewStructuralBasis {
    structural_contract_digest: Arc<str>,
    truth_view_basis_digest: Arc<str>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgePreviewStructuralBasis {
    /// Declares the read shape and truth view on which structural reuse depends.
    pub fn from_snapshot_contract(
        contract: &SnapshotReadContract,
        truth_view: &BridgeTruthViewSelector,
    ) -> Self {
        let structural_contract_digest: Arc<str> = Arc::from(format!(
            "sha256:{:x}",
            Sha256::digest(contract.canonical_basis())
        ));
        let truth_view_basis_digest = Arc::<str>::from(truth_view.digest());
        let canonical_basis = Arc::<str>::from(format!(
            "preview-structural-basis|contract={}|truth-view={}",
            structural_contract_digest.as_ref(),
            truth_view_basis_digest.as_ref(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            structural_contract_digest,
            truth_view_basis_digest,
            canonical_basis,
            digest: Arc::from(format!("preview-structural-basis:sha256:{digest:x}")),
        }
    }

    pub fn structural_contract_digest(&self) -> &str {
        self.structural_contract_digest.as_ref()
    }

    pub fn truth_view_basis_digest(&self) -> &str {
        self.truth_view_basis_digest.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
