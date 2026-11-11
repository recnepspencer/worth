use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{AsyncRequestTruthViewBasisIdentityTag, BridgeIdentity};
use crate::input::envelope::{TruthBranchIdentity, TruthCommitIdentity};
use crate::snapshot::TruthSnapshotIdentity;
use crate::subscription::BridgePreviewActiveSubscription;

pub type BridgeAsyncRequestTruthViewBasisIdentity =
    BridgeIdentity<AsyncRequestTruthViewBasisIdentityTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeAsyncRequestTruthViewBasisKind {
    Authoritative,
    BranchHead,
    Historical,
    Preview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncRequestTruthViewBasis {
    basis_identity: BridgeAsyncRequestTruthViewBasisIdentity,
    kind: BridgeAsyncRequestTruthViewBasisKind,
    truth_branch_identity: Option<TruthBranchIdentity>,
    truth_commit_identity: Option<TruthCommitIdentity>,
    truth_snapshot_identity: Option<TruthSnapshotIdentity>,
    preview_active_subscription_identity:
        Option<crate::subscription::BridgePreviewActiveSubscriptionIdentity>,
    preview_parent_truth_view_basis_digest: Option<Arc<str>>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeAsyncRequestTruthViewBasis {
    pub fn authoritative(
        branch_identity: TruthBranchIdentity,
        commit_identity: TruthCommitIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self::new(
            BridgeAsyncRequestTruthViewBasisKind::Authoritative,
            Some(branch_identity),
            Some(commit_identity),
            Some(snapshot_identity),
            None,
            None,
        )
    }

    pub fn branch_head(
        branch_identity: TruthBranchIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self::new(
            BridgeAsyncRequestTruthViewBasisKind::BranchHead,
            Some(branch_identity),
            None,
            Some(snapshot_identity),
            None,
            None,
        )
    }

    pub fn historical(
        branch_identity: TruthBranchIdentity,
        commit_identity: TruthCommitIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self::new(
            BridgeAsyncRequestTruthViewBasisKind::Historical,
            Some(branch_identity),
            Some(commit_identity),
            Some(snapshot_identity),
            None,
            None,
        )
    }

    pub fn preview(preview_active: &BridgePreviewActiveSubscription) -> Self {
        Self::new(
            BridgeAsyncRequestTruthViewBasisKind::Preview,
            None,
            None,
            None,
            Some(
                preview_active
                    .preview_active_subscription_identity()
                    .clone(),
            ),
            Some(Arc::from(
                preview_active.parent_truth_view_basis_digest().to_owned(),
            )),
        )
    }

    fn new(
        kind: BridgeAsyncRequestTruthViewBasisKind,
        truth_branch_identity: Option<TruthBranchIdentity>,
        truth_commit_identity: Option<TruthCommitIdentity>,
        truth_snapshot_identity: Option<TruthSnapshotIdentity>,
        preview_active_subscription_identity: Option<
            crate::subscription::BridgePreviewActiveSubscriptionIdentity,
        >,
        preview_parent_truth_view_basis_digest: Option<Arc<str>>,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-async-request-truth-view-basis|kind={kind:?}|branch={}|commit={}|snapshot={}|preview-active={}|preview-parent={}",
            truth_branch_identity
                .as_ref()
                .map(TruthBranchIdentity::as_str)
                .unwrap_or("-"),
            truth_commit_identity
                .as_ref()
                .map(TruthCommitIdentity::as_str)
                .unwrap_or("-"),
            truth_snapshot_identity
                .as_ref()
                .map(TruthSnapshotIdentity::as_str)
                .unwrap_or("-"),
            preview_active_subscription_identity
                .as_ref()
                .map(BridgeIdentity::as_str)
                .unwrap_or("-"),
            preview_parent_truth_view_basis_digest
                .as_deref()
                .unwrap_or("-"),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            basis_identity: BridgeAsyncRequestTruthViewBasisIdentity::admit_bridge_owned(format!(
                "bridge-async-request-truth-view-basis-id:sha256:{digest:x}"
            )),
            kind,
            truth_branch_identity,
            truth_commit_identity,
            truth_snapshot_identity,
            preview_active_subscription_identity,
            preview_parent_truth_view_basis_digest,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-async-request-truth-view-basis:sha256:{digest:x}"
            )),
        }
    }

    pub fn basis_identity(&self) -> &BridgeAsyncRequestTruthViewBasisIdentity {
        &self.basis_identity
    }

    pub fn kind(&self) -> BridgeAsyncRequestTruthViewBasisKind {
        self.kind
    }

    pub fn truth_branch_identity(&self) -> Option<&TruthBranchIdentity> {
        self.truth_branch_identity.as_ref()
    }

    pub fn truth_commit_identity(&self) -> Option<&TruthCommitIdentity> {
        self.truth_commit_identity.as_ref()
    }

    pub fn truth_snapshot_identity(&self) -> Option<&TruthSnapshotIdentity> {
        self.truth_snapshot_identity.as_ref()
    }

    pub fn preview_active_subscription_identity(
        &self,
    ) -> Option<&crate::subscription::BridgePreviewActiveSubscriptionIdentity> {
        self.preview_active_subscription_identity.as_ref()
    }

    pub fn preview_parent_truth_view_basis_digest(&self) -> Option<&str> {
        self.preview_parent_truth_view_basis_digest.as_deref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
#[cfg(test)]
#[path = "truth_basis_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
