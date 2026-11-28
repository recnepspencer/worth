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
    authority: BridgeTruthAuthorityIdentity<TruthCommitIdentity, BridgeCommitIdentityKind>,
}

impl BridgeAdmittedTruthCommitIdentity {
    pub(crate) fn admit(projection: TruthCommitIdentity) -> Self {
        Self {
            authority: admit_bridge_truth_authority_identity_for_kind::<_, BridgeCommitIdentityKind>(
                projection,
            ),
        }
    }

    pub fn projection(&self) -> &TruthCommitIdentity {
        self.authority.value()
    }

    pub fn bridge_trust_boundary(
        &self,
    ) -> BridgeTruthBoundaryBridgedIdentity<TruthCommitIdentity, BridgeCommitIdentityKind> {
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
    authority: BridgeTruthAuthorityIdentity<TruthSnapshotIdentity, BridgeSnapshotIdentityKind>,
}

impl BridgeAdmittedTruthSnapshotIdentity {
    pub(crate) fn admit(projection: TruthSnapshotIdentity) -> Self {
        Self {
            authority: admit_bridge_truth_authority_identity_for_kind::<
                _,
                BridgeSnapshotIdentityKind,
            >(projection),
        }
    }

    pub fn projection(&self) -> &TruthSnapshotIdentity {
        self.authority.value()
    }

    pub fn bridge_trust_boundary(
        &self,
    ) -> BridgeTruthBoundaryBridgedIdentity<TruthSnapshotIdentity, BridgeSnapshotIdentityKind> {
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
    authority:
        BridgeTruthAuthorityIdentity<RelationalBridgeRecordIdentityParts, BridgeRecordIdentityKind>,
}

impl BridgeAdmittedTruthRecordIdentity {
    pub(crate) fn admit(projection: RelationalBridgeRecordIdentityParts) -> Self {
        Self {
            authority: admit_bridge_truth_authority_identity_for_kind::<_, BridgeRecordIdentityKind>(
                projection,
            ),
        }
    }

    pub fn projection(&self) -> RelationalBridgeRecordIdentityParts {
        *self.authority.value()
    }

    pub fn bridge_trust_boundary(
        &self,
    ) -> BridgeTruthBoundaryBridgedIdentity<
        RelationalBridgeRecordIdentityParts,
        BridgeRecordIdentityKind,
    > {
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
