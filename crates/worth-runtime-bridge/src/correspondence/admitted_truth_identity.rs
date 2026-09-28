use std::sync::Arc;

use crate::identity_authority::{
    admit_bridge_truth_authority_identity_for_kind, BridgeCommitIdentityKind,
    BridgeRecordIdentityKind, BridgeSnapshotIdentityKind, BridgeTruthAuthorityIdentity,
    BridgeTruthBoundaryBridgedIdentity,
};
use crate::input::envelope::TruthCommitIdentity;
use crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts;
use crate::snapshot::TruthSnapshotIdentity;

/// Bridge-owned admission that a commit projection came from the registered
/// authoritative source and survived correspondence delivery preflight.
#[derive(Clone)]
pub struct BridgeAdmittedTruthCommitIdentity {
    projection: TruthCommitIdentity,
    authority: BridgeTruthAuthorityIdentity<Arc<str>, BridgeCommitIdentityKind>,
}

impl BridgeAdmittedTruthCommitIdentity {
    pub(crate) fn admit(projection: TruthCommitIdentity) -> Self {
        let authority = admit_bridge_truth_authority_identity_for_kind::<BridgeCommitIdentityKind>(
            Arc::from(projection.as_str()),
        );
        Self {
            projection,
            authority,
        }
    }

    pub fn projection(&self) -> &TruthCommitIdentity {
        &self.projection
    }

    pub fn bridge_trust_boundary(
        &self,
    ) -> BridgeTruthBoundaryBridgedIdentity<Arc<str>, BridgeCommitIdentityKind> {
        self.authority.clone().bridge_trust_boundary()
    }
}

impl std::fmt::Debug for BridgeAdmittedTruthCommitIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeAdmittedTruthCommitIdentity")
            .field("posture", &"current")
            .finish_non_exhaustive()
    }
}

/// Bridge-owned admission that a snapshot projection was loaded by the
/// registered source and retained by an admitted delivery.
#[derive(Clone)]
pub struct BridgeAdmittedTruthSnapshotIdentity {
    projection: TruthSnapshotIdentity,
    authority: BridgeTruthAuthorityIdentity<Arc<str>, BridgeSnapshotIdentityKind>,
}

impl BridgeAdmittedTruthSnapshotIdentity {
    pub(crate) fn admit(projection: TruthSnapshotIdentity) -> Self {
        let authority = admit_bridge_truth_authority_identity_for_kind::<BridgeSnapshotIdentityKind>(
            Arc::from(projection.as_str()),
        );
        Self {
            projection,
            authority,
        }
    }

    pub fn projection(&self) -> &TruthSnapshotIdentity {
        &self.projection
    }

    pub fn bridge_trust_boundary(
        &self,
    ) -> BridgeTruthBoundaryBridgedIdentity<Arc<str>, BridgeSnapshotIdentityKind> {
        self.authority.clone().bridge_trust_boundary()
    }
}

impl std::fmt::Debug for BridgeAdmittedTruthSnapshotIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeAdmittedTruthSnapshotIdentity")
            .field("posture", &"current")
            .finish_non_exhaustive()
    }
}

/// Bridge-owned admission of the exact Relational record named by one
/// delivered authoritative change.
#[derive(Clone)]
pub struct BridgeAdmittedTruthRecordIdentity {
    projection: RelationalBridgeRecordIdentityParts,
    authority: BridgeTruthAuthorityIdentity<Arc<str>, BridgeRecordIdentityKind>,
}

impl BridgeAdmittedTruthRecordIdentity {
    pub(crate) fn admit(projection: RelationalBridgeRecordIdentityParts) -> Self {
        let authority = admit_bridge_truth_authority_identity_for_kind::<BridgeRecordIdentityKind>(
            Arc::from(projection.bridge_entity_identity()),
        );
        Self {
            projection,
            authority,
        }
    }

    pub fn projection(&self) -> RelationalBridgeRecordIdentityParts {
        self.projection
    }

    pub fn bridge_trust_boundary(
        &self,
    ) -> BridgeTruthBoundaryBridgedIdentity<Arc<str>, BridgeRecordIdentityKind> {
        self.authority.clone().bridge_trust_boundary()
    }
}

impl std::fmt::Debug for BridgeAdmittedTruthRecordIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeAdmittedTruthRecordIdentity")
            .field("posture", &"current")
            .finish_non_exhaustive()
    }
}
