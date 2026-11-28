use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{
    BridgeIdentity, HistoricalEvaluationDeclarationIdentityTag, TruthViewSelectorIdentityTag,
};
use crate::input::envelope::{TruthBranchIdentity, TruthCommitIdentity};
use crate::policy::BridgeDiagnosticsTier;
use crate::snapshot::TruthSnapshotIdentity;

pub type BridgeTruthViewSelectorIdentity = BridgeIdentity<TruthViewSelectorIdentityTag>;
pub type HistoricalEvaluationDeclarationIdentity =
    BridgeIdentity<HistoricalEvaluationDeclarationIdentityTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeTruthViewKind {
    CommittedSnapshot,
    HistoricalCommit,
    BranchHead,
    BranchSnapshot,
    BranchCommit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeReplayMode {
    Disabled,
    Enabled,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeDeliveryIntent {
    PrepareOnly,
    DeliverInvalidation,
    PrepareSignalEvaluation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTruthViewSelector {
    selector_identity: BridgeTruthViewSelectorIdentity,
    view_kind: BridgeTruthViewKind,
    branch_identity: TruthBranchIdentity,
    commit_identity: Option<TruthCommitIdentity>,
    snapshot_identity: Option<TruthSnapshotIdentity>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeTruthViewSelector {
    pub fn committed_snapshot(
        branch_identity: TruthBranchIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self::new(
            BridgeTruthViewKind::CommittedSnapshot,
            branch_identity,
            None,
            Some(snapshot_identity),
        )
    }

    pub fn historical_commit(
        branch_identity: TruthBranchIdentity,
        commit_identity: TruthCommitIdentity,
    ) -> Self {
        Self::new(
            BridgeTruthViewKind::HistoricalCommit,
            branch_identity,
            Some(commit_identity),
            None,
        )
    }

    pub fn branch_head(branch_identity: TruthBranchIdentity) -> Self {
        Self::new(BridgeTruthViewKind::BranchHead, branch_identity, None, None)
    }

    pub fn branch_snapshot(
        branch_identity: TruthBranchIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self::new(
            BridgeTruthViewKind::BranchSnapshot,
            branch_identity,
            None,
            Some(snapshot_identity),
        )
    }

    pub fn branch_commit(
        branch_identity: TruthBranchIdentity,
        commit_identity: TruthCommitIdentity,
    ) -> Self {
        Self::new(
            BridgeTruthViewKind::BranchCommit,
            branch_identity,
            Some(commit_identity),
            None,
        )
    }

    fn new(
        view_kind: BridgeTruthViewKind,
        branch_identity: TruthBranchIdentity,
        commit_identity: Option<TruthCommitIdentity>,
        snapshot_identity: Option<TruthSnapshotIdentity>,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "truth-view-selector|kind:{view_kind:?}|branch:{}|commit:{}|snapshot:{}",
            branch_identity.canonical_label(),
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
        let selector_identity = BridgeTruthViewSelectorIdentity::admit_bridge_owned(format!(
            "truth-view-selector:sha256:{digest:x}"
        ));
        Self {
            selector_identity,
            view_kind,
            branch_identity,
            commit_identity,
            snapshot_identity,
            canonical_basis,
            digest: Arc::from(format!("truth-view-selector:sha256:{digest:x}")),
        }
    }

    pub fn selector_identity(&self) -> &BridgeTruthViewSelectorIdentity {
        &self.selector_identity
    }

    pub fn view_kind(&self) -> BridgeTruthViewKind {
        self.view_kind
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
pub struct ValidatedTruthViewSelectorSet {
    selectors: Arc<[BridgeTruthViewSelector]>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl ValidatedTruthViewSelectorSet {
    pub(crate) fn singleton(selector: BridgeTruthViewSelector) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "validated-truth-view-selector-set|selector={}",
            selector.canonical_basis()
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            selectors: Arc::from(vec![selector]),
            canonical_basis,
            digest: Arc::from(format!(
                "validated-truth-view-selector-set:sha256:{digest:x}"
            )),
        }
    }

    pub fn selectors(&self) -> &[BridgeTruthViewSelector] {
        &self.selectors
    }

    pub fn first(&self) -> &BridgeTruthViewSelector {
        &self.selectors[0]
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalEvaluationDeclaration {
    declaration_identity: HistoricalEvaluationDeclarationIdentity,
    validated_selector_set: ValidatedTruthViewSelectorSet,
    replay_mode: BridgeReplayMode,
    diagnostics_mode: BridgeDiagnosticsTier,
    delivery_intent: BridgeDeliveryIntent,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl HistoricalEvaluationDeclaration {
    pub fn new(
        selector: BridgeTruthViewSelector,
        replay_mode: BridgeReplayMode,
        diagnostics_mode: BridgeDiagnosticsTier,
        delivery_intent: BridgeDeliveryIntent,
    ) -> Self {
        let validated_selector_set = ValidatedTruthViewSelectorSet::singleton(selector);
        let canonical_basis = Arc::<str>::from(format!(
            "historical-evaluation-declaration|selectors={}|replay:{replay_mode:?}|delivery:{delivery_intent:?}",
            validated_selector_set.canonical_basis(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        let declaration_identity = HistoricalEvaluationDeclarationIdentity::admit_bridge_owned(
            format!("historical-evaluation-declaration:sha256:{digest:x}"),
        );
        Self {
            declaration_identity,
            validated_selector_set,
            replay_mode,
            diagnostics_mode,
            delivery_intent,
            canonical_basis,
            digest: Arc::from(format!(
                "historical-evaluation-declaration:sha256:{digest:x}"
            )),
        }
    }

    pub fn declaration_identity(&self) -> &HistoricalEvaluationDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn selector(&self) -> &BridgeTruthViewSelector {
        self.validated_selector_set.first()
    }

    pub fn validated_selector_set(&self) -> &ValidatedTruthViewSelectorSet {
        &self.validated_selector_set
    }

    pub fn replay_mode(&self) -> BridgeReplayMode {
        self.replay_mode
    }

    pub fn diagnostics_mode(&self) -> BridgeDiagnosticsTier {
        self.diagnostics_mode
    }

    pub fn delivery_intent(&self) -> BridgeDeliveryIntent {
        self.delivery_intent
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BridgeDeliveryIntent, BridgeReplayMode, BridgeTruthViewKind, BridgeTruthViewSelector,
        HistoricalEvaluationDeclaration,
    };

    use crate::policy::BridgeDiagnosticsTier;

    #[test]
    fn truth_view_selector_is_canonical_for_same_inputs() {
        let left = BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        );
        let right = BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        );

        assert_eq!(left, right);
        assert_eq!(left.view_kind(), BridgeTruthViewKind::BranchSnapshot);
        assert_eq!(
            left.canonical_basis(),
            "truth-view-selector|kind:BranchSnapshot|branch:relational-branch:analysis|commit:-|snapshot:relational-snapshot:1:version:1"
        );
    }

    #[test]
    fn declaration_identity_is_canonical_for_same_inputs() {
        let left = HistoricalEvaluationDeclaration::new(
            BridgeTruthViewSelector::historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            ),
            BridgeReplayMode::Required,
            BridgeDiagnosticsTier::Exhaustive,
            BridgeDeliveryIntent::PrepareSignalEvaluation,
        );
        let right = HistoricalEvaluationDeclaration::new(
            BridgeTruthViewSelector::historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            ),
            BridgeReplayMode::Required,
            BridgeDiagnosticsTier::Exhaustive,
            BridgeDeliveryIntent::PrepareSignalEvaluation,
        );

        assert_eq!(left, right);
        assert_eq!(
            left.canonical_basis(),
            format!(
                "historical-evaluation-declaration|selectors={}|replay:Required|delivery:PrepareSignalEvaluation",
                left.validated_selector_set().canonical_basis(),
            )
        );
    }

    #[test]
    fn declaration_identity_is_invariant_across_diagnostics_tiers() {
        let selector = BridgeTruthViewSelector::historical_commit(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        );
        let left = HistoricalEvaluationDeclaration::new(
            selector.clone(),
            BridgeReplayMode::Required,
            BridgeDiagnosticsTier::Standard,
            BridgeDeliveryIntent::PrepareSignalEvaluation,
        );
        let right = HistoricalEvaluationDeclaration::new(
            selector,
            BridgeReplayMode::Required,
            BridgeDiagnosticsTier::Exhaustive,
            BridgeDeliveryIntent::PrepareSignalEvaluation,
        );

        assert_eq!(left.declaration_identity(), right.declaration_identity());
        assert_eq!(left.digest(), right.digest());
    }
}
