use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::input::envelope::TruthBranchIdentity;
use crate::preview::{
    BridgePreviewSessionHandle, BridgePreviewSessionIdentity, BridgePreviewSessionLivenessObserver,
};
use crate::snapshot::TruthSnapshotIdentity;

use super::super::{
    BridgeSubscriptionCounters, BridgeSubscriptionPreviewBasisIdentity,
    BridgeSubscriptionPreviewLifecycleIdentity, BridgeSubscriptionPreviewParentBasisIdentity,
    BridgeSubscriptionPreviewResidueScopeIdentity, BridgeSubscriptionPreviewScopeIdentity,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionPreviewBasisBinding {
    pub(super) preview_basis_identity: BridgeSubscriptionPreviewBasisIdentity,
    pub(super) preview_scope_identity: BridgeSubscriptionPreviewScopeIdentity,
    pub(super) preview_parent_basis_identity: BridgeSubscriptionPreviewParentBasisIdentity,
    pub(super) preview_lifecycle_identity: BridgeSubscriptionPreviewLifecycleIdentity,
    pub(super) preview_residue_scope_identity: BridgeSubscriptionPreviewResidueScopeIdentity,
    pub(super) session: BridgePreviewSessionLivenessObserver,
    truth_branch_identity: TruthBranchIdentity,
    truth_snapshot_identity: TruthSnapshotIdentity,
    pub(super) parent_truth_view_basis_digest: Arc<str>,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    pub(super) digest: Arc<str>,
}

impl BridgeSubscriptionPreviewBasisBinding {
    pub(crate) fn admit(preview: &BridgePreviewSessionHandle) -> Self {
        let session = preview.liveness_observer();
        let name = session.session_identity().as_str();
        let branches = preview.branches();
        let truth_branch_identity = branches.relational().clone();
        let truth_snapshot_identity = preview.snapshot_identity().clone();
        let parent = crate::facade::BridgeTruthViewSelector::branch_snapshot(
            truth_branch_identity.clone(),
            truth_snapshot_identity.clone(),
        );
        let parent_digest = parent.digest();
        let signal = branches.signal().as_str();
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-preview-basis|session={name:?}|incarnation={}|signal={signal:?}|parent={parent_digest}",
            session.incarnation(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            preview_basis_identity: BridgeSubscriptionPreviewBasisIdentity::admit_bridge_owned(
                format!("bridge-subscription-preview-basis-id:sha256:{digest:x}"),
            ),
            preview_scope_identity: BridgeSubscriptionPreviewScopeIdentity::admit_bridge_owned(
                format!("bridge-subscription-preview-scope-id:sha256:{digest:x}"),
            ),
            preview_parent_basis_identity:
                BridgeSubscriptionPreviewParentBasisIdentity::admit_bridge_owned(format!(
                    "bridge-subscription-preview-parent-basis-id:sha256:{parent_digest}"
                )),
            preview_lifecycle_identity:
                BridgeSubscriptionPreviewLifecycleIdentity::admit_bridge_owned(format!(
                    "bridge-subscription-preview-lifecycle-id:sha256:{digest:x}"
                )),
            preview_residue_scope_identity:
                BridgeSubscriptionPreviewResidueScopeIdentity::admit_bridge_owned(format!(
                    "bridge-subscription-preview-residue-scope-id:sha256:{digest:x}"
                )),
            session,
            truth_branch_identity,
            truth_snapshot_identity,
            parent_truth_view_basis_digest: Arc::from(parent_digest),
            counters: BridgeSubscriptionCounters::from_subscription_preview_basis_admission(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-subscription-preview-basis:sha256:{digest:x}"
            )),
        }
    }

    pub fn preview_basis_identity(&self) -> &BridgeSubscriptionPreviewBasisIdentity {
        &self.preview_basis_identity
    }
    pub fn preview_scope_identity(&self) -> &BridgeSubscriptionPreviewScopeIdentity {
        &self.preview_scope_identity
    }
    pub fn preview_parent_basis_identity(&self) -> &BridgeSubscriptionPreviewParentBasisIdentity {
        &self.preview_parent_basis_identity
    }
    pub fn preview_lifecycle_identity(&self) -> &BridgeSubscriptionPreviewLifecycleIdentity {
        &self.preview_lifecycle_identity
    }
    pub fn preview_residue_scope_identity(&self) -> &BridgeSubscriptionPreviewResidueScopeIdentity {
        &self.preview_residue_scope_identity
    }
    pub fn preview_session_identity(&self) -> &BridgePreviewSessionIdentity {
        self.session.session_identity()
    }
    pub fn truth_branch_identity(&self) -> &TruthBranchIdentity {
        &self.truth_branch_identity
    }
    pub fn truth_snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.truth_snapshot_identity
    }
    pub fn parent_truth_view_basis_digest(&self) -> &str {
        self.parent_truth_view_basis_digest.as_ref()
    }
    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }
    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
