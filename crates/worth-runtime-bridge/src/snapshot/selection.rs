use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::input::envelope::{TruthBranchIdentity, TruthCommitIdentity};
use crate::snapshot::{
    BridgeTruthViewSelector, HistoricalEvaluationDeclaration, ResolvedTruthViewPolicy,
    SnapshotReadPacket, TruthSnapshotIdentity,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTruthViewAuthorityBasis {
    branch_identity: TruthBranchIdentity,
    commit_identity: Option<TruthCommitIdentity>,
    snapshot_identity: Option<TruthSnapshotIdentity>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeTruthViewAuthorityBasis {
    pub(crate) fn from_selector(selector: &BridgeTruthViewSelector) -> Self {
        Self::from_resolved(
            selector,
            selector.commit_identity().cloned(),
            selector.snapshot_identity().cloned(),
        )
    }

    pub(crate) fn from_resolved_snapshot(
        selector: &BridgeTruthViewSelector,
        commit_identity: TruthCommitIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self::from_resolved(selector, Some(commit_identity), Some(snapshot_identity))
    }

    fn from_resolved(
        selector: &BridgeTruthViewSelector,
        commit_identity: Option<TruthCommitIdentity>,
        snapshot_identity: Option<TruthSnapshotIdentity>,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "truth-view-authority|selector={}|selector-basis={}|branch={}|commit={}|snapshot={}",
            selector.selector_identity().as_str(),
            selector.canonical_basis(),
            selector.branch_identity().canonical_label(),
            commit_identity
                .as_ref()
                .map(TruthCommitIdentity::canonical_label)
                .unwrap_or_else(|| "-".to_owned()),
            snapshot_identity
                .as_ref()
                .map(TruthSnapshotIdentity::canonical_label)
                .unwrap_or_else(|| "-".to_owned()),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            branch_identity: selector.branch_identity().clone(),
            commit_identity,
            snapshot_identity,
            canonical_basis,
            digest: Arc::from(format!("truth-view-authority:sha256:{digest:x}")),
        }
    }

    pub fn branch_identity(&self) -> &TruthBranchIdentity {
        &self.branch_identity
    }

    pub fn commit_identity(&self) -> Option<&TruthCommitIdentity> {
        self.commit_identity.as_ref()
    }

    pub fn snapshot_identity(&self) -> Option<&TruthSnapshotIdentity> {
        self.snapshot_identity.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedTruthViewPacket {
    declaration: HistoricalEvaluationDeclaration,
    resolved_policy: ResolvedTruthViewPolicy,
    authority_basis: BridgeTruthViewAuthorityBasis,
    read_packet: SnapshotReadPacket,
    pinned_commit: Option<Arc<PinnedCommitSnapshot>>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl PlannedTruthViewPacket {
    pub(crate) fn new(
        declaration: HistoricalEvaluationDeclaration,
        resolved_policy: ResolvedTruthViewPolicy,
        authority_basis: BridgeTruthViewAuthorityBasis,
        read_packet: SnapshotReadPacket,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "planned-truth-view-packet|declaration={}|validated-selectors={}|policy={}|authority={}|read-packet={}",
            declaration.declaration_identity().as_str(),
            declaration.validated_selector_set().digest(),
            resolved_policy.digest(),
            authority_basis.digest(),
            read_packet.digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            declaration,
            resolved_policy,
            authority_basis,
            read_packet,
            pinned_commit: None,
            canonical_basis,
            digest: Arc::from(format!("planned-truth-view-packet:sha256:{digest:x}")),
        }
    }

    pub(crate) fn with_pinned_commit(mut self, pinned: Option<PinnedCommitSnapshot>) -> Self {
        self.pinned_commit = pinned.map(Arc::new);
        self
    }

    pub(crate) fn pinned_commit(&self) -> Option<&PinnedCommitSnapshot> {
        self.pinned_commit.as_deref()
    }

    pub fn declaration(&self) -> &HistoricalEvaluationDeclaration {
        &self.declaration
    }

    pub fn resolved_policy(&self) -> &ResolvedTruthViewPolicy {
        &self.resolved_policy
    }

    pub fn authority_basis(&self) -> &BridgeTruthViewAuthorityBasis {
        &self.authority_basis
    }

    pub fn read_packet(&self) -> &SnapshotReadPacket {
        &self.read_packet
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Clone)]
pub(crate) struct PinnedCommitSnapshot {
    pub(crate) source: Arc<dyn crate::facade::SnapshotReadSource>,
    pub(crate) reader: crate::relational_source::snapshot_reading::RuntimePublicationSnapshotReader,
}

impl std::fmt::Debug for PinnedCommitSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PinnedCommitSnapshot")
            .finish_non_exhaustive()
    }
}

impl PartialEq for PinnedCommitSnapshot {
    fn eq(&self, other: &Self) -> bool {
        use crate::snapshot::TruthSnapshotReader as _;
        Arc::ptr_eq(&self.source, &other.source)
            && self.reader.snapshot_identity() == other.reader.snapshot_identity()
    }
}

impl Eq for PinnedCommitSnapshot {}

#[cfg(test)]
mod tests {
    use super::{BridgeTruthViewAuthorityBasis, PlannedTruthViewPacket};

    use crate::policy::BridgeDiagnosticsTier;
    use crate::snapshot::{
        BridgeDeliveryIntent, BridgeReplayMode, BridgeTruthViewSelector,
        HistoricalEvaluationDeclaration, ResolvedTruthViewPolicy, SnapshotReadPacket,
        TruthViewReplayContinuity, TruthViewRetentionAdmission, TruthViewSourceCapability,
    };

    #[test]
    fn authority_basis_is_canonical_for_same_selector() {
        let selector = BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        );

        let left = BridgeTruthViewAuthorityBasis::from_selector(&selector);
        let right = BridgeTruthViewAuthorityBasis::from_selector(&selector);

        assert_eq!(left, right);
        assert_eq!(
            left.canonical_basis(),
            format!(
                "truth-view-authority|selector={}|selector-basis={}|branch=relational-branch:analysis|commit=-|snapshot=relational-snapshot:1:version:1",
                selector.selector_identity().as_str(),
                selector.canonical_basis(),
            )
        );
    }

    #[test]
    fn planned_truth_view_packet_is_canonical_for_same_inputs() {
        let declaration = HistoricalEvaluationDeclaration::new(
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            BridgeReplayMode::Enabled,
            BridgeDiagnosticsTier::Standard,
            BridgeDeliveryIntent::PrepareSignalEvaluation,
        );
        let policy = ResolvedTruthViewPolicy::admitted(
            &declaration,
            TruthViewRetentionAdmission::SnapshotResident,
            TruthViewSourceCapability::DirectSnapshotRead,
            TruthViewReplayContinuity::ReplayPermitted,
        );
        let authority = BridgeTruthViewAuthorityBasis::from_selector(declaration.selector());
        let left = PlannedTruthViewPacket::new(
            declaration.clone(),
            policy.clone(),
            authority.clone(),
            SnapshotReadPacket::new(vec![]),
        );
        let right = PlannedTruthViewPacket::new(
            declaration,
            policy,
            authority,
            SnapshotReadPacket::new(vec![]),
        );

        assert_eq!(left, right);
        assert_eq!(
            left.canonical_basis(),
            format!(
                "planned-truth-view-packet|declaration={}|validated-selectors={}|policy={}|authority={}|read-packet={}",
                left.declaration().declaration_identity().as_str(),
                left.declaration().validated_selector_set().digest(),
                left.resolved_policy().digest(),
                left.authority_basis().digest(),
                left.read_packet().digest(),
            )
        );
    }
}
