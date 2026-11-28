use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::{BridgePolicyDeclaration, BridgePolicyFieldKind, BridgePolicySourceClass};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgePolicyRejectionKind {
    PolicySourceAmbiguity,
    UnsupportedExecutionMode,
    ReplayPolicyConflict,
    DiagnosticsPolicyConflict,
    ArtifactRetentionConflict,
    PreviewPolicyBoundaryViolation,
    TruthViewPolicyConflict,
    PolicyLoweringMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgePolicyRejectionStage {
    Validation,
    Admission,
    Lowering,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePolicyRejection {
    declaration_identity: super::BridgePolicyDeclarationIdentity,
    kind: BridgePolicyRejectionKind,
    stage: BridgePolicyRejectionStage,
    field_kind: BridgePolicyFieldKind,
    primary_source: BridgePolicySourceClass,
    conflicting_source: BridgePolicySourceClass,
    detail: Arc<str>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgePolicyRejection {
    pub fn new(
        declaration: &BridgePolicyDeclaration,
        kind: BridgePolicyRejectionKind,
        stage: BridgePolicyRejectionStage,
        field_kind: BridgePolicyFieldKind,
        primary_source: BridgePolicySourceClass,
        conflicting_source: BridgePolicySourceClass,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        let detail = detail.into();
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-policy-rejection|declaration={}|kind:{kind:?}|stage:{stage:?}|field:{field_kind:?}|primary:{primary_source:?}|conflict:{conflicting_source:?}|detail:{}",
            declaration.declaration_identity().as_str(),
            detail.as_ref(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            declaration_identity: declaration.declaration_identity().clone(),
            kind,
            stage,
            field_kind,
            primary_source,
            conflicting_source,
            detail,
            canonical_basis,
            digest: Arc::from(format!("bridge-policy-rejection:sha256:{digest:x}")),
        }
    }

    pub fn declaration_identity(&self) -> &super::BridgePolicyDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn kind(&self) -> BridgePolicyRejectionKind {
        self.kind
    }

    pub fn stage(&self) -> BridgePolicyRejectionStage {
        self.stage
    }

    pub fn field_kind(&self) -> BridgePolicyFieldKind {
        self.field_kind
    }

    pub fn primary_source(&self) -> BridgePolicySourceClass {
        self.primary_source
    }

    pub fn conflicting_source(&self) -> BridgePolicySourceClass {
        self.conflicting_source
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

mod report;
