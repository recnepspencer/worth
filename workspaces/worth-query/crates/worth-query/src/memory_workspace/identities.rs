use worth_runtime_bridge::facade::{
    BridgeIdentityEvidence, RelationalBridgeSnapshotIdentityParts, TruthCommitIdentity,
    TruthSnapshotIdentity,
};

use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use std::sync::Arc;
use worth_foundational::facade::admit_foundational_authority_identity;

use crate::identity_authority::{
    query_receipt_admission_authority, query_runtime_backend_authority, QueryCommitIdentityKind,
    QueryReceiptAuthorityIdentity, QueryRuntimeBackendAuthorityIdentity, QuerySnapshotIdentityKind,
};
#[path = "identities/commit_behavior.rs"]
mod commit_behavior;
#[path = "identities/entity.rs"]
mod entity;
pub use entity::WorthQueryEntityIdentity;

#[derive(Clone)]
pub struct WorthQueryCommitIdentity {
    inner: WorthQueryCommitIdentityInner,
    _authority: Option<QueryReceiptAuthorityIdentity<Arc<str>, QueryCommitIdentityKind>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WorthQueryCommitIdentityInner {
    Absent,
    RelationalBridge {
        bridge_identity: TruthCommitIdentity,
    },
    Preview {
        evidence_identity: WorthQueryEvidenceIdentity,
    },
}

impl WorthQueryCommitIdentity {
    pub fn from_bridge_commit_projection(bridge_identity: TruthCommitIdentity) -> Self {
        Self {
            inner: WorthQueryCommitIdentityInner::RelationalBridge { bridge_identity },
            _authority: None,
        }
    }

    pub(crate) fn from_relational_commit_id(commit_id: u64) -> Self {
        Self::from_bridge_commit_projection(TruthCommitIdentity::from_relational_commit_id(
            commit_id,
        ))
    }

    pub(crate) fn from_runtime_receipt_commit(commit_id: u64) -> Self {
        let bridge_identity = TruthCommitIdentity::from_relational_commit_id(commit_id);
        let authority_basis = bridge_identity.bridge_admission_evidence();
        let authority = admit_foundational_authority_identity(
            Arc::<str>::from(authority_basis.terminal_projection_for_reporting()),
            query_receipt_admission_authority(),
        );
        Self {
            inner: WorthQueryCommitIdentityInner::RelationalBridge { bridge_identity },
            _authority: Some(authority),
        }
    }

    pub(crate) fn preview(evidence_identity: WorthQueryEvidenceIdentity) -> Self {
        Self {
            inner: WorthQueryCommitIdentityInner::Preview { evidence_identity },
            _authority: None,
        }
    }

    pub fn admit_external_token(
        token: crate::identity_authority::QueryExternalIdentityToken<
            std::sync::Arc<str>,
            crate::identity_authority::QueryCommitIdentityKind,
        >,
    ) -> Self {
        super::truth_identity_admission::admit_external_commit_token(token)
    }

    pub fn absent() -> Self {
        Self {
            inner: WorthQueryCommitIdentityInner::Absent,
            _authority: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.inner, WorthQueryCommitIdentityInner::Absent)
    }
}

#[derive(Clone)]
pub struct WorthQuerySnapshotIdentity {
    inner: WorthQuerySnapshotIdentityInner,
    _authority: Option<QueryRuntimeBackendAuthorityIdentity<Arc<str>, QuerySnapshotIdentityKind>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WorthQuerySnapshotIdentityInner {
    EmptyRelationalState,
    RelationalBridge {
        bridge_identity: TruthSnapshotIdentity,
        parts: RelationalBridgeSnapshotIdentityParts,
    },
    Preview {
        evidence_identity: WorthQueryEvidenceIdentity,
    },
}

impl WorthQuerySnapshotIdentity {
    pub fn empty_relational_state() -> Self {
        Self {
            inner: WorthQuerySnapshotIdentityInner::EmptyRelationalState,
            _authority: None,
        }
    }

    pub(crate) fn empty_runtime_state() -> Self {
        let authority = admit_foundational_authority_identity(
            Arc::<str>::from("worth-query-runtime-empty-relational-snapshot-v1"),
            query_runtime_backend_authority(),
        );
        Self {
            inner: WorthQuerySnapshotIdentityInner::EmptyRelationalState,
            _authority: Some(authority),
        }
    }

    pub(crate) fn from_relational_snapshot(parts: RelationalBridgeSnapshotIdentityParts) -> Self {
        Self::from_bridge_snapshot_projection(TruthSnapshotIdentity::from_relational_snapshot(
            parts,
        ))
        .expect("relational snapshot projection retains its typed payload")
    }

    pub fn from_bridge_snapshot_projection(bridge_identity: TruthSnapshotIdentity) -> Option<Self> {
        let parts = bridge_identity.relational_snapshot_parts()?;
        Some(Self {
            inner: WorthQuerySnapshotIdentityInner::RelationalBridge {
                bridge_identity,
                parts,
            },
            _authority: None,
        })
    }

    pub(crate) fn from_runtime_snapshot(parts: RelationalBridgeSnapshotIdentityParts) -> Self {
        let bridge_identity = TruthSnapshotIdentity::from_relational_snapshot(parts);
        let authority_basis = bridge_identity.bridge_admission_evidence();
        let authority = admit_foundational_authority_identity(
            Arc::<str>::from(authority_basis.terminal_projection_for_reporting()),
            query_runtime_backend_authority(),
        );
        Self {
            inner: WorthQuerySnapshotIdentityInner::RelationalBridge {
                bridge_identity,
                parts,
            },
            _authority: Some(authority),
        }
    }

    pub(crate) fn preview(evidence_identity: WorthQueryEvidenceIdentity) -> Self {
        Self {
            inner: WorthQuerySnapshotIdentityInner::Preview { evidence_identity },
            _authority: None,
        }
    }

    pub fn admit_external_token(
        token: crate::identity_authority::QueryExternalIdentityToken<
            std::sync::Arc<str>,
            crate::identity_authority::QuerySnapshotIdentityKind,
        >,
    ) -> Self {
        super::truth_identity_admission::admit_external_snapshot_token(token)
    }

    pub(crate) fn bridge_identity(&self) -> Option<&TruthSnapshotIdentity> {
        match &self.inner {
            WorthQuerySnapshotIdentityInner::EmptyRelationalState => None,
            WorthQuerySnapshotIdentityInner::RelationalBridge {
                bridge_identity, ..
            } => Some(bridge_identity),
            WorthQuerySnapshotIdentityInner::Preview { .. } => None,
        }
    }

    pub(crate) fn relational_parts(&self) -> Option<RelationalBridgeSnapshotIdentityParts> {
        match &self.inner {
            WorthQuerySnapshotIdentityInner::EmptyRelationalState => None,
            WorthQuerySnapshotIdentityInner::RelationalBridge { parts, .. } => Some(*parts),
            WorthQuerySnapshotIdentityInner::Preview { .. } => None,
        }
    }

    pub fn evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        match &self.inner {
            WorthQuerySnapshotIdentityInner::EmptyRelationalState => {
                WorthQueryEvidenceIdentity::compose(
                    WorthQueryEvidenceScope::WriteReceiptSnapshotIdentity,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("snapshot_state"),
                    "empty-relational",
                )
                .seal()
            }
            WorthQuerySnapshotIdentityInner::RelationalBridge { parts, .. } => {
                WorthQueryEvidenceIdentity::compose(
                    WorthQueryEvidenceScope::WriteReceiptSnapshotIdentity,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("relational_snapshot_id"),
                    parts.snapshot_id() as usize,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("relational_version_id"),
                    parts.version_id() as usize,
                )
                .seal()
            }
            WorthQuerySnapshotIdentityInner::Preview { evidence_identity } => {
                evidence_identity.clone()
            }
        }
    }

    pub fn bridge_admission_evidence(&self) -> BridgeIdentityEvidence {
        match &self.inner {
            WorthQuerySnapshotIdentityInner::RelationalBridge {
                bridge_identity, ..
            } => bridge_identity.bridge_admission_evidence(),
            WorthQuerySnapshotIdentityInner::EmptyRelationalState
            | WorthQuerySnapshotIdentityInner::Preview { .. } => {
                self.evidence_identity().bridge_evidence_identity()
            }
        }
    }

    pub fn terminal_projection_for_reporting(&self) -> String {
        self.evidence_identity()
            .terminal_projection_for_reporting()
            .to_string()
    }

    pub fn is_same_current_identity_as(&self, candidate: &Self) -> bool {
        self._authority.is_some() && candidate._authority.is_some() && self.inner == candidate.inner
    }

    pub(crate) fn has_current_authority(&self) -> bool {
        self._authority.is_some()
    }

    pub(crate) fn admit_runtime_backend_authority(mut self) -> Self {
        if self._authority.is_none() {
            let authority_basis = self.evidence_identity();
            self._authority = Some(admit_foundational_authority_identity(
                Arc::<str>::from(authority_basis.terminal_projection_for_reporting()),
                query_runtime_backend_authority(),
            ));
        }
        self
    }

    pub(crate) fn matches_admitted_historical_projection(&self, declared_basis: &str) -> bool {
        declared_basis == self.evidence_identity().as_str()
            || declared_basis
                == self
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting()
    }
}

impl std::fmt::Debug for WorthQuerySnapshotIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQuerySnapshotIdentity")
            .field(
                "posture",
                &if self._authority.is_some() {
                    "current"
                } else {
                    "projection"
                },
            )
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthQuerySnapshotIdentity {
    fn eq(&self, candidate: &Self) -> bool {
        self.inner == candidate.inner
    }
}

impl Eq for WorthQuerySnapshotIdentity {}
